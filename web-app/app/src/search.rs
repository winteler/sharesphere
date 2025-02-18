use std::collections::BTreeSet;
use leptos::server;
use server_fn::ServerFnError;
use crate::errors::AppError;
use crate::sphere::{SphereHeader};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    sphere::SPHERE_FETCH_LIMIT,
    user::USER_FETCH_LIMIT,
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::collections::BTreeSet;
    use sqlx::PgPool;
    use crate::comment::CommentWithContext;
    use crate::errors::AppError;
    use crate::post::PostWithSphereInfo;
    use crate::post::ssr::PostJoinCategory;
    use crate::sphere::SphereHeader;

    pub async fn get_matching_username_set(
        username_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<BTreeSet<String>, AppError> {
        let username_vec = sqlx::query!(
            "SELECT username FROM users WHERE username LIKE $1 ORDER BY username LIMIT $2",
            format!("{username_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        let mut username_set = BTreeSet::<String>::new();

        for row in username_vec {
            username_set.insert(row.username);
        }

        Ok(username_set)
    }

    pub async fn get_matching_sphere_header_vec(
        sphere_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT sphere_name, icon_url, is_nsfw
            FROM spheres
            WHERE sphere_name LIKE $1
            ORDER BY sphere_name LIMIT $2",
            format!("{sphere_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_header_vec)
    }

    pub async fn search_post(
        search_query: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url FROM (
                SELECT *, ts_rank(document, plainto_tsquery('simple', $1)) AS rank
                FROM posts
                WHERE document @@ plainto_tsquery('simple', $1)
                ORDER BY rank DESC
            ) p
            JOIN spheres s on s.sphere_id = p.sphere_id
            LEFT JOIN sphere_categories c on c.category_id = p.category_id"
        )
            .bind(search_query)
            .fetch_all(db_pool)
            .await?;

        let post_vec = post_vec.into_iter().map(PostJoinCategory::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn search_comment(
        search_query: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<CommentWithContext>, AppError> {
        let comment_vec = sqlx::query_as::<_, CommentWithContext>(
            "SELECT c.*, s.sphere_name, s.icon_url, s.is_nsfw, p.satellite_id, p.title as post_title FROM (
                SELECT *, ts_rank(document, plainto_tsquery('simple', $1)) AS rank
                FROM comments
                WHERE document @@ plainto_tsquery('simple', $1)
                ORDER BY rank DESC
            ) c
            JOIN posts p ON p.post_id = c.post_id
            JOIN spheres s ON s.sphere_id = p.sphere_id"
        )
            .bind(search_query)
            .fetch_all(db_pool)
            .await?;

        Ok(comment_vec)
    }
}

#[server]
pub async fn get_matching_username_set(
    username_prefix: String,
) -> Result<BTreeSet<String>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let username_set = ssr::get_matching_username_set(&username_prefix, USER_FETCH_LIMIT, &db_pool).await?;
    Ok(username_set)
}

#[server]
pub async fn get_matching_sphere_header_vec(
    sphere_prefix: String,
) -> Result<Vec<SphereHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_matching_sphere_header_vec(
        &sphere_prefix,
        SPHERE_FETCH_LIMIT,
        &db_pool
    ).await?;
    Ok(sphere_header_vec)
}