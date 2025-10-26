use leptos::prelude::*;
use sharesphere_utils::errors::AppError;
use crate::comment::{CommentWithContext};
use crate::post::{PostWithSphereInfo};
use crate::ranking::SortType;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::session::ssr::get_db_pool,
    sharesphere_utils::checks::check_username,
    crate::comment::{COMMENT_BATCH_SIZE},
    crate::post::{POST_BATCH_SIZE},
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_utils::errors::AppError;
    use crate::comment::CommentWithContext;
    use crate::post::PostWithSphereInfo;
    use crate::post::ssr::PostJoinSphereInfo;
    use crate::ranking::SortType;

    pub async fn get_user_post_vec(
        username: &str,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinSphereInfo>(
            format!(
                "SELECT
                    p.*,
                    u.username as creator_name,
                    c.category_name,
                    c.category_color,
                    s.icon_url as sphere_icon_url,
                    s.sphere_name
                FROM posts p
                JOIN users u ON u.user_id = p.creator_id
                JOIN spheres s ON s.sphere_id = p.sphere_id
                LEFT JOIN sphere_categories c ON c.category_id = p.category_id
                WHERE
                    u.username = $1 AND
                    p.moderator_id IS NULL AND
                    p.delete_timestamp IS NULL
                ORDER BY {} DESC
                LIMIT $2
                OFFSET $3",
                sort_type.to_order_by_code(),
            ).as_str()
        )
            .bind(username)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        let post_vec = post_vec.into_iter().map(PostJoinSphereInfo::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn get_user_comment_vec(
        username: &str,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<CommentWithContext>, AppError> {
        let comment_vec = sqlx::query_as::<_, CommentWithContext>(
            format!(
                "SELECT
                    c.*,
                    u.username as creator_name,
                    s.sphere_name,
                    s.icon_url,
                    s.is_nsfw,
                    p.satellite_id,
                    p.title as post_title
                FROM comments c
                JOIN users u ON u.user_id = c.creator_id
                JOIN posts p ON p.post_id = c.post_id
                JOIN spheres s ON s.sphere_id = p.sphere_id
                WHERE
                    u.username = $1 AND
                    c.moderator_id IS NULL AND
                    c.delete_timestamp IS NULL
                ORDER BY {} DESC
                LIMIT $2
                OFFSET $3",
                sort_type.to_order_by_code(),
            ).as_str()
        )
            .bind(username)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(comment_vec)
    }
}

#[server]
pub async fn get_user_post_vec(
    username: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, AppError> {
    check_username(&username, false)?;
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_user_post_vec(
        &username,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_user_comment_vec(
    username: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithContext>, AppError> {
    check_username(&username, false)?;
    let db_pool = get_db_pool()?;

    let comment_vec = ssr::get_user_comment_vec(
        &username,
        sort_type,
        COMMENT_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(comment_vec)
}