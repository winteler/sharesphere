use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
    sharesphere_utils::checks::{check_satellite_name, check_sphere_name, check_string_length},
    sharesphere_utils::constants::MAX_CONTENT_LENGTH,
    sharesphere_utils::editor::ssr::get_html_and_markdown_strings,
    crate::satellite::ssr::get_active_satellite_vec_by_sphere_name,
};
use crate::ranking::SortType;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Satellite {
    pub satellite_id: i64,
    pub satellite_name: String,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_nsfw: bool,
    pub is_spoiler: bool,
    pub num_posts: i32,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub disable_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone)]
pub struct SatelliteState {
    pub satellite_id: Memo<i64>,
    pub sort_type: RwSignal<SortType>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub satellite_resource: Resource<Result<Satellite, AppError>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::user::User;
    use sharesphere_utils::errors::AppError;
    use crate::satellite::Satellite;
    use crate::sphere::Sphere;

    pub async fn get_satellite_by_id(satellite_id: i64, db_pool: &PgPool) -> Result<Satellite, AppError> {
        let satellite = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE satellite_id = $1",
            satellite_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }

    pub async fn get_active_satellite_vec_by_sphere_name(sphere_name: &str, db_pool: &PgPool) -> Result<Vec<Satellite>, AppError> {
        let satellite_vec = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE
                sphere_name = $1 AND
                disable_timestamp IS NULL
            ORDER BY satellite_name",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(satellite_vec)
    }

    pub async fn get_satellite_vec_by_sphere_name(sphere_name: &str, db_pool: &PgPool) -> Result<Vec<Satellite>, AppError> {
        let satellite_vec = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE sphere_name = $1
            ORDER BY satellite_name",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(satellite_vec)
    }

    pub async fn get_satellite_sphere(satellite_id: i64, db_pool: &PgPool) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as::<_, Sphere>(
            "SELECT s.* FROM spheres s
            JOIN satellites sa ON sa.sphere_id = s.sphere_id
            WHERE sa.satellite_id = $1"
        )
            .bind(satellite_id)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn create_satellite(
        sphere_name: &str,
        satellite_name: &str,
        body: &str,
        markdown_body: Option<&str>,
        is_nsfw: bool,
        is_spoiler: bool,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;

        let satellite = sqlx::query_as!(
            Satellite,
            "INSERT INTO satellites
            (satellite_name, sphere_id, sphere_name, body, markdown_body, is_nsfw, is_spoiler, creator_id)
            VALUES (
                $1,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                $2, $3, $4,
                (
                    CASE
                        WHEN $5 THEN TRUE
                        ELSE (SELECT is_nsfw FROM spheres WHERE sphere_name = $2)
                    END
                ),
                $6, $7
            )
            RETURNING *",
            satellite_name,
            sphere_name,
            body,
            markdown_body,
            is_nsfw,
            is_spoiler,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }

    pub async fn update_satellite(
        satellite_id: i64,
        satellite_name: &str,
        body: &str,
        markdown_body: Option<&str>,
        is_nsfw: bool,
        is_spoiler: bool,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        let sphere = get_satellite_sphere(satellite_id, db_pool).await?;
        user.check_permissions(&sphere.sphere_name, PermissionLevel::Manage)?;

        let satellite = sqlx::query_as!(
            Satellite,
            "UPDATE satellites
            SET
                satellite_name = $1,
                body = $2,
                markdown_body = $3,
                is_nsfw = $4,
                is_spoiler = $5
            WHERE satellite_id = $6
            RETURNING *",
            satellite_name,
            body,
            markdown_body,
            is_nsfw || sphere.is_nsfw,
            is_spoiler,
            satellite_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }

    pub async fn disable_satellite(
        satellite_id: i64,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        let sphere = get_satellite_sphere(satellite_id, db_pool).await?;
        user.check_permissions(&sphere.sphere_name, PermissionLevel::Manage)?;

        let satellite = sqlx::query_as!(
            Satellite,
            "UPDATE satellites
            SET disable_timestamp = NOW()
            WHERE satellite_id = $1
            RETURNING *",
            satellite_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }
}

#[server]
pub async fn get_satellite_by_id(
    satellite_id: i64,
) -> Result<Satellite, AppError> {
    let db_pool = get_db_pool()?;
    let satellite = ssr::get_satellite_by_id(satellite_id, &db_pool).await?;
    Ok(satellite)
}

#[server]
pub async fn get_satellite_vec_by_sphere_name(
    sphere_name: String,
    only_active: bool,
) -> Result<Vec<Satellite>, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let satellite_vec = match only_active {
        true => get_active_satellite_vec_by_sphere_name(&sphere_name, &db_pool).await?,
        false => ssr::get_satellite_vec_by_sphere_name(&sphere_name, &db_pool).await?,
    };
    Ok(satellite_vec)
}

#[server]
pub async fn create_satellite(
    sphere_name: String,
    satellite_name: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> Result<Satellite, AppError> {
    check_sphere_name(&sphere_name)?;
    check_satellite_name(&satellite_name)?;
    check_string_length(&body, "Satellite body", MAX_CONTENT_LENGTH as usize, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let (body, markdown_body) = get_html_and_markdown_strings(body, is_markdown).await?;

    let satellite = ssr::create_satellite(
        &sphere_name,
        &satellite_name,
        &body,
        markdown_body.as_deref(),
        is_nsfw,
        is_spoiler,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

#[server]
pub async fn update_satellite(
    satellite_id: i64,
    satellite_name: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> Result<Satellite, AppError> {
    check_satellite_name(&satellite_name)?;
    check_string_length(&body, "Satellite body", MAX_CONTENT_LENGTH as usize, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let (body, markdown_body) = get_html_and_markdown_strings(body, is_markdown).await?;

    let satellite = ssr::update_satellite(
        satellite_id,
        &satellite_name,
        &body,
        markdown_body.as_deref(),
        is_nsfw,
        is_spoiler,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

#[server]
pub async fn disable_satellite(
    satellite_id: i64,
) -> Result<Satellite, AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let satellite = ssr::disable_satellite(
        satellite_id,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}