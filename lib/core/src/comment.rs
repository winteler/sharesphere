use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Form;
use serde::{Deserialize, Serialize};
use sharesphere_utils::constants::{DELETED_MESSAGE};
use sharesphere_utils::errors::AppError;
use crate::post::Post;
use crate::ranking::{SortType, Vote};
use crate::sphere::SphereHeader;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::{get_user, ssr::check_user},
        session::ssr::get_db_pool,
    },
    sharesphere_utils::editor::ssr::get_html_and_markdown_strings,
    crate::ranking::{ssr::vote_on_content, VoteValue}
};
use sharesphere_auth::auth_widget::AuthorWidget;
use sharesphere_utils::node_utils::has_reached_scroll_load_threshold;
use sharesphere_utils::routes::{get_post_path, COMMENT_ID_QUERY_PARAM};
use sharesphere_utils::widget::{ContentBody, IsPinnedWidget, LoadIndicators, ScoreIndicator, TimeSinceWidget};
use crate::moderation::ModeratedBody;

pub const COMMENT_BATCH_SIZE: i64 = 50;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: i64,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_edited: bool,
    pub moderator_message: Option<String>,
    pub infringed_rule_id: Option<i64>,
    pub infringed_rule_title: Option<String>,
    pub parent_id: Option<i64>,
    pub post_id: i64,
    pub creator_id: i64,
    pub creator_name: String,
    pub is_creator_moderator: bool,
    pub moderator_id: Option<i64>,
    pub moderator_name: Option<String>,
    pub is_pinned: bool,
    pub score: i32,
    pub score_minus: i32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentWithChildren {
    pub comment: Comment,
    pub vote: Option<Vote>,
    pub child_comments: Vec<CommentWithChildren>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentWithContext {
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub comment: Comment,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub sphere_header: SphereHeader,
    pub satellite_id: Option<i64>,
    pub post_title: String,
}

impl Comment {
    pub fn is_active(&self) -> bool {
        self.delete_timestamp.is_none() && self.moderator_id.is_none()
    }
}

impl CommentWithContext {
    pub fn from_comment(
        comment: Comment,
        sphere_header: SphereHeader,
        post: &Post,
    ) -> CommentWithContext {
        CommentWithContext {
            comment,
            sphere_header,
            satellite_id: post.satellite_id,
            post_title: post.title.clone(),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_utils::errors::AppError;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::user::User;
    use crate::sphere::Sphere;

    use crate::post::ssr::{get_post_sphere, increment_post_comment_count};
    use crate::ranking::{SortType, VoteValue};

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow, Ord, PartialOrd, Serialize, Deserialize)]
    pub struct CommentWithVote {
        #[sqlx(flatten)]
        pub comment: Comment,
        pub vote_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub vote_comment_id: Option<Option<i64>>,
        pub vote_user_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl CommentWithVote {
        pub fn into_comment_with_children(self) -> CommentWithChildren {
            let comment_vote = if self.vote_id.is_some() {
                Some(Vote {
                    vote_id: self.vote_id.unwrap(),
                    user_id: self.vote_user_id.unwrap(),
                    comment_id: self.vote_comment_id.unwrap(),
                    post_id: self.vote_post_id.unwrap(),
                    value: VoteValue::from(self.value.unwrap()),
                    timestamp: self.vote_timestamp.unwrap(),
                })
            } else {
                None
            };

            CommentWithChildren {
                comment: self.comment,
                vote: comment_vote,
                child_comments: Vec::<CommentWithChildren>::new(),
            }
        }
    }

    pub async fn get_comment_by_id(
        comment_id: i64,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let comment = sqlx::query_as::<_, Comment>(
            "SELECT * FROM comments
            WHERE comment_id = $1",
        )
            .bind(comment_id)
            .fetch_one(db_pool)
            .await?;

        Ok(comment)
    }

    pub async fn get_comment_sphere(
        comment_id: i64,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as::<_, Sphere>(
            "SELECT s.*
            FROM spheres s
            JOIN posts p on p.sphere_id = s.sphere_id
            JOIN comments c on c.post_id = p.post_id
            WHERE c.comment_id = $1"
        )
            .bind(comment_id)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    fn process_comment_tree(
        comment_with_vote_vec: Vec<CommentWithVote>,
        allow_partial_tree: bool,
    ) -> Vec<CommentWithChildren> {
        let mut comment_tree = Vec::new();
        let mut stack = Vec::<(i64, Vec<CommentWithChildren>)>::new();
        for comment_with_vote in comment_with_vote_vec {
            let mut current = comment_with_vote.into_comment_with_children();

            if let Some((top_parent_id, child_comments)) = stack.last_mut() {
                if *top_parent_id == current.comment.comment_id {
                    // child comments at the top of the stack belong to the current comment, add them
                    current.child_comments.append(child_comments);
                    stack.pop();
                }
            }

            // if the current element has a parent, add it to the stack. Otherwise, add it to the comment tree as a root element.
            if let Some(parent_id) = current.comment.parent_id {
                if let Some((top_parent_id, top_child_comments)) = stack.last_mut() {
                    if parent_id == *top_parent_id {
                        // same parent id as the top of the stack, add it
                        top_child_comments.push(current);
                    } else {
                        // different parent id as the top of the stack, add it as a new element on the stack
                        stack.push((parent_id, Vec::from([current])));
                    }
                } else {
                    // no element on the stack, add the current comment as a new element
                    stack.push((parent_id, Vec::from([current])));
                }
            } else {
                comment_tree.push(current);
            }
        }

        // Handle comment trees that do not start from a root comment
        if allow_partial_tree && comment_tree.is_empty() && stack.len() == 1 {
            let (_, partial_comment_tree) = stack.into_iter().next().unwrap();
            comment_tree = partial_comment_tree;
        }

        comment_tree
    }

    pub async fn get_post_comment_tree(
        post_id: i64,
        sort_type: SortType,
        max_depth: Option<usize>,
        user_id: Option<i64>,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<CommentWithChildren>, AppError> {
        if post_id < 1 {
            return Err(AppError::new("Invalid post id."));
        }

        let sort_column = sort_type.to_order_by_code();

        let comment_with_vote_vec = sqlx::query_as::<_, CommentWithVote>(
            format!(
                "WITH RECURSIVE comment_tree AS (
                    (
                        SELECT
                            c.*,
                            v.vote_id,
                            v.user_id as vote_user_id,
                            v.post_id as vote_post_id,
                            v.comment_id as vote_comment_id,
                            v.value,
                            v.timestamp as vote_timestamp,
                            1 AS depth,
                            ARRAY[(c.is_pinned, c.{sort_column}, c.comment_id)] AS path
                        FROM comments c
                        LEFT JOIN votes v
                        ON v.comment_id = c.comment_id AND
                           v.user_id = $1
                        WHERE
                            c.post_id = $2 AND
                            c.parent_id IS NULL
                        ORDER BY c.is_pinned DESC, c.{sort_column} DESC
                        LIMIT $4
                        OFFSET $5
                    )
                    UNION ALL (
                        SELECT
                            n.*,
                            vr.vote_id,
                            vr.user_id as vote_user_id,
                            vr.post_id as vote_post_id,
                            vr.comment_id as vote_comment_id,
                            vr.value,
                            vr.timestamp as vote_timestamp,
                            r.depth + 1,
                            r.path || (n.is_pinned, n.{sort_column}, n.comment_id)
                        FROM comment_tree r
                        JOIN comments n ON n.parent_id = r.comment_id
                        LEFT JOIN votes vr
                        ON vr.comment_id = n.comment_id AND
                           vr.user_id = $1
                        WHERE ($3 IS NULL OR r.depth <= $3)
                    )
                )
                SELECT * FROM comment_tree
                ORDER BY path DESC"
            )
                .as_str(),
        )
            .bind(user_id)
            .bind(post_id)
            .bind(max_depth.map(|max_depth| (max_depth+ 1) as i64))
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        let comment_tree = process_comment_tree(comment_with_vote_vec, false);

        Ok(comment_tree)
    }

    /// Retrieves the comment tree of `comment_id`'s parent, itself and its children
    pub async fn get_comment_tree_by_id(
        comment_id: i64,
        sort_type: SortType,
        max_depth: Option<usize>,
        user_id: Option<i64>,
        db_pool: &PgPool,
    ) -> Result<CommentWithChildren, AppError> {
        if comment_id < 1 {
            return Err(AppError::new("Invalid comment id."));
        }

        let sort_column = sort_type.to_order_by_code();

        let comment_with_vote_vec = sqlx::query_as::<_, CommentWithVote>(
            format!(
                "WITH RECURSIVE comment_tree AS (
                    (
                        SELECT
                            c.*,
                            v.vote_id,
                            v.user_id as vote_user_id,
                            v.post_id as vote_post_id,
                            v.comment_id as vote_comment_id,
                            v.value,
                            v.timestamp as vote_timestamp,
                            1 AS depth,
                            ARRAY[(c.is_pinned, c.{sort_column}, c.comment_id)] AS path
                        FROM comments c
                        LEFT JOIN votes v
                        ON v.comment_id = c.comment_id AND
                           v.user_id = $1
                        WHERE
                            c.comment_id = $2
                        ORDER BY c.is_pinned DESC, c.{sort_column} DESC
                    )
                    UNION ALL (
                        SELECT
                            n.*,
                            vr.vote_id,
                            vr.user_id as vote_user_id,
                            vr.post_id as vote_post_id,
                            vr.comment_id as vote_comment_id,
                            vr.value,
                            vr.timestamp as vote_timestamp,
                            r.depth + 1,
                            r.path || (n.is_pinned, n.{sort_column}, n.comment_id)
                        FROM comment_tree r
                        JOIN comments n ON n.parent_id = r.comment_id
                        LEFT JOIN votes vr
                        ON vr.comment_id = n.comment_id AND
                           vr.user_id = $1
                        WHERE ($3 IS NULL OR r.depth <= $3)
                    )
                )
                SELECT * FROM (
                    SELECT * FROM comment_tree
                    ORDER BY path DESC
                    LIMIT $4
                ) AS filtered_comment_tree
                UNION ALL (
                    SELECT
                        c1.*,
                        v.vote_id,
                        v.user_id as vote_user_id,
                        v.post_id as vote_post_id,
                        v.comment_id as vote_comment_id,
                        v.value,
                        v.timestamp as vote_timestamp,
                        0 as depth,
                        ARRAY[(c1.is_pinned, c1.{sort_column}, c1.comment_id)] AS path
                    FROM comments c1
                    LEFT JOIN votes v
                        ON v.comment_id = c1.comment_id AND
                           v.user_id = $1
                    WHERE c1.comment_id = (
                        SELECT c2.parent_id
                        FROM comments c2
                        WHERE comment_id = $2
                    )
                )"
            ).as_str(),
        )
            .bind(user_id)
            .bind(comment_id)
            .bind(max_depth.map(|max_depth| (max_depth+ 1) as i64))
            .bind(COMMENT_BATCH_SIZE)
            .fetch_all(db_pool)
            .await?;

        let comment_tree = process_comment_tree(comment_with_vote_vec, true);

        if comment_tree.len() > 1 {
            return Err(AppError::new(format!("Comment tree for comment {comment_id} should have a single root element.")));
        }

        comment_tree.into_iter().next().ok_or(AppError::new(format!("No comment tree found for comment {comment_id}")))
    }

    pub async fn create_comment(
        post_id: i64,
        parent_comment_id: Option<i64>,
        comment: &str,
        markdown_comment: Option<&str>,
        is_pinned: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let sphere = get_post_sphere(post_id, &db_pool).await?;
        user.check_can_publish_on_sphere(&sphere.sphere_name)?;
        if comment.is_empty() {
            return Err(AppError::new("Cannot create empty comment."));
        }
        if is_pinned {
            user.check_permissions(&sphere.sphere_name, PermissionLevel::Moderate)?;
        }
        let comment = sqlx::query_as::<_, Comment>(
            "INSERT INTO comments (
                body, markdown_body, parent_id, post_id, is_pinned, creator_id, creator_name, is_creator_moderator
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        )
            .bind(comment)
            .bind(markdown_comment)
            .bind(parent_comment_id)
            .bind(post_id)
            .bind(is_pinned)
            .bind(user.user_id)
            .bind(user.username.clone())
            .bind(user.check_permissions(&sphere.sphere_name, PermissionLevel::Moderate).is_ok())
            .fetch_one(db_pool)
            .await?;

        increment_post_comment_count(post_id, &db_pool).await?;

        Ok(comment)
    }

    pub async fn update_comment(
        comment_id: i64,
        comment_body: &str,
        comment_markdown_body: Option<&str>,
        is_pinned: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        if is_pinned {
            let sphere = get_comment_sphere(comment_id, &db_pool).await?;
            user.check_permissions(&sphere.sphere_name, PermissionLevel::Moderate)?;
        }
        let comment = sqlx::query_as::<_, Comment>(
            "UPDATE comments SET
                body = $1,
                markdown_body = $2,
                is_pinned = $3,
                edit_timestamp = NOW()
            WHERE
                comment_id = $4 AND
                creator_id = $5 AND
                moderator_id IS NULL AND
                delete_timestamp IS NULL
            RETURNING *",
        )
            .bind(comment_body)
            .bind(comment_markdown_body)
            .bind(is_pinned)
            .bind(comment_id)
            .bind(user.user_id)
            .fetch_one(db_pool)
            .await?;

        Ok(comment)
    }

    pub async fn delete_comment(
        comment_id: i64,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let deleted_comment = sqlx::query_as::<_, Comment>(
            "UPDATE comments SET
                body = '',
                markdown_body = NULL,
                is_pinned = false,
                creator_name = '',
                edit_timestamp = NOW(),
                delete_timestamp = NOW()
            WHERE
                comment_id = $1 AND
                creator_id = $2 AND
                moderator_id IS NULL
            RETURNING *",
        )
            .bind(comment_id)
            .bind(user.user_id)
            .fetch_one(db_pool)
            .await?;

        Ok(deleted_comment)
    }

    #[cfg(test)]
    mod tests {
        use sharesphere_auth::user::User;
        use crate::comment::ssr::CommentWithVote;
        use crate::comment::{Comment};
        use crate::ranking::VoteValue;

        #[test]
        fn test_comment_join_vote_into_comment_with_children() {
            let user = User::default();
            let mut comment = Comment::default();
            comment.creator_id = user.user_id;

            let comment_without_vote = CommentWithVote {
                comment: comment.clone(),
                vote_id: None,
                vote_post_id: None,
                vote_comment_id: None,
                vote_user_id: None,
                value: None,
                vote_timestamp: None,
            };
            let comment_without_vote = comment_without_vote.into_comment_with_children();
            assert_eq!(comment_without_vote.comment, comment);
            assert_eq!(comment_without_vote.vote, None);
            assert_eq!(comment_without_vote.child_comments.is_empty(), true);

            let comment_with_vote = CommentWithVote {
                comment: comment.clone(),
                vote_id: Some(0),
                vote_post_id: Some(comment.post_id),
                vote_comment_id: Some(Some(comment.comment_id)),
                vote_user_id: Some(user.user_id),
                value: Some(1),
                vote_timestamp: Some(comment.create_timestamp),
            };
            let comment_with_vote = comment_with_vote.into_comment_with_children();
            let user_vote = comment_with_vote.vote.expect("CommentWithChildren should contain vote.");
            assert_eq!(comment_with_vote.comment, comment);
            assert_eq!(user_vote.user_id, user.user_id);
            assert_eq!(user_vote.comment_id, Some(comment.comment_id));
            assert_eq!(user_vote.value, VoteValue::Up);
            assert_eq!(user_vote.comment_id, Some(comment.comment_id));
            assert_eq!(comment_with_vote.child_comments.is_empty(), true);
        }
    }
}

#[server]
pub async fn get_post_comment_tree(
    post_id: i64,
    sort_type: SortType,
    max_depth: Option<usize>,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithChildren>, AppError> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_post_comment_tree(
        post_id,
        sort_type,
        max_depth,
        user_id,
        COMMENT_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(comment_tree)
}

#[server]
pub async fn get_comment_tree_by_id(
    comment_id: i64,
    sort_type: SortType,
    max_depth: Option<usize>,
) -> Result<CommentWithChildren, AppError> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_comment_tree_by_id(
        comment_id,
        sort_type,
        max_depth,
        user_id,
        &db_pool,
    ).await?;

    Ok(comment_tree)
}

#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
    is_markdown: bool,
    is_pinned: Option<bool>,
) -> Result<CommentWithChildren, AppError> {
    log::trace!("Create comment for post {post_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = get_html_and_markdown_strings(comment, is_markdown).await?;

    let mut comment = ssr::create_comment(
        post_id,
        parent_comment_id,
        comment.as_str(),
        markdown_comment.as_deref(),
        is_pinned.unwrap_or(false),
        &user,
        &db_pool,
    )
        .await?;

    let vote = vote_on_content(
        VoteValue::Up,
        comment.post_id,
        Some(comment.comment_id),
        None,
        &user,
        &db_pool,
    ).await?;

    comment.score = 1;

    Ok(CommentWithChildren {
        comment,
        vote,
        child_comments: Vec::<CommentWithChildren>::default(),
    })
}

#[server]
pub async fn edit_comment(
    comment_id: i64,
    comment: String,
    is_markdown: bool,
    is_pinned: Option<bool>,
) -> Result<Comment, AppError> {
    log::trace!("Edit comment {comment_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = get_html_and_markdown_strings(comment, is_markdown).await?;

    let comment = ssr::update_comment(
        comment_id,
        comment.as_str(),
        markdown_comment.as_deref(),
        is_pinned.unwrap_or(false),
        &user,
        &db_pool,
    )
        .await?;

    Ok(comment)
}

#[server]
pub async fn delete_comment(
    comment_id: i64,
) -> Result<(), AppError> {
    log::trace!("Edit comment {comment_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::delete_comment(
        comment_id,
        &user,
        &db_pool,
    ).await?;

    Ok(())
}

/// Displays the body of a comment
#[component]
pub fn CommentBody(
    #[prop(into)]
    comment: Signal<Comment>
) -> impl IntoView {
    view! {
        {
            move || comment.with(|comment| match (
                &comment.delete_timestamp,
                &comment.moderator_message,
                &comment.infringed_rule_title
            ) {
                (Some(_), _, _) => view! {
                    <div class="pl-2 text-left">
                        <ContentBody
                            body=String::from(DELETED_MESSAGE)
                            is_markdown=false
                        />
                    </div>
                }.into_any(),
                (None, Some(moderator_message), Some(infringed_rule_title)) => view! {
                    <ModeratedBody
                        infringed_rule_title=infringed_rule_title.clone()
                        moderator_message=moderator_message.clone()
                    />
                }.into_any(),
                _ => view! {
                    <div class="pl-2 text-left">
                        <ContentBody
                            body=comment.body.clone()
                            is_markdown=comment.markdown_body.is_some()
                        />
                    </div>
                }.into_any(),
            })
        }
    }.into_any()
}

/// Displays a comment with context (post title, sphere, score, etc.)
#[component]
pub fn CommentWithContext(
    comment: CommentWithContext
) -> impl IntoView {
    let score = comment.comment.score;
    let author = comment.comment.creator_name.clone();
    let is_moderator = comment.comment.is_creator_moderator;
    let timestamp = comment.comment.create_timestamp;
    let is_pinned = comment.comment.is_pinned;

    let post_path = get_post_path(&comment.sphere_header.sphere_name, comment.satellite_id, comment.comment.post_id);
    view! {
        <Form method="GET" action=post_path>
            <input name=COMMENT_ID_QUERY_PARAM value=comment.comment.comment_id class="hidden"/>
            <button class="w-full flex flex-col gap-1 pl-1 pt-1 pb-2 my-1 rounded-sm hover:bg-base-200">
                <CommentBody comment=comment.comment/>
                <div class="flex gap-1 items-center">
                    <SphereHeader sphere_header=comment.sphere_header/>
                    <div class="text-sm">"-"</div>
                    <div class="text-sm">{comment.post_title}</div>
                    <IsPinnedWidget is_pinned/>
                </div>
                <div class="flex gap-1">
                    <ScoreIndicator score/>
                    <AuthorWidget author is_moderator/>
                    <TimeSinceWidget timestamp/>
                </div>
            </button>
        </Form>
    }
}

/// Component to display a vector of comments and indicate when more need to be loaded
#[component]
pub fn CommentMiniatureList(
    /// signal containing the comments to display
    #[prop(into)]
    comment_vec: Signal<Vec<CommentWithContext>>,
    /// signal indicating new comments are being loaded
    #[prop(into)]
    is_loading: Signal<bool>,
    /// signal containing an eventual loading error in order to display it
    #[prop(into)]
    load_error: Signal<Option<AppError>>,
    /// signal to request loading additional comments
    additional_load_count: RwSignal<i32>,
    /// reference to the container of the comments in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col overflow-y-auto w-full pr-2 divide-y divide-base-content/20"
            on:scroll=move |_| if has_reached_scroll_load_threshold(list_ref) && !is_loading.get_untracked() {
                additional_load_count.update(|value| *value += 1);
            }
            node_ref=list_ref
        >
            <For
                each= move || comment_vec.get().into_iter()
                key=|comment| comment.comment.comment_id
                let(comment)
            >
                <li>
                    <CommentWithContext comment/>
                </li>
            </For>
        </ul>
        <LoadIndicators load_error is_loading/>
    }
}