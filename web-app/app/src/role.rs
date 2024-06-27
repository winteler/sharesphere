use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum AdminRole {
    None = 0,
    Moderator = 1,
    Admin = 2,
}

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum PermissionLevel {
    None = 0,
    Moderate = 1,
    Ban = 2,
    Configure = 3,
    Elect = 4,
    Lead = 5,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserForumRole {
    pub role_id: i64,
    pub user_id: i64,
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

    pub async fn set_user_forum_role(
        forum_id: i64,
        forum_name: &str,
        user_id: i64,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        if permission_level == PermissionLevel::Lead {
            set_forum_leader(forum_id, forum_name, user_id, grantor, db_pool.clone()).await
        } else {
            grantor.check_can_elect_in_forum(forum_name)?;
            let permission_level_str: &str = permission_level.into();
            let user_forum_role = sqlx::query_as!(
                UserForumRole,
                "INSERT INTO user_forum_roles (user_id, forum_id, forum_name, permission_level, grantor_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
                user_id,
                forum_id,
                forum_name,
                permission_level_str,
                grantor.user_id,
            )
                .fetch_one(&db_pool)
                .await?;
            Ok((user_forum_role, None))
        }
    }
    async fn set_forum_leader(
        forum_id: i64,
        forum_name: &str,
        user_id: i64,
        grantor: &User,
        db_pool: PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        let lead_level_str: &str = PermissionLevel::Lead.into();
        let current_leader = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles \
            WHERE forum_id = $1 AND \
                  permission_level = $2",
            forum_id,
            lead_level_str,
        )
            .fetch_one(&db_pool)
            .await;

        match current_leader {
            Ok(current_leader) => {
                grantor.check_is_forum_leader(forum_name)?;
                let user_forum_role = sqlx::query_as!(
                    UserForumRole,
                    "UPDATE user_forum_roles \
                    SET \
                        user_id = $1, \
                        grantor_id = $2, \
                        timestamp = CURRENT_TIMESTAMP \
                    WHERE role_id = $3 \
                    RETURNING *",
                    user_id,
                    grantor.user_id,
                    current_leader.role_id,
                )
                    .fetch_one(&db_pool)
                    .await?;
                Ok((user_forum_role, Some(current_leader.user_id)))
            },
            Err(sqlx::error::Error::RowNotFound) => {
                let user_forum_role = sqlx::query_as!(
                    UserForumRole,
                    "INSERT INTO user_forum_roles (user_id, forum_id, forum_name, permission_level, grantor_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
                    user_id,
                    forum_id,
                    forum_name,
                    lead_level_str,
                    grantor.user_id,
                )
                    .fetch_one(&db_pool)
                    .await?;
                Ok((user_forum_role, None))
            },
            Err(e) => {
                log::error!("Failed to get current forum leader with error: {e}");
                Err(e.into())
            }
        }
    }

    pub async fn set_user_admin_role(
        user_id: i64,
        admin_role: AdminRole,
        grantor: &User,
        db_pool: PgPool,
    ) -> Result<SqlUser, AppError> {
        grantor.check_is_admin()?;
        let admin_role_str: &str = admin_role.into();
        let sql_user = sqlx::query_as!(
            SqlUser,
            "UPDATE users \
            SET \
                admin_role = $1, \
                timestamp = CURRENT_TIMESTAMP \
            WHERE user_id = $2 \
            RETURNING *",
            admin_role_str,
            user_id,
        )
            .fetch_one(&db_pool)
            .await?;
        Ok(sql_user)
    }
}