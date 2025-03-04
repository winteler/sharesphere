use std::collections::HashMap;
use std::fmt;

use const_format::concatcp;
use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use leptos_router::params::ParamsMap;
use leptos_use::{signal_debounced, use_textarea_autosize};
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState, PUBLISH_ROUTE};
use crate::comment::{CommentButtonWithCount, CommentSection, CommentWithChildren, COMMENT_BATCH_SIZE};
use crate::constants::{BEST_STR, HOT_STR, RECENT_STR, TRENDING_STR};
use crate::content::{Content, ContentBody};
use crate::editor::{FormMarkdownEditor, TextareaData};
use crate::embed::{Embed, EmbedPreview, EmbedType, Link, LinkType};
use crate::errors::AppError;
use crate::form::{IsPinnedCheckbox, LabeledFormCheckbox};
use crate::icons::{EditIcon};
use crate::moderation::{ModeratePostButton, ModeratedBody, ModerationInfoButton};
use crate::ranking::{ScoreIndicator, SortType, Vote, VotePanel};
use crate::satellite::SATELLITE_ROUTE_PREFIX;
use crate::search::get_matching_sphere_header_vec;
use crate::sphere::{SphereCategoryDropdown, SphereHeader, SphereHeaderLink, SphereState, SPHERE_ROUTE_PREFIX};
use crate::sphere_category::{get_sphere_category_vec, SphereCategory, SphereCategoryBadge, SphereCategoryHeader};
use crate::unpack::{ActionError, SuspenseUnpack, TransitionUnpack};
use crate::widget::{AuthorWidget, CommentCountWidget, LoadIndicators, ModalDialog, ModalFormButtons, ModeratorWidget, TagsWidget, TimeSinceEditWidget, TimeSinceWidget};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::{get_user, ssr::check_user},
    editor::ssr::get_html_and_markdown_bodies,
    embed::verify_link_and_get_embed,
    ranking::{ssr::vote_on_content, VoteValue},
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
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub link: Link,
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

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PostInheritedAttributes {
    pub is_spoiler: bool,
    pub is_nsfw: bool,
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
        let post = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts
            WHERE post_id = $1",
        )
            .bind(post_id)
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

    pub async fn get_post_inherited_attributes(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<PostInheritedAttributes, AppError> {
        let inherited_attributes = sqlx::query_as::<_, PostInheritedAttributes>(
            "SELECT
                COALESCE(sa.is_spoiler, FALSE) AS is_spoiler,
                COALESCE(sa.is_nsfw, s.is_nsfw) AS is_nsfw
            FROM posts p
            JOIN spheres s on s.sphere_id = p.sphere_id
            LEFT JOIN satellites sa on sa.satellite_id = p.satellite_id
            WHERE p.post_id = $1",
        )
            .bind(post_id)
            .fetch_one(db_pool)
            .await?;

        Ok(inherited_attributes)
    }

    pub async fn get_post_sphere(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as::<_, Sphere>(
            "SELECT s.*
            FROM spheres s
            JOIN posts p on p.sphere_id = s.sphere_id
            WHERE p.post_id = $1"
        )
            .bind(post_id)
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
        user: Option<&User>,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let posts_filters = user.map(|user| user.get_posts_filter()).unwrap_or_default();
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT p.* FROM posts p
                JOIN spheres s on s.sphere_id = p.sphere_id
                WHERE
                    s.sphere_name = $1 AND
                    p.category_id IS NOT DISTINCT FROM COALESCE($2, p.category_id) AND
                    p.moderator_id IS NULL AND
                    p.satellite_id IS NULL AND
                    (
                        $3 IS NULL OR NOT p.is_spoiler OR p.create_timestamp < CURRENT_TIMESTAMP - (INTERVAL '1 day' * $3)
                    ) AND
                    (
                        $4 OR NOT p.is_nsfw
                    )
                ORDER BY p.is_pinned DESC, {} DESC
                LIMIT $5
                OFFSET $6",
                sort_type.to_order_by_code(),
            ).as_str(),
        )
            .bind(sphere_name)
            .bind(sphere_category_id)
            .bind(posts_filters.days_hide_spoiler)
            .bind(posts_filters.show_nsfw)
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
        user: Option<&User>,
        db_pool: &PgPool,
    ) -> Result<Vec<Post>, AppError> {
        let posts_filters = user.map(|user| user.get_posts_filter()).unwrap_or_default();
        let post_vec = sqlx::query_as::<_, Post>(
            format!(
                "SELECT p.* FROM posts p
                JOIN satellites s on s.satellite_id = p.satellite_id
                WHERE
                    s.satellite_id = $1 AND
                    p.category_id IS NOT DISTINCT FROM COALESCE($2, p.category_id) AND
                    p.moderator_id IS NULL AND
                    (
                        $3 IS NULL OR NOT p.is_spoiler OR p.create_timestamp < CURRENT_TIMESTAMP - (INTERVAL '1 day' * $3)
                    ) AND
                    (
                        $4 OR NOT p.is_nsfw
                    )
                ORDER BY p.is_pinned DESC, {} DESC
                LIMIT $5
                OFFSET $6",
                sort_type.to_order_by_code(),
            ).as_str(),
        )
            .bind(satellite_id)
            .bind(sphere_category_id)
            .bind(posts_filters.days_hide_spoiler)
            .bind(posts_filters.show_nsfw)
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
        user: Option<&User>,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let (days_hide_spoiler, show_nsfw) = match user {
            Some(user) => (user.days_hide_spoiler, user.show_nsfw),
            None => (None, false),
        };
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            format!(
                "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url
                FROM posts p
                JOIN spheres s on s.sphere_id = p.sphere_id
                LEFT JOIN sphere_categories c on c.category_id = p.category_id
                WHERE
                    p.moderator_id IS NULL AND
                    p.satellite_id IS NULL AND
                    (
                        $1 IS NULL OR NOT p.is_spoiler OR p.create_timestamp < CURRENT_TIMESTAMP - (INTERVAL '1 day' * $1)
                    ) AND
                    (
                        $2 OR NOT p.is_nsfw
                    )
                ORDER BY {} DESC
                LIMIT $3
                OFFSET $4",
                sort_type.to_order_by_code(),
            ).as_str()
        )
            .bind(days_hide_spoiler)
            .bind(show_nsfw)
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
        user: Option<&User>,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let posts_filters = user.map(|user| user.get_posts_filter()).unwrap_or_default();
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
                    p.satellite_id IS NULL AND
                    (
                        $2 IS NULL OR NOT p.is_spoiler OR p.create_timestamp < CURRENT_TIMESTAMP - (INTERVAL '1 day' * $2)
                    ) AND
                    (
                        $3 OR NOT p.is_nsfw
                    )
                ORDER BY {} DESC
                LIMIT $4
                OFFSET $5",
                sort_type.to_order_by_code(),
            )
            .as_str(),
        )
            .bind(user_id)
            .bind(posts_filters.days_hide_spoiler)
            .bind(posts_filters.show_nsfw)
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
        link: Link,
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

        let post = sqlx::query_as::<_, Post>(
            "INSERT INTO posts (
                title, body, markdown_body, link_type, link_url, link_embed, link_thumbnail_url, is_nsfw, is_spoiler, category_id, sphere_id,
                sphere_name, satellite_id, is_pinned, creator_id, creator_name, is_creator_moderator
            )
             VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                (
                    CASE
                        WHEN $8 THEN TRUE
                        ELSE (
                            (SELECT is_nsfw FROM spheres WHERE sphere_name = $11) OR
                            COALESCE(
                                (SELECT is_nsfw FROM satellites WHERE satellite_id = $12),
                                FALSE
                            )
                        )
                    END
                ),
                (
                    CASE
                        WHEN $9 THEN TRUE
                        ELSE COALESCE(
                            (SELECT is_spoiler FROM satellites WHERE satellite_id = $12),
                            FALSE
                        )
                    END
                ),
                $10,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $11),
                $11, $12, $13, $14, $15, $16
            ) RETURNING *",
        )
            .bind(post_title)
            .bind(post_body)
            .bind(post_markdown_body)
            .bind(link.link_type as i16)
            .bind(link.link_url)
            .bind(link.link_embed)
            .bind(link.link_thumbnail_url)
            .bind(is_nsfw)
            .bind(is_spoiler)
            .bind(category_id)
            .bind(sphere_name)
            .bind(satellite_id)
            .bind(is_pinned)
            .bind(user.user_id)
            .bind(user.username.clone())
            .bind(user.check_permissions(sphere_name, PermissionLevel::Moderate).is_ok())
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn update_post(
        post_id: i64,
        post_title: &str,
        post_body: &str,
        post_markdown_body: Option<&str>,
        link: Link,
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

        let post = sqlx::query_as::<_, Post>(
            "UPDATE posts SET
                title = $1,
                body = $2,
                markdown_body = $3,
                link_type = $4,
                link_url = $5,
                link_embed = $6,
                link_thumbnail_url = $7,
                is_nsfw = (
                    CASE
                        WHEN $8 THEN TRUE
                        ELSE (
                            SELECT s.is_nsfw OR COALESCE(sa.is_nsfw, FALSE) FROM posts p
                            JOIN spheres s ON s.sphere_id = p.sphere_id
                            LEFT JOIN satellites sa ON sa.satellite_id = p.satellite_id
                            WHERE p.post_id = $12
                        )
                    END
                ),
                is_spoiler = (
                    CASE
                        WHEN $9 THEN TRUE
                        ELSE (
                            SELECT COALESCE(sa.is_spoiler, FALSE) FROM posts p
                            LEFT JOIN satellites sa ON sa.satellite_id = p.satellite_id
                            WHERE post_id = $12
                        )
                    END
                ),
                is_pinned = $10,
                category_id = $11,
                edit_timestamp = CURRENT_TIMESTAMP
            WHERE
                post_id = $12 AND
                creator_id = $13
            RETURNING *",
        )
            .bind(post_title)
            .bind(post_body)
            .bind(post_markdown_body)
            .bind(link.link_type as i16)
            .bind(link.link_url)
            .bind(link.link_embed)
            .bind(link.link_thumbnail_url)
            .bind(is_nsfw)
            .bind(is_spoiler)
            .bind(is_pinned)
            .bind(category_id)
            .bind(post_id)
            .bind(user.user_id)
            .fetch_one(db_pool)
            .await?;

        Ok(post)
    }

    pub async fn increment_post_comment_count(
        post_id: i64,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = sqlx::query_as::<_, Post>(
            "UPDATE posts
            SET num_comments = num_comments + 1
            WHERE post_id = $1
            RETURNING *",
        )
            .bind(post_id)
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
pub async fn get_post_inherited_attributes(post_id: i64) -> Result<PostInheritedAttributes, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    Ok(ssr::get_post_inherited_attributes(post_id, &db_pool).await?)
}

#[server]
pub async fn get_sorted_post_vec(
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, ServerFnError<AppError>> {
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_sorted_post_vec(
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
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
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_subscribed_post_vec(
        user_id,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
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
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;
    let post_vec = ssr::get_post_vec_by_sphere_name(
        sphere_name.as_str(),
        sphere_category_id,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
        &db_pool,
    ).await?;
    Ok(post_vec)
}

#[server]
pub async fn get_post_vec_by_satellite_id(
    satellite_id: i64,
    sphere_category_id: Option<i64>,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, ServerFnError<AppError>> {
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;
    let post_vec = ssr::get_post_vec_by_satellite_id(
        satellite_id,
        sphere_category_id,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
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
    embed_type: EmbedType,
    link: Option<String>,
    is_markdown: bool,
    is_spoiler: bool,
    is_nsfw: bool,
    is_pinned: Option<bool>,
    category_id: Option<i64>,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = get_html_and_markdown_bodies(body, is_markdown).await?;

    let (link, _) = match (embed_type, link) {
        (embed_type, Some(link)) if embed_type != EmbedType::None => verify_link_and_get_embed(embed_type, &link).await,
        _ => (Link::default(), None),
    };

    let post = ssr::create_post(
        sphere.as_str(),
        satellite_id,
        title.as_str(),
        body.as_str(),
        markdown_body.as_deref(),
        link,
        is_spoiler,
        is_nsfw,
        is_pinned.unwrap_or(false),
        category_id,
        &user,
        &db_pool,
    ).await?;

    let _vote = vote_on_content(VoteValue::Up, post.post_id, None, None, &user, &db_pool).await?;

    log::trace!("Created post with id: {}", post.post_id);
    let new_post_path = get_post_path(&sphere, satellite_id, post.post_id);

    leptos_axum::redirect(new_post_path.as_str());
    Ok(())
}

#[server]
pub async fn edit_post(
    post_id: i64,
    title: String,
    body: String,
    embed_type: EmbedType,
    link: Option<String>,
    is_markdown: bool,
    is_spoiler: bool,
    is_nsfw: bool,
    is_pinned: Option<bool>,
    category_id: Option<i64>,
) -> Result<Post, ServerFnError<AppError>> {
    log::trace!("Edit post {post_id}, title = {title}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = get_html_and_markdown_bodies(body, is_markdown).await?;

    let (link, _) = match (embed_type, link) {
        (embed_type, Some(link)) if embed_type != EmbedType::None => verify_link_and_get_embed(embed_type, &link).await,
        _ => (Link::default(), None),
    };

    let post = ssr::update_post(
        post_id,
        title.as_str(),
        body.as_str(),
        markdown_body.as_deref(),
        link,
        is_spoiler,
        is_nsfw,
        is_pinned.unwrap_or(false),
        category_id,
        &user,
        &db_pool,
    ).await?;

    log::trace!("Updated post with id: {}", post.post_id);
    Ok(post)
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
    let is_loading = RwSignal::new(false);
    let additional_load_count = RwSignal::new(0);
    let container_ref = NodeRef::<html::Div>::new();

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
                <div class="card">
                    <div class="card-body">
                        <div class="flex flex-col gap-2">
                            <h2 class="card-title">{post_with_info.post.title.clone()}</h2>
                            <PostBody post=post_with_info.post.clone()/>
                            <Embed link=post_with_info.post.link.clone()/>
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
            </TransitionUnpack>
            <CommentSection post_id comment_vec is_loading additional_load_count/>
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
            <div class="flex gap-2 items-center">
            {
                sphere_header.map(|sphere_header| view! { <SphereHeaderLink sphere_header/> })
            }
            {
                sphere_category.map(|category_header| view! { <SphereCategoryBadge category_header/> })
            }
            <TagsWidget is_spoiler is_nsfw/>
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
fn PostWidgetBar<'a>(
    post: &'a PostWithInfo,
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
            <CommentButtonWithCount post_id=post.post.post_id comment_vec count=post.post.num_comments/>
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

/// Component to display a vector of sphere posts and indicate when more need to be loaded
#[component]
pub fn PostMiniatureList(
    /// signal containing the posts to display
    #[prop(into)]
    post_vec: Signal<Vec<PostWithSphereInfo>>,
    /// signal indicating new posts are being loaded
    #[prop(into)]
    is_loading: Signal<bool>,
    /// signal containing an eventual loading error in order to display it
    #[prop(into)]
    load_error: Signal<Option<AppError>>,
    /// signal to request loading additional posts
    additional_load_count: RwSignal<i64>,
    /// reference to the container of the posts in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
    #[prop(default = true)]
    show_sphere_header: bool,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col overflow-y-auto w-full pr-2 divide-y divide-base-content/20"
            on:scroll=move |_| match list_ref.get() {
                Some(node_ref) => {
                    if node_ref.scroll_top() + node_ref.offset_height() >= node_ref.scroll_height() && !is_loading.get_untracked() {
                        additional_load_count.update(|value| *value += 1);
                    }
                },
                None => log::error!("Post container 'ul' node failed to load."),
            }
            node_ref=list_ref
        >
            <For
                each= move || post_vec.get().into_iter()
                key=|post| post.post.post_id
                children=move |post_info| {
                    let post = post_info.post;
                    let sphere_header = match show_sphere_header {
                        true => Some(SphereHeader::new(post.sphere_name.clone(), post_info.sphere_icon_url, false)),
                        false => None,
                    };
                    let post_path = get_post_path(&post.sphere_name, post.satellite_id, post.post_id);
                    view! {
                        <li>
                            <a href=post_path>
                                <div class="flex flex-col gap-1 pl-1 pt-1 pb-2 my-1 rounded hover:bg-base-content/20">
                                    <h2 class="card-title pl-1">{post.title.clone()}</h2>
                                    <PostBadgeList
                                        sphere_header
                                        sphere_category=post_info.sphere_category
                                        is_spoiler=post.is_spoiler
                                        is_nsfw=post.is_nsfw
                                    />
                                    <div class="flex gap-1">
                                        <ScoreIndicator score=post.score/>
                                        <CommentCountWidget count=post.num_comments/>
                                        <AuthorWidget author=post.creator_name.clone() is_moderator=post.is_creator_moderator/>
                                        <TimeSinceWidget timestamp=post.create_timestamp/>
                                    </div>
                                </div>
                            </a>
                        </li>
                    }
                }
            />
            <LoadIndicators load_error is_loading/>
        </ul>
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
pub fn PostForm(
    title_input: RwSignal<String>,
    body_data: TextareaData,
    embed_type_input: RwSignal<EmbedType>,
    link_input: RwSignal<String>,
    #[prop(into)]
    sphere_name: Signal<String>,
    #[prop(into)]
    is_parent_spoiler: Signal<bool>,
    #[prop(into)]
    is_parent_nsfw: Signal<bool>,
    category_vec_resource: Resource<Result<Vec<SphereCategory>, ServerFnError<AppError>>>,
    #[prop(default = None)]
    current_post: Option<StoredValue<Post>>,
) -> impl IntoView {
    let (is_markdown, is_spoiler, is_nsfw, is_pinned, category_id) = match current_post {
        Some(post) => post.with_value(|post| {
            let (current_body, is_markdown) = match &post.markdown_body {
                Some(body) => (body.clone(), true),
                None => (post.body.clone(), false),
            };
            body_data.set_content.set(current_body);
            (is_markdown, post.is_spoiler, post.is_nsfw, post.is_pinned, post.category_id)
        }),
        None => (false, false, false, false, None),
    };

    view! {
        <input
            type="text"
            name="title"
            placeholder="Title"
            class="input input-bordered input-primary h-input_m"
            value=title_input
            autofocus
            autocomplete="off"
            on:input=move |ev| title_input.set(event_target_value(&ev))
        />
        <FormMarkdownEditor
            name="body"
            is_markdown_name="is_markdown"
            placeholder="Content"
            data=body_data
            is_markdown
        />
        <LinkForm link_input embed_type_input title_input/>
        { move || {
            match is_parent_spoiler.get() {
                true => view! { <LabeledFormCheckbox name="is_spoiler" label="Spoiler" value=true disabled=true/> },
                false => view! { <LabeledFormCheckbox name="is_spoiler" label="Spoiler" value=is_spoiler/> },
            }
        }}
        { move || {
            match is_parent_nsfw.get() {
                true => view! { <LabeledFormCheckbox name="is_nsfw" label="NSFW content" value=true disabled=true/> },
                false => view! { <LabeledFormCheckbox name="is_nsfw" label="NSFW content" value=is_nsfw/> },
            }
        }}
        <IsPinnedCheckbox sphere_name value=is_pinned/>
        <SphereCategoryDropdown category_vec_resource init_category_id=category_id name="category_id" show_inactive=false/>
    }
}

/// Component to give a link to external content
#[component]
pub fn LinkForm(
    embed_type_input: RwSignal<EmbedType>,
    link_input: RwSignal<String>,
    title_input: RwSignal<String>,
) -> impl IntoView {
    let select_ref = NodeRef::<html::Select>::new();
    let input_ref = NodeRef::<html::Input>::new();
    view! {
        <div class="w-full flex flex-col gap-2">
            <div class="h-full flex gap-2 items-center">
                <span class="label-text w-fit">"Link"</span>
                <select
                    name="embed_type"
                    class="select select-bordered h-input_m w-fit"
                    node_ref=select_ref
                >
                    <option
                        selected=move || embed_type_input.get_untracked() == EmbedType::None
                        on:click=move |_| {
                            embed_type_input.set(EmbedType::None);
                            link_input.set(String::default());
                            if let Some(link_input_ref) = input_ref.get_untracked() {
                                link_input_ref.set_value("");
                            }
                        }
                    >
                        "None"
                    </option>
                    <option
                        selected=move || embed_type_input.get_untracked() == EmbedType::Link
                        on:click=move |_| embed_type_input.set(EmbedType::Link)
                    >
                        "Link"
                    </option>
                    <option
                        selected=move || embed_type_input.get_untracked() == EmbedType::Embed
                        on:click=move |_| embed_type_input.set(EmbedType::Embed)
                    >
                        "Embed"
                    </option>
                </select>
                <input
                    type="text"
                    name="link"
                    placeholder="Link"
                    class="input input-bordered input-primary h-input_m grow"
                    value=link_input
                    autofocus
                    autocomplete="off"
                    on:input=move |ev| {
                        link_input.set(event_target_value(&ev));
                    }
                    node_ref=input_ref
                />
            </div>
            <EmbedPreview embed_type_input link_input title_input select_ref/>
        </div>
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

    let is_sphere_selected = RwSignal::new(false);
    let is_sphere_nsfw = RwSignal::new(false);
    let sphere_name_input = RwSignal::new(sphere_query());
    let sphere_name_debounced: Signal<String> = signal_debounced(sphere_name_input, 250.0);

    let title_input = RwSignal::new(String::default());
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(textarea_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref,
    };
    let link_input = RwSignal::new(String::default());
    let embed_type_input = RwSignal::new(EmbedType::None);

    let matching_spheres_resource = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_prefix| get_matching_sphere_header_vec(sphere_prefix),
    );

    let category_vec_resource = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_name| get_sphere_category_vec(sphere_name)
    );

    view! {
        <div class="w-4/5 2xl:w-2/5 p-2 mx-auto flex flex-col gap-2 overflow-auto">
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
                                    Some(header) if header.sphere_name == sphere_name_input.get_untracked() => {
                                        is_sphere_nsfw.set(header.is_nsfw);
                                        is_sphere_selected.set(true);
                                    },
                                    _ => {
                                        is_sphere_selected.set(true);
                                        is_sphere_nsfw.set(false);
                                    }
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
                    <PostForm
                        title_input
                        body_data
                        embed_type_input
                        link_input
                        sphere_name=sphere_name_input
                        is_parent_spoiler=false
                        is_parent_nsfw=is_sphere_nsfw
                        category_vec_resource
                    />
                    <button type="submit" class="btn btn-active btn-secondary" disabled=move || {
                        !is_sphere_selected.get() ||
                        title_input.read().is_empty() ||
                        (
                            body_data.content.read().is_empty() &&
                            *embed_type_input.read() == EmbedType::None
                        ) || (
                            *embed_type_input.read() != EmbedType::None &&
                            link_input.read().is_empty() // TODO check valid url?
                        )
                    }>
                        "Submit"
                    </button>
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
            class="w-full flex justify-center"
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

    let (post_id, title, link_type, link_url) = post.with_value(|post| (
        post.post_id,
        post.title.clone(),
        post.link.link_type,
        post.link.link_url.clone(),
    ));
    let title_input = RwSignal::new(title);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(textarea_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref,
    };
    let embed_type_input = RwSignal::new(match link_type {
        LinkType::None => EmbedType::None,
        LinkType::Link => EmbedType::Link,
        _ => EmbedType::Embed,
    });
    let link_input = RwSignal::new(link_url.unwrap_or_default());
    let disable_publish = Signal::derive(move || {
        title_input.read().is_empty() ||
        (
            body_data.content.read().is_empty() &&
            embed_type_input.read() == EmbedType::None
        ) || (
            embed_type_input.read() != EmbedType::None &&
            link_input.read().is_empty()
        )
    });

    let inherited_attributes_resource = Resource::new(
        move || (),
        move |_| get_post_inherited_attributes(post_id)
    );

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3 w-4/5 2xl:w-2/5">
            <div class="text-center font-bold text-2xl">"Edit your post"</div>
            <ActionForm action=state.edit_post_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="post_id"
                        class="hidden"
                        value=post_id
                    />
                    <SuspenseUnpack resource=inherited_attributes_resource let:inherited_post_attr>
                        <PostForm
                            title_input
                            body_data
                            embed_type_input
                            link_input
                            sphere_name=sphere_state.sphere_name
                            is_parent_spoiler=inherited_post_attr.is_spoiler
                            is_parent_nsfw=inherited_post_attr.is_nsfw
                            category_vec_resource=sphere_state.sphere_categories_resource
                            current_post=Some(post)
                        />
                    </SuspenseUnpack>
                    <ModalFormButtons
                        disable_publish
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError action=state.edit_post_action.into()/>
        </div>
    }
}

/// # Returns the path to a post given its id, sphere and optional satellite
///
/// ```
/// use app::post::get_post_path;
///
/// assert_eq!(get_post_path("test", None, 1), "/spheres/test/posts/1");
/// assert_eq!(get_post_path("test", Some(1), 1), "/spheres/test/satellites/1/posts/1");
/// ```
pub fn get_post_path(
    sphere_name: &str,
    satellite_id: Option<i64>,
    post_id: i64,
) -> String {
    match satellite_id {
        Some(satellite_id) => format!(
            "{SPHERE_ROUTE_PREFIX}/{sphere_name}{SATELLITE_ROUTE_PREFIX}/{satellite_id}{POST_ROUTE_PREFIX}/{}",
            post_id
        ),
        None => format!("{SPHERE_ROUTE_PREFIX}/{sphere_name}{POST_ROUTE_PREFIX}/{}", post_id)
    }
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

pub fn add_sphere_info_to_post_vec(
    post_vec: Vec<Post>, 
    sphere_category_map: HashMap<i64, SphereCategoryHeader>,
    sphere_icon_url: Option<String>,
) -> Vec<PostWithSphereInfo> {
    post_vec.into_iter().map(|post| {
        let category_id = match post.category_id {
            Some(category_id) => sphere_category_map.get(&category_id).cloned(),
            None => None,
        };
        PostWithSphereInfo::from_post(post, category_id, sphere_icon_url.clone())
    }).collect()
}

#[cfg(test)]
mod tests {
    use crate::colors::Color;
    use crate::constants::{BEST_STR, HOT_STR, RECENT_STR, TRENDING_STR};
    use crate::embed::{Link};
    use crate::post::{Post, PostSortType, PostWithSphereInfo};
    use crate::sphere_category::SphereCategoryHeader;

    fn create_post_with_category(sphere_name: &str, title: &str, category_id: Option<i64>) -> Post {
        Post {
            post_id: 0,
            title: title.to_string(),
            body: String::default(),
            markdown_body: None,
            link: Link::default(),
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

    #[test]
    fn test_add_sphere_info_to_post_vec() {
        // TODO
    }
}
