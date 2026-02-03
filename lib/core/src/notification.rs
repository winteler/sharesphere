use leptos::prelude::*;
use leptos_fluent::move_tr;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};
use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::{LoadingIcon, NotificationIcon};
use sharesphere_utils::routes::{get_comment_link, get_post_link, NOTIFICATION_ROUTE};
use sharesphere_utils::unpack::SuspenseUnpack;
use sharesphere_utils::widget::TimeSinceWidget;
use sharesphere_auth::auth_widget::{AuthorWidget, LoginWindow};

use crate::sidebar::HomeSidebar;
use crate::sphere::{SphereHeader, SphereHeaderLink};
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
    pub sphere_id: i64,
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub sphere_header: SphereHeader,
    pub sphere_name: String,
    pub satellite_id: Option<i64>,
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
                SELECT username FROM users WHERE user_id = $3
            ), post_info AS (
                SELECT sphere_id, satellite_id, creator_id FROM posts WHERE post_id = $1
            ), new_notification AS (
                INSERT INTO notifications (sphere_id, satellite_id, post_id, comment_id, user_id, trigger_user_id, notification_type)
                VALUES (
                    (SELECT sphere_id FROM post_info),
                    (SELECT satellite_id FROM post_info),
                    $1, $2,
                    CASE
                        WHEN $2 IS NULL THEN
                            (SELECT creator_id FROM post_info)
                        ELSE
                            (SELECT creator_id FROM comments WHERE comment_id = $2)
                    END,
                    $3, $4
                )
                RETURNING *
            )
            SELECT n.*, u.username AS trigger_username, s.sphere_name, s.icon_url, s.is_nsfw
            FROM new_notification n, trigger_user u, spheres s
            WHERE s.sphere_id = n.sphere_id",
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
            "SELECT n.*, u.username AS trigger_username, s.sphere_name, s.icon_url, s.is_nsfw
            FROM notifications n
            JOIN USERS u ON u.user_id = n.trigger_user_id
            JOIN spheres s ON s.sphere_id = n.sphere_id
            WHERE n.user_id = $1
            ORDER BY n.create_timestamp DESC",
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

/// Main page for notifications
#[component]
pub fn NotificationHome() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    *state.notif_reload_trigger.write() += 1;
    view! {
        <SuspenseUnpack resource=state.user let:user>
        {
            match user {
                Some(_) => view! { <NotificationList/> }.into_any(),
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </SuspenseUnpack>
        <HomeSidebar/>
    }
}

/// List of notifications
#[component]
pub fn NotificationList() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <div class="w-full xl:w-3/5 3xl:w-2/5 p-2 mx-auto flex flex-col gap-2">
            <h2 class="py-4 text-4xl text-center">{move_tr!("notifications")}</h2>
            <ul class="flex flex-col flex-1 w-full overflow-x-hidden overflow-y-auto px-2 xl:px-4 divide-y divide-base-content/20">
            <SuspenseUnpack resource=state.notifications let:notifications>
            {
                notifications.iter().map(|notification| view! {
                    <li><NotificationItem notification=notification.clone()/></li>
                }).collect_view()
            }
            </SuspenseUnpack>
            </ul>
        </div>
    }
}

/// Single notification
#[component]
pub fn NotificationItem(notification: Notification) -> impl IntoView {
    let is_moderation = notification.notification_type == NotificationType::Moderation;
    let (message, link) = match (notification.notification_type, notification.comment_id) {
        (NotificationType::Comment, Some(comment_id)) => (
            move_tr!("notification-comment-reply"),
            get_comment_link(&notification.sphere_header.sphere_name, notification.satellite_id, notification.post_id, comment_id),
        ),
        (NotificationType::Comment, None) => (
            move_tr!("notification-post-reply"),
            get_post_link(&notification.sphere_header.sphere_name, notification.satellite_id, notification.post_id),
        ),
        (NotificationType::Moderation, Some(comment_id)) => (
            move_tr!("notification-moderate-post"),
            get_comment_link(&notification.sphere_header.sphere_name, notification.satellite_id, notification.post_id, comment_id),
        ),
        (NotificationType::Moderation, None) => (
            move_tr!("notification-moderate-comment"),
            get_post_link(&notification.sphere_header.sphere_name, notification.satellite_id, notification.post_id),
        ),
    };
    view! {
        <a class="w-full p-2 my-1 flex flex-col gap-1 rounded-sm hover:bg-base-200" href=link>
            <div class="flex gap-1">
                <AuthorWidget author=notification.trigger_username is_moderator=is_moderation/>
                <div>{message}</div>
                <SphereHeaderLink sphere_header=notification.sphere_header/>
            </div>
            <TimeSinceWidget timestamp=notification.create_timestamp/>
        </a>
    }
}