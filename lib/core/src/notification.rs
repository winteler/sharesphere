use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};
use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::{LoadingIcon, NotificationIcon};
use sharesphere_utils::routes::{NOTIFICATION_ROUTE};

use crate::state::GlobalState;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
};

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

    pub async fn get_notifications(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Notification>, AppError> {
        let notification_vec = sqlx::query_as::<_, Notification>(
            "SELECT n.*, u.username AS trigger_username
            FROM notifications n
            JOIN USERS u ON u.user_id = n.trigger_user_id
            WHERE n.user_id = $1",
        )
            .bind(user_id)
            .fetch_all(db_pool)
            .await?;

        Ok(notification_vec)
    }

    pub async fn read_notification(
        notification_id: i64,
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE notifications SET is_read = TRUE
            WHERE notification_id = $1 and user_id = $2",
            notification_id,
            user_id,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn read_all_notifications(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE notifications SET is_read = TRUE
            WHERE user_id = $1",
            user_id,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn get_notifications() -> Result<Vec<Notification>, AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::get_notifications(user.user_id, &db_pool).await
}

#[server]
pub async fn read_notification(
    notification_id: i64,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::read_notification(notification_id, user.user_id, &db_pool).await
}

#[server]
pub async fn read_all_notifications() -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::read_all_notifications(user.user_id, &db_pool).await
}

/// When logged in, displays a bell button with the number of unread notifications, redirects to the notification page on click.
#[component]
pub fn NotificationButton() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            {
                move || Suspend::new(async move {
                    match state.user.await {
                        Ok(Some(_)) => {
                            let notif_link = view! {
                                <a class="button-rounded-ghost" href=NOTIFICATION_ROUTE>
                                    <NotificationIcon/>
                                </a>
                            }.into_any();
                            match state.notifications.await {
                                Ok(notifications) if !notifications.is_empty() => {
                                    let unread_notif_count = notifications.into_iter().filter(|notif| !notif.is_read).count();
                                    let unread_notif_count = match unread_notif_count {
                                        x if x > 99 => String::from("99+"),
                                        x => x.to_string(),
                                    };
                                    view! {
                                        <a class="button-rounded-ghost relative flex" href=NOTIFICATION_ROUTE>
                                            <NotificationIcon/>
                                            <div class="absolute left-0 bottom-0 z-10 mb-5 ml-6 p-1 px-2 w-fit bg-base-200 rounded-full text-xs select-none">
                                                {unread_notif_count}
                                            </div>
                                        </a>
                                    }.into_any()
                                },
                                Ok(_) => notif_link,
                                Err(e) => {
                                    log::error!("Failed to fetch notifications: {}", e);
                                    notif_link
                                }
                            }
                        },
                        _ => ().into_any(),
                    }
                })
            }
        </Transition>
    }
}