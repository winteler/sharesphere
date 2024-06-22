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

    use crate::errors::AppError;
    use crate::forum::Forum;

    use super::*;

    pub async fn set_forum_leader(
        forum: Forum,
        user_id: i64,
        db_pool: PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        let lead_level_str: &str = PermissionLevel::Lead.into();
        let current_leader = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles \
            WHERE forum_id = $1 AND \
                  permission_level = $2",
            forum.forum_id,
            lead_level_str,
        )
            .fetch_one(&db_pool)
            .await;

        match current_leader {
            Ok(current_leader) => {
                let user_forum_role = sqlx::query_as!(
                    UserForumRole,
                    "UPDATE user_forum_roles \
                    SET user_id = $1, \
                    timestamp = CURRENT_TIMESTAMP \
                    WHERE role_id = $2 \
                    RETURNING *",
                    user_id,
                    current_leader.role_id,
                )
                    .fetch_one(&db_pool)
                    .await?;
                Ok((user_forum_role, Some(current_leader.user_id)))
            },
            Err(sqlx::error::Error::RowNotFound) => {
                let user_forum_role = sqlx::query_as!(
                    UserForumRole,
                    "INSERT INTO user_forum_roles (user_id, forum_id, forum_name, permission_level) VALUES ($1, $2, $3, $4) RETURNING *",
                    user_id,
                    forum.forum_id,
                    forum.forum_name,
                    lead_level_str,
                )
                    .fetch_one(&db_pool)
                    .await?;
                Ok((user_forum_role, None))
            },
            Err(e) => {
                log::error!("Failed to set forum leader with error: {e}");
                Err(e.into())
            }
        }
    }
}