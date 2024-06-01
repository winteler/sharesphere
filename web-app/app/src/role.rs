use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[repr(i16)]
pub enum UserRole {
    User = 0,
    Moderator = 1,
    Leader = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[repr(i16)]
pub enum AdminRole {
    None = 0,
    Moderator = 1,
    Admin = 2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserForumRole {
    pub role_id: i64,
    pub user_id: i64,
    pub forum_id: i64,
    pub forum_name: String,
    pub user_role: UserRole,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<i16> for UserRole {
    fn from(value: i16) -> UserRole {
        match value {
            2 => UserRole::Leader,
            1 => UserRole::Moderator,
            _ => UserRole::User,
        }
    }
}

impl From<i16> for AdminRole {
    fn from(value: i16) -> AdminRole {
        match value {
            2 => AdminRole::Admin,
            1 => AdminRole::Moderator,
            _ => AdminRole::None,
        }
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
    ) -> Result<UserForumRole, AppError> {

        let current_leader_role = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles \
            WHERE forum_id = $1 AND \
                  user_role = $2",
            forum.forum_id,
            UserRole::Leader as i16,
        )
            .fetch_one(&db_pool)
            .await;

        match current_leader_role {
            Ok(current_leader_role) => {
                Ok(sqlx::query_as!(
                    UserForumRole,
                    "UPDATE user_forum_roles \
                    SET user_id = $1, \
                    timestamp = CURRENT_TIMESTAMP \
                    WHERE role_id = $2 \
                    RETURNING *",
                    user_id,
                    current_leader_role.role_id,
                )
                    .fetch_one(&db_pool)
                    .await?)
            },
            Err(sqlx::error::Error::RowNotFound) => {
                Ok(sqlx::query_as!(
                    UserForumRole,
                    "INSERT INTO user_forum_roles (user_id, forum_id, forum_name, user_role) VALUES ($1, $2, $3, $4) RETURNING *",
                    user_id,
                    forum.forum_id,
                    forum.forum_name,
                    UserRole::Leader as i16,
                )
                    .fetch_one(&db_pool)
                    .await?)
            },
            Err(e) => {
                log::error!("Failed to set forum leader with error: {e}");
                Err(e.into())
            }
        }
    }
}