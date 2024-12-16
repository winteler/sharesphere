use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Satellite {
    pub satellite_id: i64,
    pub satellite_name: String,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub description: String,
    pub is_nsfw: bool,
    pub is_spoiler: bool,
    pub num_posts: i32,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::errors::AppError;
    use crate::role::PermissionLevel;
    use crate::satellite::Satellite;
    use crate::sphere::Sphere;
    use crate::user::User;
    use sqlx::PgPool;

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
        let sphere = sqlx::query_as!(
            Sphere,
            "SELECT s.* FROM spheres s
            JOIN satellites sa ON sa.sphere_id = s.sphere_id
            WHERE sa.satellite_id = $1",
            satellite_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn create_satellite(
        satellite_name: &str,
        sphere_name: &str,
        description: &str,
        is_nsfw: bool,
        is_spoiler: bool,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        
        let satellite = sqlx::query_as!(
            Satellite,
            "INSERT INTO satellites
            (satellite_name, sphere_id, sphere_name, description, is_nsfw, is_spoiler, creator_id) 
            VALUES (
                $1,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                $2, $3,
                (
                    CASE 
                        WHEN $4 THEN TRUE
                        ELSE (SELECT is_nsfw FROM spheres WHERE sphere_name = $2)
                    END
                ),
                $5, $6
            ) 
            RETURNING *",
            satellite_name,
            sphere_name,
            description,
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
        description: &str,
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
                description = $2,
                is_nsfw = $3,
                is_spoiler = $4
            WHERE satellite_id = $5
            RETURNING *",
            satellite_name,
            description,
            is_nsfw || sphere.is_nsfw,
            is_spoiler,
            satellite_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }

    pub async fn delete_satellite(
        satellite_id: i64,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        let sphere = get_satellite_sphere(satellite_id, db_pool).await?;
        user.check_permissions(&sphere.sphere_name, PermissionLevel::Manage)?;

        let satellite = sqlx::query_as!(
            Satellite,
            "UPDATE satellites
            SET delete_timestamp = CURRENT_TIMESTAMP
            WHERE satellite_id = $1
            RETURNING *",
            satellite_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }
}

