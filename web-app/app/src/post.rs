use std::fmt;

use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use leptos_use::signal_debounced;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::app::ssr::get_db_pool;
use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
#[cfg(feature = "ssr")]
use crate::auth::{get_user, ssr::check_user};
use crate::comment::{get_post_comment_tree, CommentButton, CommentSection, CommentWithChildren, COMMENT_BATCH_SIZE};
use crate::constants::{BEST_STR, HOT_STR, RECENT_STR, TRENDING_STR};
use crate::content::{CommentSortWidget, ContentBody};
#[cfg(feature = "ssr")]
use crate::editor::get_styled_html_from_markdown;
use crate::editor::FormMarkdownEditor;
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
#[cfg(feature = "ssr")]
use crate::forum::FORUM_ROUTE_PREFIX;
use crate::forum::{get_matching_forum_name_set, ForumState};
use crate::icons::{EditIcon, InternalErrorIcon, LoadingIcon};
use crate::moderation::{ModeratePostButton, ModeratedBody};
#[cfg(feature = "ssr")]
use crate::ranking::{ssr::vote_on_content, VoteValue};
use crate::ranking::{SortType, Vote, VotePanel};
use crate::unpack::TransitionUnpack;
use crate::widget::{ActionError, AuthorWidget, ModalDialog, ModalFormButtons, ModeratorWidget, TimeSinceEditWidget, TimeSinceWidget};

pub const CREATE_POST_SUFFIX: &str = "/content";
pub const CREATE_POST_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);
pub const CREATE_POST_FORUM_QUERY_PARAM: &str = "forum";
pub const POST_ROUTE_PREFIX: &str = "/posts";
pub const POST_ROUTE_PARAM_NAME: &str = "post_name";
pub const POST_ROUTE: &str =
    concatcp!(POST_ROUTE_PREFIX, PARAM_ROUTE_PREFIX, POST_ROUTE_PARAM_NAME);
pub const POST_BATCH_SIZE: i64 = 50;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub title: String,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_nsfw: bool,
    pub spoiler_level: i32,
    pub tags: Option<String>,
    pub is_edited: bool,
    pub meta_post_id: Option<i64>,
    pub forum_id: i64,
    pub forum_name: String,
    pub creator_id: i64,
    pub creator_name: String,
    pub moderator_message: Option<String>,
    pub infringed_rule_id: Option<i64>,
    pub infringed_rule_title: Option<String>,
    pub moderator_id: Option<i64>,
    pub moderator_name: Option<String>,
    pub num_comments: i32,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: f32,
    pub trending_score: f32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub scoring_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PostWithUserInfo {
    pub post: Post,
    pub vote: Option<Vote>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PostSortType {
    Hot,
    Trending,
    Best,
    Recent,
}

impl fmt::Display for PostSortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sort_type_name = match self {
            PostSortType::Hot => HOT_STR,
            PostSortType::Trending => TRENDING_STR,
            PostSortType::Best => BEST_STR,
            PostSortType::Recent => RECENT_STR,
        };
        write!(f, "{sort_type_name}")
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::constants::{BEST_ORDER_BY_COLUMN, HOT_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN, TRENDING_ORDER_BY_COLUMN};
    use crate::errors::AppError;
    use crate::forum::Forum;
    use crate::ranking::VoteValue;
    use crate::user::User;
    use sqlx::PgPool;

    use super::*;

    #[derive(Clone, Debug, PartialEq, sqlx::FromRow, PartialOrd, Serialize, Deserialize)]
    pub struct PostJoinVote {
        #[sqlx(flatten)]
        pub post: super::Post,
        pub vote_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub vote_comment_id: Option<Option<i64>>,
        pub vote_user_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl PostJoinVote {
        pub fn into_post_with_info(self) -> PostWithUserInfo {
            let post_vote = if self.vote_id.is_some() {
                Some(Vote {
                    vote_id: self.vote_id.unwrap(),
                    post_id: self.vote_post_id.unwrap(),
                    comment_id: None,
                    user_id: self.vote_user_id.unwrap(),
                    value: VoteValue::from(self.value.unwrap()),
                    timestamp: self.vote_timestamp.unwrap(),
                })
            } else {
                None
            };

            PostWithUserInfo {
                post: self.post,
                vote: post_vote,
            }
        }
    }

    impl PostSortType {
        pub fn to_order_by_code(self) -> &'static str {
            match self {
                PostSortType::Hot => HOT_ORDER_BY_COLUMN,
                PostSortType::Trending => TRENDING_ORDER_BY_COLUMN,
                PostSortType::Best => BEST_ORDER_BY_COLUMN,
                PostSortType::Recent => RECENT_ORDER_BY_COLUMN,
            }
        }
    }

    pub async fn get_post_by_id(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = sqlx::query_as!(
            Post,
            "SELECT * FROM posts
            WHERE post_id = $1",
            post_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn get_post_with_info_by_id(
        post_id: i64,
        user: Option<&User>,
        db_pool: &PgPool,
    ) -> Result<PostWithUserInfo, AppError> {

        let user_id = match &user {
            Some(user) => Some(user.user_id),
            None => None,
        };

        let post_join_vote = sqlx::query_as::<_, PostJoinVote>(
            "SELECT p.*,
            v.vote_id,
            v.user_id as vote_user_id,
            v.post_id as vote_post_id,
            v.comment_id as vote_comment_id,
            v.value,
            v.timestamp as vote_timestamp
            FROM posts p
            LEFT JOIN votes v
            ON v.post_id = p.post_id AND
               v.comment_id IS NULL AND
               v.user_id = $1
            WHERE p.post_id = $2",
        )
        .bind(user_id)
        .bind(post_id)
        .fetch_one(db_pool)
        .await?;

        Ok(post_join_vote.into_post_with_info())
    }

    pub async fn get_post_forum(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<Forum, AppError> {
        let forum = sqlx::query_as!(
            Forum,
            "SELECT f.*
            FROM forums f
            JOIN posts p on p.forum_id = f.forum_id
            WHERE p.post_id = $1",
            post_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(forum)
    }

    pub async fn get_post_vec_by_forum_name(
        forum_name: &str,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT p.* FROM posts p \
                JOIN forums f on f.forum_id = p.forum_id \
                WHERE \
                    f.forum_name = $1 AND \
                    p.moderator_id IS NULL \
                ORDER BY {} DESC \
                LIMIT $2 \
                OFFSET $3",
                sort_type.to_order_by_code()
            )
            .as_str(),
        )
        .bind(forum_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(db_pool)
        .await?;

        Ok(post_vec)
    }

    pub async fn get_sorted_post_vec(
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT * FROM posts \
                WHERE moderator_id IS NULL \
                ORDER BY {} DESC \
                LIMIT $1 \
                OFFSET $2",
                sort_type.to_order_by_code()
            )
            .as_str(),
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(db_pool)
        .await?;

        Ok(post_vec)
    }

    pub async fn get_subscribed_post_vec(
        user_id: i64,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT p.* FROM posts p \
                JOIN forums f on f.forum_id = p.forum_id \
                WHERE \
                    f.forum_id IN (\
                        SELECT forum_id FROM forum_subscriptions WHERE user_id = $1\
                    ) AND \
                    p.moderator_id IS NULL \
                ORDER BY {} DESC \
                LIMIT $2 \
                OFFSET $3",
                sort_type.to_order_by_code(),
            )
            .as_str(),
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(db_pool)
        .await?;

        Ok(post_vec)
    }

    pub async fn create_post(
        forum_name: &str,
        post_title: &str,
        post_body: &str,
        post_markdown_body: Option<&str>,
        is_nsfw: bool,
        tag: Option<String>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        user.check_can_publish_on_forum(forum_name)?;
        if forum_name.is_empty() || post_title.is_empty() {
            return Err(AppError::new(
                "Cannot create content without a valid forum and title.",
            ));
        }
        let post = sqlx::query_as!(
            Post,
            "INSERT INTO posts (title, body, markdown_body, is_nsfw, tags, forum_id, forum_name, creator_id, creator_name)
             VALUES (
                $1, $2, $3, $4, $5,
                (SELECT forum_id FROM forums WHERE forum_name = $6),
                $6, $7, $8
            ) RETURNING *",
            post_title,
            post_body,
            post_markdown_body,
            is_nsfw,
            tag,
            forum_name,
            user.user_id,
            user.username,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn update_post(
        post_id: i64,
        post_title: &str,
        post_body: &str,
        post_markdown_body: Option<&str>,
        is_nsfw: bool,
        tag: Option<String>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        if post_title.is_empty() {
            return Err(AppError::new(
                "Cannot create content without a valid forum and title.",
            ));
        }

        let post = sqlx::query_as!(
            Post,
            "UPDATE posts SET
                title = $1,
                body = $2,
                markdown_body = $3,
                is_nsfw = $4,
                tags = $5,
                edit_timestamp = CURRENT_TIMESTAMP
            WHERE
                post_id = $6 AND
                creator_id = $7
            RETURNING *",
            post_title,
            post_body,
            post_markdown_body,
            is_nsfw,
            tag.unwrap_or_default(),
            post_id,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn update_post_scores(db_pool: &PgPool) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE posts \
            SET scoring_timestamp = CURRENT_TIMESTAMP \
            WHERE create_timestamp > (CURRENT_TIMESTAMP - INTERVAL '2 days')",
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::constants::{BEST_ORDER_BY_COLUMN, HOT_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN, TRENDING_ORDER_BY_COLUMN};
        use crate::post::ssr::PostJoinVote;
        use crate::post::{Post, PostSortType};
        use crate::ranking::VoteValue;
        use crate::user::User;

        #[test]
        fn test_post_join_vote_into_post_with_info() {
            let user = User::default();
            let mut user_post = Post::default();
            user_post.creator_id = user.user_id;

            let user_post_without_vote = PostJoinVote {
                post: user_post.clone(),
                vote_id: None,
                vote_post_id: None,
                vote_comment_id: None,
                vote_user_id: None,
                value: None,
                vote_timestamp: None,
            };
            let user_post_with_info = user_post_without_vote.into_post_with_info();
            assert_eq!(user_post_with_info.post, user_post);
            assert_eq!(user_post_with_info.vote, None);

            let user_post_with_vote = PostJoinVote {
                post: user_post.clone(),
                vote_id: Some(0),
                vote_post_id: Some(user_post.post_id),
                vote_comment_id: None,
                vote_user_id: Some(user.user_id),
                value: Some(1),
                vote_timestamp: Some(user_post.create_timestamp),
            };
            let user_post_with_info = user_post_with_vote.into_post_with_info();
            let user_vote = user_post_with_info.vote.expect("PostWithUserInfo should contain vote.");
            assert_eq!(user_post_with_info.post, user_post);
            assert_eq!(user_vote.user_id, user.user_id);
            assert_eq!(user_vote.post_id, user_post.post_id);
            assert_eq!(user_vote.value, VoteValue::Up);
            assert_eq!(user_vote.comment_id, None);

            let mut other_post = Post::default();
            other_post.creator_id = user.user_id + 1;

            let other_post_with_vote = PostJoinVote {
                post: other_post.clone(),
                vote_id: Some(0),
                vote_post_id: Some(other_post.post_id),
                vote_comment_id: None,
                vote_user_id: Some(user.user_id),
                value: Some(-1),
                vote_timestamp: Some(other_post.create_timestamp),
            };
            let other_post_with_info = other_post_with_vote.into_post_with_info();
            let user_vote = other_post_with_info.vote.expect("PostWithUserInfo should contain vote.");
            assert_eq!(other_post_with_info.post, other_post);
            assert_eq!(user_vote.user_id, user.user_id);
            assert_eq!(user_vote.post_id, other_post.post_id);
            assert_eq!(user_vote.value, VoteValue::Down);
            assert_eq!(user_vote.comment_id, None);
        }

        #[test]
        fn test_post_sort_type_to_order_by_code() {
            assert_eq!(PostSortType::Hot.to_order_by_code(), HOT_ORDER_BY_COLUMN);
            assert_eq!(PostSortType::Trending.to_order_by_code(), TRENDING_ORDER_BY_COLUMN);
            assert_eq!(PostSortType::Best.to_order_by_code(), BEST_ORDER_BY_COLUMN);
            assert_eq!(PostSortType::Recent.to_order_by_code(), RECENT_ORDER_BY_COLUMN);
        }
    }
}

#[server]
pub async fn get_post_with_info_by_id(post_id: i64) -> Result<PostWithUserInfo, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user = get_user().await?;
    Ok(ssr::get_post_with_info_by_id(post_id, user.as_ref(), &db_pool).await?)
}

#[server]
pub async fn get_sorted_post_vec(
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_sorted_post_vec(
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    )
    .await?;

    Ok(post_vec)
}

#[server]
pub async fn get_subscribed_post_vec(
    user_id: i64,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_subscribed_post_vec(
        user_id,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    )
    .await?;

    Ok(post_vec)
}

#[server]
pub async fn get_post_vec_by_forum_name(
    forum_name: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let post_vec = ssr::get_post_vec_by_forum_name(
        forum_name.as_str(),
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    )
    .await?;
    Ok(post_vec)
}

#[server]
pub async fn create_post(
    forum: String,
    title: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    tag: Option<String>,
) -> Result<(), ServerFnError> {
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = match is_markdown {
        true => (
            get_styled_html_from_markdown(body.clone()).await?,
            Some(body.as_str()),
        ),
        false => (body, None),
    };

    let post = ssr::create_post(
        forum.as_str(),
        title.as_str(),
        body.as_str(),
        markdown_body,
        is_nsfw,
        tag,
        &user,
        &db_pool,
    )
        .await?;

    // TODO: move in ssr::create_post?
    let _vote = vote_on_content(VoteValue::Up, post.post_id, None, None, &user, &db_pool).await?;

    log::trace!("Created post with id: {}", post.post_id);
    let new_post_path: &str = &(FORUM_ROUTE_PREFIX.to_owned()
        + "/"
        + forum.as_str()
        + POST_ROUTE_PREFIX
        + "/"
        + post.post_id.to_string().as_ref());
    leptos_axum::redirect(new_post_path);
    Ok(())
}

#[server]
pub async fn edit_post(
    post_id: i64,
    title: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    tag: Option<String>,
) -> Result<Post, ServerFnError> {
    log::trace!("Edit post {post_id}, title = {title}, body = {body}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = match is_markdown {
        true => (
            get_styled_html_from_markdown(body.clone()).await?,
            Some(body.as_str()),
        ),
        false => (body, None),
    };

    let post = ssr::update_post(
        post_id,
        title.as_str(),
        body.as_str(),
        markdown_body,
        is_nsfw,
        tag,
        &user,
        &db_pool,
    )
    .await?;

    log::trace!("Updated post with id: {}", post.post_id);
    Ok(post)
}

/// Get a memo returning the last valid post id from the url. Used to avoid triggering resources when leaving pages
pub fn get_post_id_memo(params: Memo<ParamsMap>) -> Memo<i64> {
    create_memo(move |current_post_id: Option<&i64>| {
        if let Some(new_post_id_string) =
            params.with(|params| params.get(POST_ROUTE_PARAM_NAME).cloned())
        {
            if let Ok(new_post_id) = new_post_id_string.parse::<i64>() {
                log::trace!("Current post id: {current_post_id:?}, new post id: {new_post_id}");
                new_post_id
            } else {
                log::trace!("Could not parse new post id: {new_post_id_string}, reuse current post id: {current_post_id:?}");
                current_post_id.cloned().unwrap_or_default()
            }
        } else {
            log::trace!("Could not find new post id, reuse current post id: {current_post_id:?}");
            current_post_id.cloned().unwrap_or_default()
        }
    })
}

/// Component to display a content
#[component]
pub fn Post() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_state = expect_context::<ForumState>();
    let params = use_params_map();
    let post_id = get_post_id_memo(params);

    let post_resource = create_resource(
        move || (post_id.get(), state.edit_post_action.version().get(), forum_state.moderate_post_action.version().get()),
        move |(post_id, _, _)| {
            log::debug!("Load data for post: {post_id}");
            get_post_with_info_by_id(post_id)
        },
    );

    let comment_vec = create_rw_signal(Vec::<CommentWithChildren>::with_capacity(
        COMMENT_BATCH_SIZE as usize,
    ));
    let additional_load_count = create_rw_signal(0);
    let is_loading = create_rw_signal(false);
    let load_error = create_rw_signal(None);
    let container_ref = create_node_ref::<html::Div>();

    // Effect for initial load, forum and sort changes,
    create_effect(move |_| {
        let post_id = post_id.get();
        let sort_type = state.comment_sort_type.get();
        is_loading.set(true);
        load_error.set(None);
        comment_vec.update(|post_vec| post_vec.clear());
        spawn_local(async move {
            match get_post_comment_tree(post_id, sort_type, 0).await {
                Ok(new_comment_vec) => {
                    comment_vec.update(|comment_vec| {
                        if let Some(list_ref) = container_ref.get_untracked() {
                            list_ref.set_scroll_top(0);
                        }
                        *comment_vec = new_comment_vec;
                    });
                }
                Err(e) => load_error.set(Some(AppError::from(&e))),
            }
            is_loading.set(false);
        });
    });

    // Effect for additional load upon reaching end of scroll
    create_effect(move |_| {
        if additional_load_count.get() > 0 {
            is_loading.set(true);
            load_error.set(None);
            let root_comment_count = comment_vec.with_untracked(|post_vec| post_vec.len());
            spawn_local(async move {
                match get_post_comment_tree(
                    post_id.get_untracked(),
                    state.comment_sort_type.get_untracked(),
                    root_comment_count,
                )
                .await
                {
                    Ok(mut new_comment_vec) => {
                        comment_vec.update(|comment_vec| comment_vec.append(&mut new_comment_vec))
                    }
                    Err(e) => load_error.set(Some(AppError::from(&e))),
                }
                is_loading.set(false);
            });
        }
    });

    view! {
        <div
            class="flex flex-col content-start gap-1 overflow-y-auto"
            on:scroll=move |_| match container_ref.get() {
                Some(node_ref) => {
                    if !is_loading.get_untracked() && node_ref.scroll_top() + node_ref.offset_height() >= node_ref.scroll_height() {
                        additional_load_count.update(|value| *value += 1);
                    }
                },
                None => log::error!("Post container 'div' node failed to load."),
            }
            node_ref=container_ref
        >
            <TransitionUnpack resource=post_resource let:post_with_info>
            {
                view! {
                    <div class="card">
                        <div class="card-body">
                            <div class="flex flex-col gap-4">
                                <h2 class="card-title">{post_with_info.post.title.clone()}</h2>
                                <PostBody post=&post_with_info.post/>
                                <PostWidgetBar post=post_with_info comment_vec/>
                            </div>
                        </div>
                    </div>
                }
            }
            </TransitionUnpack>
            <CommentSortWidget/>
            <CommentSection comment_vec/>
            <Show when=move || load_error.with(|error| error.is_some())>
            {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(load_error.get().unwrap());
                view! {
                    <div class="flex justify-start py-4"><ErrorTemplate outside_errors/></div>
                }
            }
            </Show>
            <Show when=is_loading>
                <LoadingIcon/>
            </Show>
        </div>
    }
}

/// Displays the body of a post
#[component]
pub fn PostBody<'a>(post: &'a Post) -> impl IntoView {

    view! {
        {
            match (&post.moderator_message, &post.infringed_rule_title) {
                (Some(moderator_message), Some(infringed_rule_title)) => view! { 
                    <ModeratedBody
                        infringed_rule_title=infringed_rule_title.clone()
                        moderator_message=moderator_message.clone()
                    />
                },
                _ => view! {
                    <ContentBody
                        body=post.body.clone()
                        is_markdown=post.markdown_body.is_some()
                    />
                }.into_view(),
            }
        }
    }
}

/// Component to encapsulate the widgets associated with each post
#[component]
fn PostWidgetBar<'a>(
    post: &'a PostWithUserInfo,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
) -> impl IntoView {
    view! {
        <div class="flex gap-1 content-center">
            <VotePanel
                post_id=post.post.post_id
                comment_id=None
                score=post.post.score
                vote=&post.vote
            />
            <CommentButton post_id=post.post.post_id comment_vec/>
            <EditPostButton author_id=post.post.creator_id post=&post.post/>
            <ModeratePostButton post_id=post.post.post_id/>
            <AuthorWidget author=post.post.creator_name.clone()/>
            <ModeratorWidget moderator=post.post.moderator_name.clone()/>
            <TimeSinceWidget timestamp=post.post.create_timestamp/>
            <TimeSinceEditWidget edit_timestamp=post.post.edit_timestamp/>
        </div>
    }
}

/// Component to edit a post
#[component]
pub fn EditPostButton<'a>(
    post: &'a Post,
    author_id: i64
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let show_dialog = create_rw_signal(false);
    let edit_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <div>
            {
                move || state.user.map(|result| match result {
                    Ok(Some(user)) if user.user_id == author_id => view! {
                        <button
                            class=edit_button_class
                            aria-expanded=move || show_dialog.get().to_string()
                            aria-haspopup="dialog"
                            on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
                        >
                            <EditIcon/>
                        </button>
                    }.into_view(),
                    _ => View::default()
                })
            }
            <EditPostDialog
                post_id=post.post_id
                post_title=post.title.clone()
                post_body=post.body.clone()
                markdown_body=post.markdown_body.clone()
                show_dialog
            />
        </div>
    }
}

/// Component to create a new post
#[component]
pub fn CreatePost() -> impl IntoView {
    let create_post_action = create_server_action::<CreatePost>();
    let create_post_result = create_post_action.value();
    // check if the server has returned an error
    let has_error = move || create_post_result.with(|val| matches!(val, Some(Err(_))));

    let query = use_query_map();
    let forum_query = move || {
        query.with_untracked(|query| {
            query
                .get(CREATE_POST_FORUM_QUERY_PARAM)
                .unwrap_or(&String::default())
                .to_string()
        })
    };

    let forum_name_input = create_rw_signal(forum_query());
    let forum_name_debounced: Signal<String> = signal_debounced(forum_name_input, 250.0);
    let post_body = create_rw_signal(String::new());
    let is_title_empty = create_rw_signal(true);
    let is_nsfw = create_rw_signal(false);
    let is_content_invalid =
        create_memo(move |_| is_title_empty.get() || post_body.with(|body| body.is_empty()));
    let is_nsfw_string = move || is_nsfw.get().to_string();

    let matching_forums_resource = create_resource(
        move || forum_name_debounced.get(),
        move |forum_prefix| get_matching_forum_name_set(forum_prefix),
    );

    view! {
        <div class="w-4/5 2xl:w-1/3 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=create_post_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Share a post!"</h2>
                    <div class="dropdown dropdown-end">
                        <input
                            tabindex="0"
                            type="text"
                            name="forum"
                            placeholder="Sphere"
                            autocomplete="off"
                            class="input input-bordered input-primary w-full h-input_m"
                            on:input=move |ev| {
                                forum_name_input.update(|name: &mut String| *name = event_target_value(&ev).to_lowercase());
                            }
                            prop:value=forum_name_input
                        />
                        <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-200 rounded-box w-full">
                            <TransitionUnpack resource=matching_forums_resource let:forum_set>
                            {
                                forum_set.iter().map(|forum_name| {
                                    view! {
                                        <li>
                                            <button type="button" value=forum_name on:click=move |ev| forum_name_input.update(|name| *name = event_target_value(&ev))>
                                                {forum_name}
                                            </button>
                                        </li>
                                    }
                                }).collect_view()
                            }
                            </TransitionUnpack>
                        </ul>
                    </div>
                    <input
                        type="text"
                        name="title"
                        placeholder="Title"
                        class="input input-bordered input-primary h-input_m"
                        autofocus
                        autocomplete="off"
                        on:input=move |ev| {
                            is_title_empty.set(event_target_value(&ev).is_empty());
                        }
                    />
                    <FormMarkdownEditor
                        name="body"
                        is_markdown_name="is_markdown"
                        placeholder="Content"
                        content=post_body
                    />
                    <div class="form-control">
                        <input type="text" name="is_nsfw" value=is_nsfw_string class="hidden"/>
                        <label class="cursor-pointer label p-0">
                            <span class="label-text">"NSFW content"</span>
                            <input type="checkbox" class="checkbox checkbox-primary" checked=is_nsfw on:click=move |_| is_nsfw.update(|value| *value = !*value)/>
                        </label>
                    </div>
                    <select name="tag" class="select select-bordered w-full max-w-xs">
                        <option disabled selected>"Tag"</option>
                        <option>"This should be"</option>
                        <option>"Customized"</option>
                    </select>
                    <button type="submit" class="btn btn-active btn-secondary" disabled=is_content_invalid>"Create"</button>
                </div>
            </ActionForm>
            <Show
                when=has_error
                fallback=move || ()
            >
                <div class="alert alert-error flex justify-center">
                    <InternalErrorIcon/>
                    <span>"Server error. Please reload the page and retry."</span>
                </div>
            </Show>
        </div>
    }
}

/// Dialog to edit a post
#[component]
pub fn EditPostDialog(
    post_id: i64,
    post_title: String,
    post_body: String,
    markdown_body: Option<String>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <EditPostForm
                post_id
                post_title=post_title.clone()
                post_body=post_body.clone()
                markdown_body=markdown_body.clone()
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to edit a post
#[component]
pub fn EditPostForm(
    post_id: i64,
    post_title: String,
    post_body: String,
    markdown_body: Option<String>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let (current_body, is_markdown) = match markdown_body {
        Some(body) => (body, true),
        None => (post_body, false),
    };
    let is_title_empty = create_rw_signal(false);
    let post = create_rw_signal(current_body);
    let is_post_empty = move || post.with(|post: &String| post.is_empty());

    let edit_post_result = state.edit_post_action.value();
    let has_error = move || edit_post_result.with(|val| matches!(val, Some(Err(_))));

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit your post"</div>
            <ActionForm action=state.edit_post_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="post_id"
                        class="hidden"
                        value=post_id
                    />
                    <input
                        type="text"
                        name="title"
                        placeholder="Title"
                        value=post_title
                        class="input input-bordered input-primary h-input_m"
                        autofocus
                        autocomplete="off"
                        on:input=move |ev| {
                            is_title_empty.set(event_target_value(&ev).is_empty());
                        }
                    />
                    <FormMarkdownEditor
                        name="body"
                        is_markdown_name="is_markdown"
                        placeholder="Content"
                        content=post
                        is_markdown
                    />
                    <ModalFormButtons
                        disable_publish=is_post_empty
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError has_error/>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{BEST_STR, HOT_STR, RECENT_STR, TRENDING_STR};
    use crate::post::PostSortType;

    #[test]
    fn test_post_sort_type_display() {
        assert_eq!(PostSortType::Hot.to_string(), HOT_STR);
        assert_eq!(PostSortType::Trending.to_string(), TRENDING_STR);
        assert_eq!(PostSortType::Best.to_string(), BEST_STR);
        assert_eq!(PostSortType::Recent.to_string(), RECENT_STR);
    }
}
