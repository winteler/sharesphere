use const_format::concatcp;
use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use leptos_router::params::ParamsMap;
use leptos_use::{signal_debounced, use_textarea_autosize};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

use crate::app::{GlobalState, PUBLISH_ROUTE};
use crate::comment::{get_post_comment_tree, CommentButton, CommentSection, CommentWithChildren, COMMENT_BATCH_SIZE};
use crate::constants::{BEST_STR, HOT_STR, RECENT_STR, TRENDING_STR};
use crate::content::{CommentSortWidget, Content, ContentBody};
use crate::editor::{FormMarkdownEditor, TextareaData};
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::form::{IsPinnedCheckbox, LabeledFormCheckbox};
use crate::icons::{EditIcon, LoadingIcon, NsfwIcon, SpoilerIcon};
use crate::moderation::{ModeratePostButton, ModeratedBody, ModerationInfoButton};
use crate::ranking::{SortType, Vote, VotePanel};
use crate::sphere::{get_matching_sphere_header_vec, SphereCategoryDropdown, SphereHeader, SphereState};
use crate::sphere_category::{get_sphere_category_vec, SphereCategoryBadge, SphereCategoryHeader};
use crate::unpack::{ActionError, ArcTransitionUnpack, TransitionUnpack};
use crate::widget::{AuthorWidget, CommentCountWidget, ModalDialog, ModalFormButtons, ModeratorWidget, TimeSinceEditWidget, TimeSinceWidget};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::{get_user, ssr::check_user},
    constants::PATH_SEPARATOR,
    editor::ssr::get_html_and_markdown_bodies,
    ranking::{ssr::vote_on_content, VoteValue},
    sphere::SPHERE_ROUTE_PREFIX,
};

pub const CREATE_POST_SUFFIX: &str = "/post";
pub const CREATE_POST_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);
pub const CREATE_POST_SPHERE_QUERY_PARAM: &str = "sphere";
pub const POST_ROUTE_PREFIX: &str = "/posts";
pub const POST_ROUTE_PARAM_NAME: &str = "post_name";
pub const POST_BATCH_SIZE: i64 = 50;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub title: String,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_nsfw: bool,
    pub is_spoiler: bool,
    pub category_id: Option<i64>,
    pub is_edited: bool,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub satellite_id: Option<i64>,
    pub creator_id: i64,
    pub creator_name: String,
    pub is_creator_moderator: bool,
    pub moderator_message: Option<String>,
    pub infringed_rule_id: Option<i64>,
    pub infringed_rule_title: Option<String>,
    pub moderator_id: Option<i64>,
    pub moderator_name: Option<String>,
    pub num_comments: i32,
    pub is_pinned: bool,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: f32,
    pub trending_score: f32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub scoring_timestamp: chrono::DateTime<chrono::Utc>,
}

// TODO try with flatten on option
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PostWithInfo {
    pub post: Post,
    pub sphere_category: Option<SphereCategoryHeader>,
    pub vote: Option<Vote>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PostWithSphereInfo {
    pub post: Post,
    pub sphere_category: Option<SphereCategoryHeader>,
    pub sphere_icon_url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PostSortType {
    Hot,
    Trending,
    Best,
    Recent,
}

impl PostWithSphereInfo {
    pub fn from_post(
        post: Post,
        sphere_category: Option<SphereCategoryHeader>,
        sphere_icon_url: Option<String>,
    ) -> Self {
        PostWithSphereInfo {
            post,
            sphere_category,
            sphere_icon_url,
        }
    }
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
    use super::*;
    use crate::colors::Color;
    use crate::constants::{BEST_ORDER_BY_COLUMN, HOT_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN, TRENDING_ORDER_BY_COLUMN};
    use crate::errors::AppError;
    use crate::ranking::VoteValue;
    use crate::role::PermissionLevel;
    use crate::sphere::Sphere;
    use crate::user::User;
    use sqlx::PgPool;

    #[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
    #[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
    pub struct PostJoinCategory {
        #[cfg_attr(feature = "ssr", sqlx(flatten))]
        pub post: Post,
        pub category_name: Option<String>,
        pub category_color: Option<Color>,
        pub sphere_icon_url: Option<String>,
    }

    #[derive(Clone, Debug, PartialEq, sqlx::FromRow, PartialOrd, Serialize, Deserialize)]
    pub struct PostJoinInfo {
        #[sqlx(flatten)]
        pub post: super::Post,
        pub category_name: Option<String>,
        pub category_color: Option<Color>,
        pub vote_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub vote_comment_id: Option<Option<i64>>,
        pub vote_user_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl PostJoinCategory {
        pub fn into_post_with_sphere_info(self) -> PostWithSphereInfo {
            let sphere_category = match (self.category_name, self.category_color) {
                (Some(category_name), Some(category_color)) => Some(SphereCategoryHeader {
                    category_name,
                    category_color,
                }),
                _ => None,
            };
            PostWithSphereInfo {
                post: self.post,
                sphere_category,
                sphere_icon_url: self.sphere_icon_url,
            }
        }
    }

    impl PostJoinInfo {
        pub fn into_post_with_info(self) -> PostWithInfo {
            let sphere_category = match (self.category_name, self.category_color) {
                (Some(category_name), Some(category_color)) => Some(SphereCategoryHeader {
                    category_name,
                    category_color,
                }),
                _ => None,
            };
            let post_vote = match (self.vote_id, self.vote_user_id, self.value, self.vote_timestamp) {
                (Some(vote_id), Some(vote_user_id), Some(value), Some(vote_timestamp)) => Some(Vote {
                    vote_id,
                    post_id: self.post.post_id,
                    comment_id: None,
                    user_id: vote_user_id,
                    value: VoteValue::from(value),
                    timestamp: vote_timestamp,
                }),
                _ => None,
            };

            PostWithInfo {
                post: self.post,
                sphere_category,
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
    ) -> Result<PostWithInfo, AppError> {

        let user_id = user.map(|user| user.user_id);

        let post_join_vote = sqlx::query_as::<_, PostJoinInfo>(
            "SELECT p.*,
                c.category_name,
                c.category_color,
                v.vote_id,
                v.user_id as vote_user_id,
                v.post_id as vote_post_id,
                v.comment_id as vote_comment_id,
                v.value,
                v.timestamp as vote_timestamp
            FROM posts p
            LEFT JOIN sphere_categories c on c.category_id = p.category_id
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

    pub async fn get_post_sphere(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as!(
            Sphere,
            "SELECT s.*
            FROM spheres s
            JOIN posts p on p.sphere_id = s.sphere_id
            WHERE p.post_id = $1",
            post_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn get_post_vec_by_sphere_name(
        sphere_name: &str,
        sphere_category_id: Option<i64>,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT p.* FROM posts p
                JOIN spheres s on s.sphere_id = p.sphere_id
                WHERE
                    s.sphere_name = $1 AND
                    p.category_id IS NOT DISTINCT FROM COALESCE($2, p.category_id) AND
                    p.moderator_id IS NULL AND
                    p.satellite_id IS NULL
                ORDER BY p.is_pinned DESC, {} DESC
                LIMIT $3
                OFFSET $4",
                sort_type.to_order_by_code(),
            ).as_str(),
        )
            .bind(sphere_name)
            .bind(sphere_category_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(post_vec)
    }

    pub async fn get_post_vec_by_satellite_id(
        satellite_id: i64,
        sphere_category_id: Option<i64>,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT p.* FROM posts p
                JOIN satellites s on s.satellite_id = p.satellite_id
                WHERE
                    s.satellite_id = $1 AND
                    p.category_id IS NOT DISTINCT FROM COALESCE($2, p.category_id) AND
                    p.moderator_id IS NULL
                ORDER BY p.is_pinned DESC, {} DESC
                LIMIT $3
                OFFSET $4",
                sort_type.to_order_by_code(),
            ).as_str(),
        )
            .bind(satellite_id)
            .bind(sphere_category_id)
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
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            format!(
                "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url
                FROM posts p
                JOIN spheres s on s.sphere_id = p.sphere_id
                LEFT JOIN sphere_categories c on c.category_id = p.category_id
                WHERE
                    p.moderator_id IS NULL AND
                    NOT p.is_nsfw AND
                    p.satellite_id IS NULL
                ORDER BY {} DESC
                LIMIT $1
                OFFSET $2",
                sort_type.to_order_by_code(),
            ).as_str()
        )
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        let post_vec = post_vec.into_iter().map(PostJoinCategory::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn get_subscribed_post_vec(
        user_id: i64,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            format!(
                "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url
                FROM posts p
                JOIN spheres s on s.sphere_id = p.sphere_id
                LEFT JOIN sphere_categories c on c.category_id = p.category_id
                WHERE
                    s.sphere_id IN (
                        SELECT sphere_id FROM sphere_subscriptions WHERE user_id = $1
                    ) AND
                    p.moderator_id IS NULL AND
                    p.satellite_id IS NULL
                ORDER BY {} DESC
                LIMIT $2
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

        let post_vec = post_vec.into_iter().map(PostJoinCategory::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn create_post(
        sphere_name: &str,
        satellite_id: Option<i64>,
        post_title: &str,
        post_body: &str,
        post_markdown_body: Option<&str>,
        is_spoiler: bool,
        is_nsfw: bool,
        is_pinned: bool,
        category_id: Option<i64>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        user.check_can_publish_on_sphere(sphere_name)?;
        if sphere_name.is_empty() || post_title.is_empty() {
            return Err(AppError::new(
                "Cannot create post without a valid sphere and title.",
            ));
        }
        if is_pinned {
            user.check_permissions(sphere_name, PermissionLevel::Moderate)?;
        }

        let post = sqlx::query_as!(
            Post,
            "INSERT INTO posts (
                title, body, markdown_body, is_nsfw, is_spoiler, category_id, sphere_id,
                sphere_name, satellite_id, is_pinned, creator_id, creator_name, is_creator_moderator
            )
             VALUES (
                $1, $2, $3,
                (
                    CASE
                        WHEN $4 THEN TRUE
                        ELSE (
                            (SELECT is_nsfw FROM spheres WHERE sphere_name = $7) OR
                            COALESCE(
                                (SELECT is_nsfw FROM satellites WHERE satellite_id = $8),
                                FALSE
                            )
                        )
                    END
                ),
                (
                    CASE
                        WHEN $5 THEN TRUE
                        ELSE COALESCE(
                            (SELECT is_spoiler FROM satellites WHERE satellite_id = $8),
                            FALSE
                        )
                    END
                ),
                $6,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $7),
                $7, $8, $9, $10, $11, $12
            ) RETURNING *",
            post_title,
            post_body,
            post_markdown_body,
            is_nsfw,
            is_spoiler,
            category_id,
            sphere_name,
            satellite_id,
            is_pinned,
            user.user_id,
            user.username,
            user.check_permissions(sphere_name, PermissionLevel::Moderate).is_ok(),
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
        is_spoiler: bool,
        is_nsfw: bool,
        is_pinned: bool,
        category_id: Option<i64>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        if post_title.is_empty() {
            return Err(AppError::new(
                "Cannot update post without a valid title.",
            ));
        }
        if is_pinned {
            let post = get_post_by_id(post_id, db_pool).await?;
            user.check_permissions(&post.sphere_name, PermissionLevel::Moderate)?;
        }

        let post = sqlx::query_as!(
            Post,
            "UPDATE posts SET
                title = $1,
                body = $2,
                markdown_body = $3,
                is_nsfw = (
                    CASE
                        WHEN $4 THEN TRUE
                        ELSE (
                            SELECT s.is_nsfw OR COALESCE(sa.is_nsfw, FALSE) FROM posts p
                            JOIN spheres s ON s.sphere_id = p.sphere_id
                            LEFT JOIN satellites sa ON sa.satellite_id = p.satellite_id
                            WHERE p.post_id = $8
                        )
                    END
                ),
                is_spoiler = (
                    CASE
                        WHEN $5 THEN TRUE
                        ELSE (
                            SELECT COALESCE(sa.is_spoiler, FALSE) FROM posts p
                            LEFT JOIN satellites sa ON sa.satellite_id = p.satellite_id
                            WHERE post_id = $8
                        )
                    END
                ),
                is_pinned = $6,
                category_id = $7,
                edit_timestamp = CURRENT_TIMESTAMP
            WHERE
                post_id = $8 AND
                creator_id = $9
            RETURNING *",
            post_title,
            post_body,
            post_markdown_body,
            is_nsfw,
            is_spoiler,
            is_pinned,
            category_id,
            post_id,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn increment_post_comment_count(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = sqlx::query_as!(
            Post,
            "UPDATE posts
            SET num_comments = num_comments + 1
            WHERE post_id = $1
            RETURNING *",
            post_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn update_post_scores(db_pool: &PgPool) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE posts
            SET scoring_timestamp = CURRENT_TIMESTAMP
            WHERE create_timestamp > (CURRENT_TIMESTAMP - INTERVAL '2 days')",
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::colors::Color;
        use crate::constants::{BEST_ORDER_BY_COLUMN, HOT_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN, TRENDING_ORDER_BY_COLUMN};
        use crate::post::ssr::PostJoinInfo;
        use crate::post::{Post, PostSortType};
        use crate::ranking::VoteValue;
        use crate::user::User;

        #[test]
        fn test_post_join_vote_into_post_with_info() {
            let user = User::default();
            let mut user_post = Post::default();
            user_post.creator_id = user.user_id;

            let user_post_without_vote = PostJoinInfo {
                post: user_post.clone(),
                category_name: None,
                category_color: None,
                vote_id: None,
                vote_post_id: None,
                vote_comment_id: None,
                vote_user_id: None,
                value: None,
                vote_timestamp: None,
            };
            let user_post_with_info = user_post_without_vote.into_post_with_info();
            assert_eq!(user_post_with_info.post, user_post);
            assert_eq!(user_post_with_info.sphere_category, None);
            assert_eq!(user_post_with_info.vote, None);

            let user_post_with_vote = PostJoinInfo {
                post: user_post.clone(),
                category_name: Some(String::from("a")),
                category_color: None,
                vote_id: Some(0),
                vote_post_id: Some(user_post.post_id),
                vote_comment_id: None,
                vote_user_id: Some(user.user_id),
                value: Some(1),
                vote_timestamp: Some(user_post.create_timestamp),
            };
            let user_post_with_info = user_post_with_vote.into_post_with_info();
            let user_vote = user_post_with_info.vote.expect("PostWithInfo should contain vote.");
            assert_eq!(user_post_with_info.post, user_post);
            assert_eq!(user_post_with_info.sphere_category, None);
            assert_eq!(user_vote.user_id, user.user_id);
            assert_eq!(user_vote.post_id, user_post.post_id);
            assert_eq!(user_vote.value, VoteValue::Up);
            assert_eq!(user_vote.comment_id, None);

            let mut other_post = Post::default();
            other_post.creator_id = user.user_id + 1;

            let other_post_with_vote = PostJoinInfo {
                post: other_post.clone(),
                category_name: Some(String::from("a")),
                category_color: Some(Color::Green),
                vote_id: Some(0),
                vote_post_id: Some(other_post.post_id),
                vote_comment_id: None,
                vote_user_id: Some(user.user_id),
                value: Some(-1),
                vote_timestamp: Some(other_post.create_timestamp),
            };
            let other_post_with_info = other_post_with_vote.into_post_with_info();
            let user_vote = other_post_with_info.vote.expect("PostWithInfo should contain vote.");
            let sphere_category = other_post_with_info.sphere_category.expect("PostWithInfo should contain category.");
            assert_eq!(other_post_with_info.post, other_post);
            assert_eq!(sphere_category.category_name, String::from("a"));
            assert_eq!(sphere_category.category_color, Color::Green);
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
pub async fn get_post_with_info_by_id(post_id: i64) -> Result<PostWithInfo, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = get_user().await?;
    Ok(ssr::get_post_with_info_by_id(post_id, user.as_ref(), &db_pool).await?)
}

#[server]
pub async fn get_sorted_post_vec(
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_sorted_post_vec(
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_subscribed_post_vec(
    user_id: i64,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_subscribed_post_vec(
        user_id,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_post_vec_by_sphere_name(
    sphere_name: String,
    sphere_category_id: Option<i64>,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let post_vec = ssr::get_post_vec_by_sphere_name(
        sphere_name.as_str(),
        sphere_category_id,
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
    sphere: String,
    satellite_id: Option<i64>,
    title: String,
    body: String,
    is_markdown: bool,
    is_spoiler: bool,
    is_nsfw: bool,
    is_pinned: Option<bool>,
    category_id: Option<i64>,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = get_html_and_markdown_bodies(body, is_markdown).await?;

    let post = ssr::create_post(
        sphere.as_str(),
        satellite_id,
        title.as_str(),
        body.as_str(),
        markdown_body.as_deref(),
        is_spoiler,
        is_nsfw,
        is_pinned.unwrap_or(false),
        category_id,
        &user,
        &db_pool,
    ).await?;

    let _vote = vote_on_content(VoteValue::Up, post.post_id, None, None, &user, &db_pool).await?;

    log::trace!("Created post with id: {}", post.post_id);
    let new_post_path: &str = &(SPHERE_ROUTE_PREFIX.to_owned()
        + PATH_SEPARATOR
        + sphere.as_str()
        + POST_ROUTE_PREFIX
        + PATH_SEPARATOR
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
    is_spoiler: bool,
    is_nsfw: bool,
    is_pinned: Option<bool>,
    category_id: Option<i64>,
) -> Result<Post, ServerFnError<AppError>> {
    log::trace!("Edit post {post_id}, title = {title}, body = {body}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = get_html_and_markdown_bodies(body, is_markdown).await?;

    let post = ssr::update_post(
        post_id,
        title.as_str(),
        body.as_str(),
        markdown_body.as_deref(),
        is_spoiler,
        is_nsfw,
        is_pinned.unwrap_or(false),
        category_id,
        &user,
        &db_pool,
    )
    .await?;

    log::trace!("Updated post with id: {}", post.post_id);
    Ok(post)
}

/// Get a memo returning the last valid post id from the url. Used to avoid triggering resources when leaving pages
pub fn get_post_id_memo(params: Memo<ParamsMap>) -> Memo<i64> {
    Memo::new(move |current_post_id: Option<&i64>| {
        if let Some(new_post_id_string) = params.read().get_str(POST_ROUTE_PARAM_NAME) {
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
    let sphere_state = expect_context::<SphereState>();
    let params = use_params_map();
    let post_id = get_post_id_memo(params);

    let post_resource = Resource::new(
        move || (post_id.get(), state.edit_post_action.version().get(), sphere_state.moderate_post_action.version().get()),
        move |(post_id, _, _)| {
            log::debug!("Load data for post: {post_id}");
            get_post_with_info_by_id(post_id)
        },
    );

    let comment_vec = RwSignal::new(Vec::<CommentWithChildren>::with_capacity(
        COMMENT_BATCH_SIZE as usize,
    ));
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let container_ref = NodeRef::<html::Div>::new();

    let _initial_comments_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            match get_post_comment_tree(
                post_id.get(),
                state.comment_sort_type.get(),
                0,
            ).await {
                Ok(ref mut init_comment_vec) => {
                    comment_vec.update(|comment_vec| {
                        std::mem::swap(comment_vec, init_comment_vec);
                    });
                    if let Some(list_ref) = container_ref.get_untracked() {
                        list_ref.set_scroll_top(0);
                    }
                },
                Err(ref e) => {
                    comment_vec.update(|comment_vec| comment_vec.clear());
                    load_error.set(Some(AppError::from(e)))
                },
            };
            is_loading.set(false);
        }
    );

    let _additional_comments_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = comment_vec.read_untracked().len();
                match get_post_comment_tree(
                    post_id.get(),
                    state.comment_sort_type.get_untracked(),
                    num_post,
                ).await {
                    Ok(ref mut additional_comment_vec) => {
                        comment_vec.update(|comment_vec| comment_vec.append(additional_comment_vec))
                    },
                    Err(ref e) => load_error.set(Some(AppError::from(e))),
                }
                is_loading.set(false);
            }
        }
    );

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
            <ArcTransitionUnpack resource=post_resource let:post_with_info>
                <div class="card">
                    <div class="card-body">
                        <div class="flex flex-col gap-2">
                            <h2 class="card-title">{post_with_info.post.title.clone()}</h2>
                            <PostBody post=post_with_info.post.clone()/>
                            <PostBadgeList
                                sphere_header=None
                                sphere_category=post_with_info.sphere_category.clone()
                                is_spoiler=post_with_info.post.is_spoiler
                                is_nsfw=post_with_info.post.is_nsfw
                            />
                            <PostWidgetBar post=post_with_info comment_vec/>
                        </div>
                    </div>
                </div>
            </ArcTransitionUnpack>
            <CommentSortWidget/>
            <CommentSection comment_vec/>
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
        </div>
    }.into_any()
}

/// Component to display a post's sphere, its category and whether it's a spoiler/NSFW
#[component]
pub fn PostBadgeList(
    sphere_header: Option<SphereHeader>,
    sphere_category: Option<SphereCategoryHeader>,
    is_spoiler: bool,
    is_nsfw: bool,
) -> impl IntoView {
    match (sphere_header, sphere_category, is_spoiler, is_nsfw) {
        (None, None, false, false) => None,
        (sphere_header, sphere_category, is_spoiler, is_nsfw) => Some(view! {
            <div class="flex gap-1 items-center">
            {
                sphere_header.map(|sphere_header| view! { <SphereHeader sphere_header/> })
            }
            {
                sphere_category.map(|category_header| view! { <SphereCategoryBadge category_header/> })
            }
            {
                match is_spoiler {
                    true => Some(view! { <div class="h-fit w-fit px-1 py-0.5 bg-black rounded-full"><SpoilerIcon/></div> }),
                    false => None
                }
            }
            {
                match is_nsfw {
                    true => Some(view! { <NsfwIcon/>}),
                    false => None
                }
            }
            </div>
        })
    }
}

/// Displays the body of a post
#[component]
pub fn PostBody(post: Post) -> impl IntoView {

    view! {
        <div class="pb-2">
        {
            match (&post.moderator_message, &post.infringed_rule_title) {
                (Some(moderator_message), Some(infringed_rule_title)) => view! { 
                    <ModeratedBody
                        infringed_rule_title=infringed_rule_title.clone()
                        moderator_message=moderator_message.clone()
                    />
                }.into_any(),
                _ => view! {
                    <ContentBody
                        body=post.body.clone()
                        is_markdown=post.markdown_body.is_some()
                    />
                }.into_any(),
            }
        }
        </div>
    }.into_any()
}

/// Component to encapsulate the widgets associated with each post
#[component]
fn PostWidgetBar(
    post: Arc<PostWithInfo>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
) -> impl IntoView {
    view! {
        <div class="flex gap-1 content-center">
            <VotePanel
                post_id=post.post.post_id
                comment_id=None
                score=post.post.score
                vote=post.vote.clone()
            />
            <CommentCountWidget count=post.post.num_comments/>
            <CommentButton post_id=post.post.post_id comment_vec/>
            <EditPostButton author_id=post.post.creator_id post=StoredValue::new(post.post.clone())/>
            <ModeratePostButton post_id=post.post.post_id/>
            <AuthorWidget author=post.post.creator_name.clone() is_moderator=post.post.is_creator_moderator/>
            <ModeratorWidget moderator=post.post.moderator_name.clone()/>
            <ModerationInfoButton content=Content::Post(post.post.clone())/>
            <TimeSinceWidget timestamp=post.post.create_timestamp/>
            <TimeSinceEditWidget edit_timestamp=post.post.edit_timestamp/>
        </div>
    }
}

/// Component to edit a post
#[component]
pub fn EditPostButton(
    post: StoredValue<Post>,
    author_id: i64
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let show_dialog = RwSignal::new(false);
    let show_button = move || match &(*state.user.read()) {
        Some(Ok(Some(user))) => user.user_id == author_id,
        _ => false,
    };
    let edit_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <Show when=show_button>
            <div>
                <button
                    class=edit_button_class
                    aria-expanded=move || show_dialog.get().to_string()
                    aria-haspopup="dialog"
                    on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
                >
                    <EditIcon/>
                </button>
                <EditPostDialog
                    post=post.get_value()
                    show_dialog
                />
            </div>
        </Show>
    }
}

/// Component to create a new post
#[component]
pub fn CreatePost() -> impl IntoView {
    let create_post_action = ServerAction::<CreatePost>::new();

    let query = use_query_map();
    let sphere_query = move || {
        query.read_untracked().get(CREATE_POST_SPHERE_QUERY_PARAM).unwrap_or_default()
    };

    let selected_sphere = RwSignal::new(None);
    let sphere_name_input = RwSignal::new(sphere_query());
    let sphere_name_debounced: Signal<String> = signal_debounced(sphere_name_input, 250.0);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(textarea_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref,
    };
    let is_title_empty = RwSignal::new(true);

    let matching_spheres_resource = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_prefix| get_matching_sphere_header_vec(sphere_prefix),
    );

    let sphere_categories_resource = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_name| get_sphere_category_vec(sphere_name)
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
                                name="sphere"
                                placeholder="Sphere"
                                autocomplete="off"
                                class="input input-bordered input-primary w-full h-input_m"
                                on:input=move |ev| {
                                    sphere_name_input.set(event_target_value(&ev).to_lowercase());
                                }
                                prop:value=sphere_name_input
                            />
                            <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-200 rounded-sm w-full">
                            <TransitionUnpack resource=matching_spheres_resource let:sphere_header_vec>
                            {
                                match sphere_header_vec.first() {
                                    Some(header) if header.sphere_name == sphere_name_input.get_untracked() => selected_sphere.set(Some(header.clone())),
                                    _ => selected_sphere.set(None)
                                };
                                sphere_header_vec.clone().into_iter().map(|sphere_header| {
                                    let sphere_name = sphere_header.sphere_name.clone();
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                on:click=move |_| sphere_name_input.set(sphere_name.clone())
                                            >
                                                <SphereHeader sphere_header/>
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
                            data=body_data
                        />
                        <LabeledFormCheckbox name="is_spoiler" label="Spoiler"/>
                        { move || {
                            match &*selected_sphere.read() {
                                Some(header) if header.is_nsfw => view! { <LabeledFormCheckbox name="is_nsfw" label="NSFW content" value=true disabled=true/> },
                                _ => view! { <LabeledFormCheckbox name="is_nsfw" label="NSFW content"/> },
                            }
                        }}
                        <IsPinnedCheckbox sphere_name=sphere_name_input/>
                        <SphereCategoryDropdown sphere_categories_resource name="category_id" show_inactive=false/>
                        <Transition>
                            <button type="submit" class="btn btn-active btn-secondary" disabled=move || match &*selected_sphere.read() {
                                Some(_) => {
                                    is_title_empty.get() ||
                                    body_data.content.read().is_empty()
                                },
                                _ => true,
                            }>
                                "Create"
                            </button>
                        </Transition>
                    </div>
            </ActionForm>
            <ActionError action=create_post_action.into()/>
        </div>
    }
}

/// Dialog to edit a post
#[component]
pub fn EditPostDialog(
    post: Post,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let post = StoredValue::new(post);
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <EditPostForm
                post
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to edit a post
#[component]
pub fn EditPostForm(
    post: StoredValue<Post>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let (current_body, is_markdown) = post.with_value(|post| match &post.markdown_body {
        Some(body) => (body.clone(), true),
        None => (post.body.clone(), false),
    });
    let (post_id, title, is_spoiler, is_nsfw, is_pinned) = post.with_value(|post| (
        post.post_id,
        post.title.clone(),
        post.is_spoiler,
        post.is_nsfw,
        post.is_pinned,
    ));
    let is_title_empty = RwSignal::new(false);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(textarea_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref,
    };
    body_data.set_content.set(current_body);
    let is_post_empty = Signal::derive(move || body_data.content.read().is_empty());

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
                        value=title
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
                        data=body_data
                        is_markdown
                    />
                    <LabeledFormCheckbox name="is_spoiler" label="Spoiler" value=is_spoiler/>
                    <LabeledFormCheckbox name="is_nsfw" label="NSFW content" value=is_nsfw/>
                    <IsPinnedCheckbox sphere_name=sphere_state.sphere_name value=is_pinned/>
                    <ModalFormButtons
                        disable_publish=is_post_empty
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError action=state.edit_post_action.into()/>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::colors::Color;
    use crate::constants::{BEST_STR, HOT_STR, RECENT_STR, TRENDING_STR};
    use crate::post::{Post, PostSortType, PostWithSphereInfo};
    use crate::sphere_category::SphereCategoryHeader;

    fn create_post_with_category(sphere_name: &str, title: &str, category_id: Option<i64>) -> Post {
        Post {
            post_id: 0,
            title: title.to_string(),
            body: String::default(),
            markdown_body: None,
            is_nsfw: false,
            is_spoiler: false,
            category_id,
            is_edited: false,
            sphere_id: 0,
            sphere_name: sphere_name.to_string(),
            satellite_id: None,
            creator_id: 0,
            creator_name: String::default(),
            is_creator_moderator: false,
            moderator_message: None,
            infringed_rule_id: None,
            infringed_rule_title: None,
            moderator_id: None,
            moderator_name: None,
            num_comments: 0,
            is_pinned: false,
            score: 0,
            score_minus: 0,
            recommended_score: 0.0,
            trending_score: 0.0,
            create_timestamp: Default::default(),
            edit_timestamp: None,
            scoring_timestamp: Default::default(),
        }
    }

    #[test]
    fn test_from_post() {
        let category_header_a = SphereCategoryHeader {
            category_name: String::from("a"),
            category_color: Color::Blue,
        };
        let category_header_b = SphereCategoryHeader {
            category_name: String::from("b"),
            category_color: Color::Red,
        };
        
        let post_1 = create_post_with_category("a", "i", Some(1));
        let post_2 = create_post_with_category("b", "j", Some(2));
        let post_3 = create_post_with_category("c", "k", Some(3));
        let post_4 = create_post_with_category("d", "l", None);
        
        let post_with_sphere_info_1 = PostWithSphereInfo::from_post(post_1.clone(), Some(category_header_a.clone()), None);
        let post_with_sphere_info_2 = PostWithSphereInfo::from_post(post_2.clone(), Some(category_header_b.clone()), None);
        let post_with_sphere_info_3 = PostWithSphereInfo::from_post(post_3.clone(), None, None);
        let post_with_sphere_info_4 = PostWithSphereInfo::from_post(post_4.clone(), None, None);
        
        assert_eq!(post_with_sphere_info_1.post, post_1);
        assert_eq!(post_with_sphere_info_1.sphere_category, Some(category_header_a));
        assert_eq!(post_with_sphere_info_1.sphere_icon_url, None);

        assert_eq!(post_with_sphere_info_2.post, post_2);
        assert_eq!(post_with_sphere_info_2.sphere_category, Some(category_header_b));
        assert_eq!(post_with_sphere_info_2.sphere_icon_url, None);

        assert_eq!(post_with_sphere_info_3.post, post_3);
        assert_eq!(post_with_sphere_info_3.sphere_category, None);
        assert_eq!(post_with_sphere_info_3.sphere_icon_url, None);

        assert_eq!(post_with_sphere_info_4.post, post_4);
        assert_eq!(post_with_sphere_info_4.sphere_category, None);
        assert_eq!(post_with_sphere_info_4.sphere_icon_url, None);
    }
    
    #[test]
    fn test_post_sort_type_display() {
        assert_eq!(PostSortType::Hot.to_string(), HOT_STR);
        assert_eq!(PostSortType::Trending.to_string(), TRENDING_STR);
        assert_eq!(PostSortType::Best.to_string(), BEST_STR);
        assert_eq!(PostSortType::Recent.to_string(), RECENT_STR);
    }
}
