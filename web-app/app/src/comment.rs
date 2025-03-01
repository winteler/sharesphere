use std::fmt;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Form;
use leptos_router::hooks::use_query_map;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};
use crate::app::GlobalState;
use crate::constants::{BEST_STR, RECENT_STR};
use crate::content::{CommentSortWidget, Content, ContentBody};
use crate::editor::{FormMarkdownEditor, TextareaData};
use crate::errors::AppError;
use crate::error_template::ErrorTemplate;
use crate::form::IsPinnedCheckbox;
use crate::icons::{AddCommentIcon, EditIcon, LoadingIcon};
use crate::moderation::{ModerateCommentButton, ModeratedBody, ModerationInfoButton};
use crate::post::{get_post_path, Post};
use crate::ranking::{ScoreIndicator, SortType, Vote, VotePanel};
use crate::sphere::{SphereHeader, SphereState};
use crate::unpack::{handle_additional_load, handle_initial_load, ActionError};
use crate::widget::{AuthorWidget, LoadIndicators, MinimizeMaximizeWidget, ModalDialog, ModalFormButtons, ModeratorWidget, LoginGuardedOpenModalButton, TimeSinceEditWidget, TimeSinceWidget};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::{get_user, ssr::check_user},
    editor::{ssr::get_html_and_markdown_bodies},
    post::ssr::increment_post_comment_count,
    ranking::{ssr::vote_on_content, VoteValue},
};

pub const COMMENT_ID_QUERY_PARAM: &str = "comment_id";
pub const COMMENT_BATCH_SIZE: i64 = 50;
const DEPTH_TO_COLOR_MAPPING_SIZE: usize = 6;
const DEPTH_TO_COLOR_MAPPING: [&str; DEPTH_TO_COLOR_MAPPING_SIZE] = [
    "bg-blue-500",
    "bg-green-500",
    "bg-yellow-500",
    "bg-orange-500",
    "bg-red-500",
    "bg-violet-500",
];

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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CommentSortType {
    Best,
    Recent,
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

impl fmt::Display for CommentSortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sort_type_name = match self {
            CommentSortType::Best => BEST_STR,
            CommentSortType::Recent => RECENT_STR,
        };
        write!(f, "{sort_type_name}")
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::constants::{BEST_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN};
    use crate::errors::AppError;
    use crate::post::ssr::{get_post_sphere};
    use crate::ranking::VoteValue;
    use crate::role::PermissionLevel;
    use crate::sphere::Sphere;
    use crate::user::User;

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

    impl CommentSortType {
        pub fn to_order_by_code(self) -> &'static str {
            match self {
                CommentSortType::Best => BEST_ORDER_BY_COLUMN,
                CommentSortType::Recent => RECENT_ORDER_BY_COLUMN,
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
                        LIMIT $3
                        OFFSET $4
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
                    )
                )
                SELECT * FROM comment_tree
                ORDER BY path DESC"
            )
                .as_str(),
        )
            .bind(user_id)
            .bind(post_id)
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
                    )
                )
                SELECT * FROM (
                    SELECT * FROM comment_tree
                    ORDER BY path DESC
                    LIMIT $3
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
                edit_timestamp = CURRENT_TIMESTAMP
            WHERE
                comment_id = $4 AND
                creator_id = $5
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

    #[cfg(test)]
    mod tests {
        use crate::comment::ssr::CommentWithVote;
        use crate::comment::{Comment, CommentSortType};
        use crate::constants::{BEST_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN};
        use crate::ranking::VoteValue;
        use crate::user::User;

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

        #[test]
        fn test_comment_sort_type_to_order_by_code() {
            assert_eq!(CommentSortType::Best.to_order_by_code(), BEST_ORDER_BY_COLUMN);
            assert_eq!(CommentSortType::Recent.to_order_by_code(), RECENT_ORDER_BY_COLUMN);
        }
    }
}

#[server]
pub async fn get_post_comment_tree(
    post_id: i64,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithChildren>, ServerFnError<AppError>> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_post_comment_tree(
        post_id,
        sort_type,
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
) -> Result<CommentWithChildren, ServerFnError<AppError>> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_comment_tree_by_id(
        comment_id,
        sort_type,
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
) -> Result<CommentWithChildren, ServerFnError<AppError>> {
    log::trace!("Create comment for post {post_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = get_html_and_markdown_bodies(comment, is_markdown).await?;

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
) -> Result<Comment, ServerFnError<AppError>> {
    log::trace!("Edit comment {comment_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = get_html_and_markdown_bodies(comment, is_markdown).await?;

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

/// Comment section component
#[component]
pub fn CommentSection(
    #[prop(into)]
    post_id: Signal<i64>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    is_loading: RwSignal<bool>,
    additional_load_count: RwSignal<i32>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let query_comment_id = move || match use_query_map().read().get(COMMENT_ID_QUERY_PARAM) {
        Some(comment_id_string) => comment_id_string.parse::<i64>().ok(),
        None => None,
    };

    view! {
        <CommentSortWidget sort_signal=state.comment_sort_type/>
        { move || {
            match query_comment_id() {
                Some(comment_id) => view! { <CommentTree comment_id comment_vec is_loading/> }.into_any(),
                None => view! { <CommentTreeVec post_id comment_vec is_loading additional_load_count/> }.into_any(),
            }
        }}
    }.into_any()
}

/// Component displaying a vector of comment trees
#[component]
pub fn CommentTreeVec(
    #[prop(into)]
    post_id: Signal<i64>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    is_loading: RwSignal<bool>,
    additional_load_count: RwSignal<i32>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let load_error = RwSignal::new(None);

    let _initial_comments_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let initial_load = get_post_comment_tree(post_id.get(), state.comment_sort_type.get(), 0).await;
            handle_initial_load(initial_load, comment_vec, load_error, None);
            is_loading.set(false);
        }
    );

    let _additional_comments_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = comment_vec.read_untracked().len();
                let additional_load = get_post_comment_tree(post_id.get(), state.comment_sort_type.get_untracked(), num_post).await;
                handle_additional_load(additional_load, comment_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <div class="flex flex-col h-fit">
            <For
                // a function that returns the items we're iterating over; a signal is fine
                each= move || comment_vec.get().into_iter().enumerate()
                // a unique key for each item as a reference
                key=|(_index, comment)| comment.comment.comment_id
                // renders each item to a view
                children=move |(index, comment_with_children)| {
                    view! {
                        <CommentBox
                            comment_with_children
                            depth=0
                            ranking=index
                        />
                    }.into_any()
                }
            />
        </div>
        <Show when=move || load_error.read().is_some()>
        {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(load_error.get().unwrap());
            view! {
                <div class="flex justify-start py-4"><ErrorTemplate outside_errors/></div>
            }.into_any()
        }
        </Show>
        <Show when=is_loading>
            <LoadingIcon/>
        </Show>
    }.into_any()
}

/// Component displaying a comment's tree
#[component]
pub fn CommentTree(
    #[prop(into)]
    comment_id: i64,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    is_loading: RwSignal<bool>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let load_error = RwSignal::new(None);

    // we set a signal with a local resource instead of using the resource directly to reuse the components from the ordinary comment tree
    let _comment_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let comment_tree = get_comment_tree_by_id(comment_id, state.comment_sort_type.get()).await;
            handle_initial_load(comment_tree.map(|comment| vec![comment]), comment_vec, load_error, None);
            is_loading.set(false);
        }
    );

    view! {
        <Form method="GET" action="">
            <button class="p-2 mt-2 rounded hover:bg-base-content/20 font-semibold">
                "Single comment tree view. Back to post."
            </button>
        </Form>
        { move || comment_vec.read().first().map(|comment| view! {
                <div class="flex flex-col h-fit">
                    <CommentBox
                        comment_with_children=comment.clone()
                        depth=0
                        ranking=0
                    />
                </div>
            })
        }
        <Show when=is_loading>
            <LoadingIcon/>
        </Show>
    }.into_any()
}

/// Comment box component
#[component]
pub fn CommentBox(
    comment_with_children: CommentWithChildren,
    depth: usize,
    ranking: usize,
) -> impl IntoView {
    let is_query_comment = move || match use_query_map().read().get(COMMENT_ID_QUERY_PARAM) {
        Some(query_comment_id) => query_comment_id.parse::<i64>().is_ok_and(|query_comment_id| query_comment_id == comment_with_children.comment.comment_id),
        None => false,
    };
    let comment = RwSignal::new(comment_with_children.comment);
    let child_comments = RwSignal::new(comment_with_children.child_comments);
    let maximize = RwSignal::new(true);
    let sidebar_css = move || {
        if *maximize.read() {
            "p-0.5 rounded hover:bg-base-content/20 flex flex-col justify-start items-center gap-1"
        } else {
            "p-0.5 rounded hover:bg-base-content/20 flex flex-col justify-center items-center"
        }
    };
    let color_bar_css = format!(
        "{} rounded-full h-full w-1 ",
        DEPTH_TO_COLOR_MAPPING[(depth + ranking) % DEPTH_TO_COLOR_MAPPING.len()]
    );

    view! {
        <div class="flex gap-1 py-1">
            <div
                class=sidebar_css
                on:click=move |_| maximize.update(|value: &mut bool| *value = !*value)
            >
                <MinimizeMaximizeWidget is_maximized=maximize/>
                <Show
                    when=maximize
                >
                    <div class=color_bar_css.clone()/>
                </Show>
            </div>
            <div class="flex flex-col gap-1">
                <div class="flex flex-col gap-1 p-1 rounded" class=(["border", "border-2", "border-base-content/50"], is_query_comment)>
                    <Show when=maximize>
                        <CommentBody comment/>
                    </Show>
                    <CommentWidgetBar
                        comment=comment
                        vote=comment_with_children.vote
                        child_comments
                    />
                </div>
                <div
                    class="flex flex-col"
                    class:hidden=move || !*maximize.read()
                >
                    <For
                        each= move || child_comments.get().into_iter().enumerate()
                        key=|(_index, comment)| comment.comment.comment_id
                        children=move |(index, comment_with_children)| {
                            view! {
                                <CommentBox
                                    comment_with_children
                                    depth=depth+1
                                    ranking=ranking+index
                                />
                            }.into_any()
                        }
                    />
                </div>
            </div>
        </div>
    }.into_any()
}

/// Displays the body of a comment
#[component]
pub fn CommentBody(
    #[prop(into)]
    comment: Signal<Comment>
) -> impl IntoView {
    view! {
        {
            move || comment.with(|comment| match (&comment.moderator_message, &comment.infringed_rule_title) {
                (Some(moderator_message), Some(infringed_rule_title)) => view! {
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

/// Component to encapsulate the widgets associated with each comment
#[component]
pub fn CommentWidgetBar(
    comment: RwSignal<Comment>,
    vote: Option<Vote>,
    child_comments: RwSignal<Vec<CommentWithChildren>>,
) -> impl IntoView {
    let (comment_id, post_id, score, author_id, author) =
        comment.with_untracked(|comment| {
            (
                comment.comment_id,
                comment.post_id,
                comment.score,
                comment.creator_id,
                comment.creator_name.clone(),
            )
        });
    let timestamp = Signal::derive(move || comment.read().create_timestamp);
    let edit_timestamp = Signal::derive(move || comment.read().edit_timestamp);
    let moderator = Signal::derive(move || comment.read().moderator_name.clone());
    let content = Signal::derive(move || Content::Comment(comment.get()));
    let is_moderator_comment = comment.read_untracked().is_creator_moderator;
    view! {
        <div class="flex gap-1">
            <VotePanel
                post_id
                comment_id=Some(comment_id)
                score
                vote
            />
            <CommentButton
                post_id
                comment_vec=child_comments
                parent_comment_id=Some(comment_id)
            />
            <EditCommentButton
                comment_id
                author_id
                comment
            />
            <ModerateCommentButton
                comment_id
                comment
            />
            <ModerationInfoButton content/>
            <AuthorWidget author is_moderator=is_moderator_comment/>
            <ModeratorWidget moderator/>
            <TimeSinceWidget timestamp/>
            <TimeSinceEditWidget edit_timestamp/>
        </div>
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

    let post_path = get_post_path(&comment.sphere_header.sphere_name, comment.satellite_id, comment.comment.post_id);
    view! {
        <Form method="GET" action=post_path>
            <input name=COMMENT_ID_QUERY_PARAM value=comment.comment.comment_id class="hidden"/>
            <button class="w-full flex flex-col gap-1 pl-1 pt-1 pb-2 my-1 rounded hover:bg-base-content/20">
                <CommentBody comment=comment.comment/>
                <div class="flex gap-1">
                    <SphereHeader sphere_header=comment.sphere_header/>
                    <div class="pt-1 pb-1.5 text-sm">"-"</div>
                    <div class="pt-1 pb-1.5 text-sm">{comment.post_title}</div>
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
    additional_load_count: RwSignal<i64>,
    /// reference to the container of the comments in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col overflow-y-auto w-full pr-2 divide-y divide-base-content/20"
            on:scroll=move |_| match list_ref.get() {
                Some(node_ref) => {
                    if node_ref.scroll_top() + node_ref.offset_height() >= node_ref.scroll_height() && !is_loading.get_untracked() {
                        additional_load_count.update(|value| *value += 1);
                    }
                },
                None => log::error!("Comment container 'ul' node failed to load."),
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
            <LoadIndicators load_error is_loading/>
        </ul>
    }
}

/// Component to open the comment form
#[component]
pub fn CommentButton(
    post_id: i64,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    #[prop(default = None)]
    parent_comment_id: Option<i64>,
) -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let comment_button_class = Signal::derive(move || match show_dialog.get() {
        true => "p-2 rounded-full bg-primary hover:bg-base-content/20 active:scale-95 transition duration-250",
        false => "p-2 rounded-full hover:bg-base-content/20 active:scale-95 transition duration-250",
    });

    view! {
        <div>
            <LoginGuardedOpenModalButton
                show_dialog
                button_class=comment_button_class
            >
                <AddCommentIcon/>
            </LoginGuardedOpenModalButton>
            <CommentDialog
                post_id
                parent_comment_id
                comment_vec
                show_dialog
            />
        </div>
    }.into_any()
}

/// Component to open the comment form and indicate comment count
#[component]
pub fn CommentButtonWithCount(
    post_id: i64,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    count: i32,
    #[prop(default = None)]
    parent_comment_id: Option<i64>,
) -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let comment_button_class = Signal::derive(move || match show_dialog.get() {
        true => "p-1.5 rounded-full bg-primary hover:bg-base-content/20 active:scale-95 transition duration-250",
        false => "p-1.5 rounded-full hover:bg-base-content/20 active:scale-95 transition duration-250",
    });

    view! {
        <div>
            <LoginGuardedOpenModalButton
                show_dialog
                button_class=comment_button_class
            >
                <div class="w-fit flex gap-1.5 items-center text-sm px-1">
                    <AddCommentIcon/>
                    {count}
                </div>
            </LoginGuardedOpenModalButton>
            <CommentDialog
                post_id
                parent_comment_id
                comment_vec
                show_dialog
            />
        </div>
    }.into_any()
}

/// Dialog to publish a comment
#[component]
pub fn CommentDialog(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <CommentForm
                post_id
                parent_comment_id
                comment_vec
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to publish a comment
#[component]
pub fn CommentForm(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let sphere_name = expect_context::<SphereState>().sphere_name;
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let comment_autosize = use_textarea_autosize(textarea_ref);
    let comment_data = TextareaData {
        content: comment_autosize.content,
        set_content: comment_autosize.set_content,
        textarea_ref,
    };

    let is_comment_empty = Signal::derive(move || comment_data.content.read().is_empty());

    let create_comment_action = ServerAction::<CreateComment>::new();

    Effect::new(move |_| {
        if let Some(Ok(comment)) = create_comment_action.value().get() {
            comment_vec.update(|comment_vec| comment_vec.insert(0, comment));
            show_form.set(false);
        }
    });

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Share a comment"</div>
            <ActionForm action=create_comment_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="post_id"
                        class="hidden"
                        value=post_id
                    />
                    <input
                        type="text"
                        name="parent_comment_id"
                        class="hidden"
                        value=parent_comment_id
                    />
                    <FormMarkdownEditor
                        name="comment"
                        is_markdown_name="is_markdown"
                        placeholder="Your comment..."
                        data=comment_data
                    />
                    <IsPinnedCheckbox sphere_name=sphere_name/>
                    <ModalFormButtons
                        disable_publish=is_comment_empty
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError action=create_comment_action.into()/>
        </div>
    }
}

/// Component to open the edit comment form
#[component]
pub fn EditCommentButton(
    comment_id: i64,
    author_id: i64,
    comment: RwSignal<Comment>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let show_dialog = RwSignal::new(false);
    let show_button = move || match &(*state.user.read()) {
        Some(Ok(Some(user))) => user.user_id == author_id,
        _ => false,
    };
    let comment_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };

    view! {
        <Suspense>
            <Show when=show_button>
                <div>
                    <button
                        class=comment_button_class
                        aria-expanded=move || show_dialog.get().to_string()
                        aria-haspopup="dialog"
                        on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
                    >
                        <EditIcon/>
                    </button>
                    <EditCommentDialog
                        comment_id
                        comment
                        show_dialog
                    />
                </div>
            </Show>
        </Suspense>
    }
}

/// Dialog to edit a comment
#[component]
pub fn EditCommentDialog(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <EditCommentForm
                comment_id
                comment
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to edit a comment
#[component]
pub fn EditCommentForm(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let sphere_name = expect_context::<SphereState>().sphere_name;
    let (current_body, is_markdown) =
        comment.with_untracked(|comment| match &comment.markdown_body {
            Some(body) => (body.clone(), true),
            None => (comment.body.clone(), false),
        });
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let comment_autosize = use_textarea_autosize(textarea_ref);
    let comment_data = TextareaData {
        content: comment_autosize.content,
        set_content: comment_autosize.set_content,
        textarea_ref,
    };
    comment_data.set_content.set(current_body);
    let is_comment_empty = Signal::derive(
        move || comment_data.content.read().is_empty()
    );
    let edit_comment_action = ServerAction::<EditComment>::new();

    let edit_comment_result = edit_comment_action.value();

    Effect::new(move |_| {
        if let Some(Ok(edited_comment)) = edit_comment_result.get() {
            comment.set(edited_comment);
            show_form.set(false);
        }
    });

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit your comment"</div>
            <ActionForm action=edit_comment_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="comment_id"
                        class="hidden"
                        value=comment_id
                    />
                    <FormMarkdownEditor
                        name="comment"
                        is_markdown_name="is_markdown"
                        placeholder="Your comment..."
                        data=comment_data
                        is_markdown
                    />
                    <IsPinnedCheckbox sphere_name value=comment.read_untracked().is_pinned/>
                    <ModalFormButtons
                        disable_publish=is_comment_empty
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError action=edit_comment_action.into()/>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::comment::CommentSortType;
    use crate::constants::{BEST_STR, RECENT_STR};

    #[test]
    fn test_post_sort_type_display() {
        assert_eq!(CommentSortType::Best.to_string(), BEST_STR);
        assert_eq!(CommentSortType::Recent.to_string(), RECENT_STR);
    }
}