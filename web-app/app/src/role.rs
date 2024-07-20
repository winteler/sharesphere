use std::str::FromStr;

use leptos::{server, ServerFnError};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};
use strum_macros::EnumIter;

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::ssr::check_user, auth::ssr::reload_user, auth::ssr::SqlUser};

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum AdminRole {
    None = 0,
    Moderator = 1,
    Admin = 2,
}

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum PermissionLevel {
    None = 0,
    Moderate = 1,
    Ban = 2,
    Manage = 3,
    Lead = 4,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserForumRole {
    pub role_id: i64,
    pub user_id: i64,
    pub username: String,
    pub forum_id: i64,
    pub forum_name: String,
    pub permission_level: PermissionLevel,
    pub grantor_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<String> for PermissionLevel {
    fn from(value: String) -> PermissionLevel {
        PermissionLevel::from_str(&value).unwrap_or(PermissionLevel::None)
    }
}

impl From<String> for AdminRole {
    fn from(value: String) -> AdminRole {
        AdminRole::from_str(&value).unwrap_or(AdminRole::None)
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::auth::{ssr::SqlUser, User};
    use crate::errors::AppError;

    use super::*;

    pub async fn get_user_forum_role(
        user_id: i64,
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<UserForumRole, AppError> {
        let user_forum_role = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles \
            WHERE user_id = $1 AND \
                  forum_name = $2",
            user_id,
            forum_name,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(user_forum_role)
    }

    pub async fn get_forum_role_vec(
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserForumRole>, AppError> {
        let forum_role_vec = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles \
            WHERE forum_name = $1",
            forum_name,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_role_vec)
    }

    pub async fn set_user_forum_role(
        user_id: i64,
        forum_name: &str,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        if permission_level == PermissionLevel::Lead {
            set_forum_leader(user_id, forum_name, grantor, db_pool).await
        } else {
            grantor.check_can_set_user_forum_role(permission_level, user_id, forum_name, db_pool).await?;
            let user_forum_role = insert_user_forum_role(
                user_id,
                forum_name,
                permission_level,
                grantor,
                db_pool,
            ).await?;
            Ok((user_forum_role, None))
        }
    }
    async fn set_forum_leader(
        user_id: i64,
        forum_name: &str,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        match grantor.check_is_forum_leader(forum_name).is_ok() {
            true => {
                let manage_level_str: &str = PermissionLevel::Manage.into();
                sqlx::query_as!(
                    UserForumRole,
                    "UPDATE user_forum_roles
                    SET
                        permission_level = $1,
                        timestamp = CURRENT_TIMESTAMP
                    WHERE user_id = $2 AND
                          forum_name = $3
                    RETURNING *",
                    manage_level_str,
                    grantor.user_id,
                    forum_name,
                )
                    .fetch_one(db_pool)
                    .await?;
                let user_forum_role = insert_user_forum_role(
                    user_id,
                    forum_name,
                    PermissionLevel::Lead,
                    grantor,
                    db_pool,
                ).await?;
                Ok((user_forum_role, Some(grantor.user_id)))
            },
            false => {
                let user_forum_role = insert_user_forum_role(
                    user_id,
                    forum_name,
                    PermissionLevel::Lead,
                    grantor,
                    db_pool,
                ).await?;
                Ok((user_forum_role, None))
            },
        }
    }

    async fn insert_user_forum_role(
        user_id: i64,
        forum_name: &str,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<UserForumRole, AppError> {
        let permission_level_str: &str = permission_level.into();
        let user_forum_role = sqlx::query_as!(
                UserForumRole,
                "INSERT INTO user_forum_roles (user_id, username, forum_id, forum_name, permission_level, grantor_id)
                VALUES ($1,
                    (SELECT username from users where user_id = $1),
                    (SELECT forum_id FROM forums WHERE forum_name = $2),
                    $2, $3, $4)
                ON CONFLICT (user_id, forum_id) DO UPDATE
                SET permission_level = EXCLUDED.permission_level,
                    timestamp = CURRENT_TIMESTAMP
                RETURNING *",
                user_id,
                forum_name,
                permission_level_str,
                grantor.user_id,
            )
            .fetch_one(db_pool)
            .await?;
        Ok(user_forum_role)
    }

    pub async fn set_user_admin_role(
        user_id: i64,
        admin_role: AdminRole,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<SqlUser, AppError> {
        grantor.check_is_admin()?;
        let admin_role_str: &str = admin_role.into();
        let sql_user = sqlx::query_as!(
            SqlUser,
            "UPDATE users
            SET
                admin_role = $1,
                timestamp = CURRENT_TIMESTAMP
            WHERE user_id = $2
            RETURNING *",
            admin_role_str,
            user_id,
        )
            .fetch_one(db_pool)
            .await?;
        Ok(sql_user)
    }
}

#[server]
pub async fn get_forum_role_vec(forum_name: String) -> Result<Vec<UserForumRole>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let role_vec = ssr::get_forum_role_vec(
        &forum_name,
        &db_pool,
    ).await?;

    Ok(role_vec)
}

#[server]
pub async fn set_user_forum_role(
    username: String,
    forum_name: String,
    permission_level: PermissionLevel,
) -> Result<UserForumRole, ServerFnError> {
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let assigned_user = SqlUser::get_by_username(&username, &db_pool).await?;

    let (forum_role, _) = ssr::set_user_forum_role(
        assigned_user.user_id,
        &forum_name,
        permission_level,
        &user,
        &db_pool,
    ).await?;

    reload_user(forum_role.user_id)?;

    Ok(forum_role)
}

#[cfg(test)]
mod tests {
    use crate::role::{AdminRole, PermissionLevel};

    #[test]
    fn test_permission_level_from_string() {
        assert_eq!(PermissionLevel::from(String::from("none")), PermissionLevel::None);
        assert_eq!(PermissionLevel::from(String::from("moderate")), PermissionLevel::Moderate);
        assert_eq!(PermissionLevel::from(String::from("ban")), PermissionLevel::Ban);
        assert_eq!(PermissionLevel::from(String::from("manage")), PermissionLevel::Manage);
        assert_eq!(PermissionLevel::from(String::from("lead")), PermissionLevel::Lead);
        assert_eq!(PermissionLevel::from(String::from("invalid")), PermissionLevel::None);
    }

    #[test]
    fn test_admin_role_from_string() {
        assert_eq!(AdminRole::from(String::from("none")), AdminRole::None);
        assert_eq!(AdminRole::from(String::from("moderator")), AdminRole::Moderator);
        assert_eq!(AdminRole::from(String::from("admin")), AdminRole::Admin);
        assert_eq!(AdminRole::from(String::from("invalid")), AdminRole::None);
    }
}