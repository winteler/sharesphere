use leptos::server;
use serde::{Deserialize, Serialize};
use server_fn::ServerFnError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::{check_user},
        session::ssr::get_db_pool,
    },
};

use sharesphere_utils::colors::Color;
use sharesphere_utils::errors::AppError;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategory {
    pub category_id: i64,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub category_name: String,
    pub category_color: Color,
    pub description: String,
    pub is_active: bool,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::user::User;
    use sharesphere_utils::colors::Color;
    use sharesphere_utils::errors::AppError;
    use crate::sphere_category::SphereCategory;

    pub const CATEGORY_NOT_DELETED_STR: &str = "Category was not deleted, it either doesn't exist or is used.";

    pub async fn set_sphere_category(
        sphere_name: &str,
        category_name: &str,
        category_color: Color,
        description: &str,
        is_active: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<SphereCategory, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;

        let category = sqlx::query_as!(
                SphereCategory,
                "INSERT INTO sphere_categories
                (sphere_id, sphere_name, category_name, category_color, description, is_active, creator_id)
                VALUES (
                    (SELECT sphere_id FROM spheres WHERE sphere_name = $1),
                    $1, $2, $3, $4, $5, $6
                ) ON CONFLICT (sphere_id, category_name) DO UPDATE
                    SET description = EXCLUDED.description,
                        category_color = EXCLUDED.category_color,
                        is_active = EXCLUDED.is_active,
                        timestamp = CURRENT_TIMESTAMP
                RETURNING *",
                sphere_name,
                category_name,
                category_color as i32,
                description,
                is_active,
                user.user_id,
            )
            .fetch_one(db_pool)
            .await?;

        Ok(category)
    }

    pub async fn delete_sphere_category(
        sphere_name: &str,
        category_name: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;

        let result = sqlx::query!(
            "DELETE FROM sphere_categories c
             WHERE sphere_name = $1 AND category_name = $2 AND NOT EXISTS (
                SELECT 1 FROM posts p WHERE p.category_id = c.category_id
             )",
            sphere_name,
            category_name,
        )
            .execute(db_pool)
            .await?;

        match result.rows_affected() {
            0 => Err(AppError::InternalServerError(String::from(CATEGORY_NOT_DELETED_STR))),
            1 => Ok(()),
            count => Err(AppError::InternalServerError(format!("Expected 1 category to be deleted, got {count} instead"))),
        }
    }
}

#[server]
pub async fn set_sphere_category(
    sphere_name: String,
    category_name: String,
    category_color: Color,
    description: String,
    is_active: bool,
) -> Result<SphereCategory, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let sphere_category = ssr::set_sphere_category(&sphere_name, &category_name, category_color, &description, is_active, &user, &db_pool).await?;
    Ok(sphere_category)
}

#[server]
pub async fn delete_sphere_category(
    sphere_name: String,
    category_name: String,
) -> Result<(), ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::delete_sphere_category(&sphere_name, &category_name, &user, &db_pool).await?;
    Ok(())
}