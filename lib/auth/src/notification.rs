use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};

#[repr(i16)]
#[derive(Clone, Copy, Debug, Default, Display, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum NotificationType {
    #[default]
    Comment = 0,
    Moderation = 1,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub notification_id: i64,
    pub post_id: i64,
    pub comment_id: Option<i64>,
    pub user_id: i64,
    pub trigger_user_id: i64,
    pub trigger_username: String,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_utils::errors::AppError;
    use crate::notification::{Notification, NotificationType};

    pub async fn create_notification(
        post_id: i64,
        comment_id: Option<i64>,
        trigger_user_id: i64,
        notification_type: NotificationType,
        db_pool: &PgPool,
    ) -> Result<Notification, AppError> {
        let notification = sqlx::query_as::<_, Notification>(
            "WITH trigger_user AS (
                SELECT * FROM users WHERE user_id = $3
            ), new_notification AS (
                INSERT INTO notifications (post_id, comment_id, user_id, trigger_user_id, notification_type)
                VALUES (
                    $1, $2,
                    CASE
                        WHEN $2 IS NULL THEN
                            (SELECT creator_id FROM posts WHERE post_id = $1)
                        ELSE
                            (SELECT creator_id FROM comments WHERE comment_id = $2)
                    END,
                    $3, $4
                )
                RETURNING *
            )
            SELECT n.*, u.username AS trigger_username
            FROM new_notification n, trigger_user u",
        )
            .bind(post_id)
            .bind(comment_id)
            .bind(trigger_user_id)
            .bind(notification_type as i16)
            .fetch_one(db_pool)
            .await?;

        Ok(notification)
    }
}