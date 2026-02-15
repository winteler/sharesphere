use std::collections::{BTreeMap, HashSet};
use chrono::{DateTime, Utc};
use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_fluent::{move_tr, tr};
use leptos_use::{breakpoints_tailwind, BreakpointsTailwind, storage::use_local_storage, use_breakpoints, use_interval_fn};
use leptos_use::{use_web_notification_with_options, ShowOptions, UseWebNotificationOptions, UseWebNotificationReturn};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};
use sharesphere_utils::constants::{LOGO_ICON_PATH, SITE_NAME};
use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::{LoadingIcon, NotificationIcon, ReadAllIcon, ReadIcon, UnreadIcon};
use sharesphere_utils::routes::{get_comment_path, get_post_path, NOTIFICATION_ROUTE};
use sharesphere_utils::unpack::SuspenseUnpack;
use sharesphere_utils::widget::{RefreshButton, TimeSinceWidget};
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

const NOTIF_STATE_STORAGE: &str = "notification_state";
const NOTIF_TAG: &str = "sharesphere-notif";
pub const NOTIF_RETENTION_DAYS: i64 = 31;
const NOTIF_RELOAD_INTERVAL_MS: u64 = 30000;

#[repr(i16)]
#[derive(Clone, Copy, Debug, Default, Display, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum NotificationType {
    #[default]
    PostReply = 0,
    CommentReply = 1,
    Moderation = 2,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct NotifHandler {
    emitted_notif_id_set: HashSet<i64>,
    timestamp_2_notif_id: BTreeMap<chrono::DateTime<chrono::Utc>, i64>,
}

impl NotifHandler {
    fn identify_new_notifications(
        &mut self,
        notif_vec: Vec<Notification>,
        unread_notif_id_set: RwSignal<HashSet<i64>>,
    ) -> Vec<Notification> {
        let unread_notif_vec: Vec<Notification> = notif_vec
            .into_iter()
            .filter(|notif| !notif.is_read)
            .collect();
        *unread_notif_id_set.write() = unread_notif_vec.iter().map(|notif| notif.notification_id).collect();

        let mut new_notif_vec = Vec::new();
        for notif in unread_notif_vec.into_iter() {
            if self.emitted_notif_id_set.insert(notif.notification_id) {
                self.timestamp_2_notif_id.insert(notif.create_timestamp, notif.notification_id);
                new_notif_vec.push(notif);
            }
        }
        new_notif_vec
    }

    fn clear_stale_notifications(&mut self, threshold_datetime: DateTime<Utc>) {
        let notif_to_keep = self.timestamp_2_notif_id.split_off(&threshold_datetime);

        for (_, value) in &self.timestamp_2_notif_id {
            self.emitted_notif_id_set.remove(value);
        }
        self.timestamp_2_notif_id = notif_to_keep;
    }

    fn send_notifications_to_browser(
        &self,
        new_notif_vec: Vec<Notification>,
        unread_notif_id_set: RwSignal<HashSet<i64>>,
        build_and_send_notif_fn: impl Fn(String) + Clone + Send + Sync,
    ) {
        if let Some(notif) = new_notif_vec.iter().next() {
            let new_notif_count = new_notif_vec.len();
            let unread_notif_count = unread_notif_id_set.read_untracked().len();
            let body = match (new_notif_count, unread_notif_count) {
                (1, 1) => get_web_notif_text(notif),
                (1, _) => get_web_notif_text(notif) + tr!("web-notif-unread-addon", {"unread_notif_count" => unread_notif_count}).as_str(),
                (new_notif_count, unread_notif_count) if new_notif_count == unread_notif_count => {
                    tr!("multi-web-notif", {"new_notif_count" => new_notif_count})
                },
                (new_notif_count, unread_notif_count) => tr!(
                    "multi-web-notif-with-unread", {"new_notif_count" => new_notif_count, "unread_notif_count" => unread_notif_count}
                ),
            };
            build_and_send_notif_fn(body);
        }
    }

    pub fn handle_notifications(
        &mut self,
        notif_vec: Vec<Notification>,
        unread_notif_id_set: RwSignal<HashSet<i64>>,
        build_and_send_notif_fn: impl Fn(String) + Clone + Send + Sync,
    ) {
        let notif_timestamp_threshold = Utc::now() - chrono::Duration::days(NOTIF_RETENTION_DAYS);

        let new_notif_vec = self.identify_new_notifications(notif_vec, unread_notif_id_set);
        self.clear_stale_notifications(notif_timestamp_threshold);
        self.send_notifications_to_browser(new_notif_vec, unread_notif_id_set, build_and_send_notif_fn);
    }
}

fn build_and_send_notif(body: String) {
    let UseWebNotificationReturn {
        show,
        ..
    } = use_web_notification_with_options(
        UseWebNotificationOptions::default()
            .renotify(true)
            .tag(NOTIF_TAG)
            .icon(LOGO_ICON_PATH)
    );

    show(
        ShowOptions::default()
            .title(SITE_NAME)
            .body(body)
    )
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_utils::errors::AppError;
    use crate::notification::{Notification, NotificationType, NOTIF_RETENTION_DAYS};

    pub async fn create_notification(
        post_id: i64,
        comment_id: Option<i64>,
        trigger_user_id: i64,
        notification_type: NotificationType,
        db_pool: &PgPool,
    ) -> Result<Option<Notification>, AppError> {
        let notification = sqlx::query_as::<_, Notification>(
            "WITH trigger_user AS (
                SELECT username FROM users WHERE user_id = $3
            ), post_info AS (
                SELECT sphere_id, satellite_id, creator_id FROM posts WHERE post_id = $1
            ), notified_user AS (
                SELECT
                    CASE
                        WHEN $2 IS NULL THEN
                            (SELECT creator_id FROM post_info)
                        ELSE
                            (SELECT creator_id FROM comments WHERE comment_id = $2)
                    END AS creator_id
            ), new_notification AS (
                INSERT INTO notifications (sphere_id, satellite_id, post_id, comment_id, user_id, trigger_user_id, notification_type)
                SELECT
                    p.sphere_id,
                    p.satellite_id,
                    $1, $2,
                    nu.creator_id,
                    $3, $4
                FROM post_info p, trigger_user tu, notified_user nu
                WHERE $3 != nu.creator_id
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
            .fetch_optional(db_pool)
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

    pub async fn set_notification_read(
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

    pub async fn set_all_notifications_read(
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

    pub async fn delete_stale_notifications(
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM notifications
            WHERE create_timestamp < NOW() - (INTERVAL '1 day' * $1)",
            NOTIF_RETENTION_DAYS as f64,
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
pub async fn set_notification_read(
    notification_id: i64,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::set_notification_read(notification_id, user.user_id, &db_pool).await
}

#[server]
pub async fn set_all_notifications_read() -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::set_all_notifications_read(user.user_id, &db_pool).await
}

/// When logged in, displays a bell button with the number of unread notifications, redirects to the notification page on click.
#[component]
pub fn NotificationButton() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let (_, set_notif_handler, _) = use_local_storage::<NotifHandler, JsonSerdeCodec>(NOTIF_STATE_STORAGE);
    let is_wide_screen = use_breakpoints(breakpoints_tailwind()).ge(BreakpointsTailwind::Lg);

    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            {
                move || Suspend::new(async move {
                    match state.user.await {
                        Ok(Some(_)) => {
                            use_interval_fn(
                                move || { *state.notif_reload_trigger.write() += 1; },
                                NOTIF_RELOAD_INTERVAL_MS,
                            );
                            let notif_link = view! {
                                <a class="button-rounded-ghost" href=NOTIFICATION_ROUTE>
                                    <NotificationIcon/>
                                </a>
                            }.into_any();
                            match state.notif_resource.await {
                                Ok(notif_vec) if notif_vec.iter().any(|notif| !notif.is_read) => {
                                    set_notif_handler.write().handle_notifications(notif_vec, state.unread_notif_id_set, build_and_send_notif);
                                    let unread_notif_count = move || match (state.unread_notif_id_set.read().len(), is_wide_screen.get()) {
                                        (x, true) if x > 99 => String::from("99+"),
                                        (x, false) if x > 9 => String::from("9+"),
                                        (x, _) => x.to_string(),
                                    };
                                    view! {
                                        <a class="button-rounded-ghost relative flex" href=NOTIFICATION_ROUTE>
                                            <NotificationIcon/>
                                            <div class="notif_counter">
                                                {unread_notif_count}
                                            </div>
                                        </a>
                                    }.into_any()
                                },
                                Ok(_) => {
                                    state.unread_notif_id_set.write().clear();
                                    notif_link
                                },
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
        <div class="w-full xl:w-3/5 3xl:w-2/5 p-2 xl:px-4 mx-auto flex flex-col gap-2">
            <h2 class="py-4 text-4xl text-center">{move_tr!("notifications")}</h2>
            <div class="flex justify-end px-4">
                <RefreshButton refresh_count=state.notif_reload_trigger/>
                <ReadAllNotificationsButton/>
            </div>
            <ul class="flex flex-col flex-1 w-full overflow-x-hidden overflow-y-auto divide-y divide-base-content/20">
            <SuspenseUnpack resource=state.notif_resource let:notif_vec>
            {
                notif_vec.iter().map(|notification| view! {
                    <li><NotificationItem notification=notification.clone()/></li>
                }).collect_view()
            }
            </SuspenseUnpack>
            </ul>
        </div>
    }
}

/// Button to set all notifications as read
#[component]
fn ReadAllNotificationsButton() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let read_all_action = Action::new(move |_: &()| async move {
        set_all_notifications_read().await
    });
    view! {
        <button
            class="button-rounded-ghost w-fit tooltip"
            data-tip=move_tr!("read-all-notifs")
            on:click=move |_| {
                state.unread_notif_id_set.write().clear();
                read_all_action.dispatch(());
            }
        >
            <ReadAllIcon/>
        </button>
    }
}

/// Button to set all notifications as read
#[component]
fn ReadNotificationButton(
    notif_id: i64,
    read_notif_action: Action<(), Result<(), AppError>>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <button
            class="button-rounded-ghost w-fit tooltip tooltip-left"
            data-tip=move_tr!("read-notif")
            on:click=move |ev| {
                ev.prevent_default();
                state.unread_notif_id_set.write().remove(&notif_id);
                read_notif_action.dispatch(());
            }
        >
            <UnreadIcon/>
        </button>
    }
}

/// Single notification
#[component]
pub fn NotificationItem(notification: Notification) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let notif_id = notification.notification_id;
    let is_notif_read = move || !state.unread_notif_id_set.read().contains(&notif_id);
    let is_moderation = notification.notification_type == NotificationType::Moderation;
    let message = get_notification_text(&notification);
    let link = get_notification_link(&notification);

    let read_notif_action = Action::new(move |_: &()| async move {
        set_notification_read(notif_id).await
    });

    Effect::new(move || if let Some(Err(e)) = &*read_notif_action.value().read() {
        log::error!("Failed to set notification as read: {}", e);
    });

    view! {
        <a
            href=link
            class="w-full p-2 my-1 flex justify-between items-center rounded-sm hover:bg-base-200"
            class:text-gray-400=is_notif_read
            on:click=move |_| {
                state.unread_notif_id_set.write().remove(&notif_id);
                read_notif_action.dispatch(());
            }
        >
            <div class="flex flex-col gap-1">
                <div class="leading-7">
                    <div class="inline-block align-middle">
                        <AuthorWidget
                            author_id=notification.trigger_user_id
                            author=notification.trigger_username
                            is_moderator=is_moderation
                            is_grayed_out=notification.is_read
                        />
                    </div>
                    <span
                        class="align-middle px-1"
                        class:text-gray-400=is_notif_read
                    >
                        {message}
                    </span>
                </div>
                <div class="flex gap-1 items-center">
                    <SphereHeaderLink sphere_header=notification.sphere_header/>
                    <TimeSinceWidget timestamp=notification.create_timestamp is_grayed_out=notification.is_read/>
                </div>
            </div>
            <Show
                when=is_notif_read
                fallback=move || view! { <ReadNotificationButton notif_id read_notif_action/> }
            >
                <div class="p-1 lg:p-2"><ReadIcon/></div>
            </Show>
        </a>
    }
}

fn get_notification_link(notification: &Notification) -> String {
    match notification.comment_id {
        Some(comment_id) => get_comment_path(
            &notification.sphere_header.sphere_name,
            notification.satellite_id,
            notification.post_id,
            comment_id,
        ),
        None => get_post_path(
            &notification.sphere_header.sphere_name,
            notification.satellite_id,
            notification.post_id,
        ),
    }
}

fn get_notification_text(notification: &Notification) -> Signal<String> {
    match (notification.notification_type, notification.comment_id) {
        (NotificationType::PostReply, _) => move_tr!("notification-post-reply"),
        (NotificationType::CommentReply, _) => move_tr!("notification-comment-reply"),
        (NotificationType::Moderation, Some(_)) => move_tr!("notification-moderate-post"),
        (NotificationType::Moderation, None) => move_tr!("notification-moderate-comment"),
    }
}

fn get_web_notif_text(notification: &Notification) -> String {
    let username = notification.trigger_username.clone();
    let sphere_name = notification.sphere_name.clone();
    match (notification.notification_type, notification.comment_id) {
        (NotificationType::PostReply, _) => tr!(
            "web-notif-post-reply", {"username" => username, "sphere_name" => sphere_name}
        ),
        (NotificationType::CommentReply, _) => tr!(
            "web-notif-comment-reply", {"username" => username, "sphere_name" => sphere_name}
        ),
        (NotificationType::Moderation, Some(_)) => tr!(
            "web-notif-moderate-post", {"username" => username, "sphere_name" => sphere_name}
        ),
        (NotificationType::Moderation, None) => tr!(
            "web-notif-moderate-comment", {"username" => username, "sphere_name" => sphere_name}
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::LazyLock;
    use leptos::prelude::*;
    use leptos_fluent::__reexports::fluent_templates::{static_loader, LanguageIdentifier, StaticLoader};
    use leptos_fluent::{tr, I18n, Language};

    use crate::notification::{get_web_notif_text, NotifHandler, Notification, NOTIF_RETENTION_DAYS};

    const EN_IDENTIFIER: LanguageIdentifier = unic_langid::langid!("en");
    const FR_IDENTIFIER: LanguageIdentifier = unic_langid::langid!("fr");

    const EN_LANG: Language = Language {
        id: &EN_IDENTIFIER,
        name: "English",
        dir: &leptos_fluent::WritingDirection::Ltr,
        flag: None,
    };
    const FR_LANG: Language = Language {
        id: &FR_IDENTIFIER,
        name: "Français",
        dir: &leptos_fluent::WritingDirection::Ltr,
        flag: None,
    };
    const LANGUAGES: &'static [&Language] = &[
        &EN_LANG,
        &FR_LANG,
    ];

    #[test]
    fn test_notif_handler_identify_new_notifications() {
        let owner = Owner::new();
        owner.set();
        let mut notif_handler = NotifHandler {
            emitted_notif_id_set: [2].into(),
            ..Default::default()
        };

        let timestamp_1 = chrono::Utc::now();
        let timestamp_2 = timestamp_1 - chrono::Duration::days(1);

        let unread_notif_id_set = RwSignal::new(HashSet::default());
        let new_notif = Notification {
            notification_id: 3,
            create_timestamp: timestamp_2,
            ..Default::default()
        };

        let notif_vec = vec![
            Notification {
                notification_id: 1,
                is_read: true,
                ..Default::default()
            },
            Notification {
                notification_id: 2,
                create_timestamp: timestamp_1,
                ..Default::default()
            },
            new_notif.clone(),
        ];

        let new_notif_vec = notif_handler.identify_new_notifications(notif_vec, unread_notif_id_set);
        assert_eq!(new_notif_vec.len(), 1);
        assert_eq!(*new_notif_vec.first().unwrap(), new_notif);

        assert_eq!(unread_notif_id_set.get_untracked(), [2, 3].into());

        assert_eq!(notif_handler.emitted_notif_id_set, [2, 3].into());
        assert_eq!(notif_handler.timestamp_2_notif_id, [(timestamp_2, 3)].into());
    }

    #[test]
    fn test_notif_handler_clear_stale_notifications() {
        let current_timestamp = chrono::Utc::now();
        let threshold_timestamp = current_timestamp - chrono::Duration::days(NOTIF_RETENTION_DAYS);
        let stale_timestamp = current_timestamp - chrono::Duration::days(NOTIF_RETENTION_DAYS + 1);

        let mut notif_handler = NotifHandler {
            emitted_notif_id_set: [1, 2, 3].into(),
            timestamp_2_notif_id: [(stale_timestamp, 1), (threshold_timestamp, 2), (current_timestamp, 3)].into(),
        };

        notif_handler.clear_stale_notifications(threshold_timestamp);
        assert_eq!(notif_handler.emitted_notif_id_set, [2, 3].into());
        assert_eq!(notif_handler.timestamp_2_notif_id, [(threshold_timestamp, 2), (current_timestamp, 3)].into());
    }

    #[test]
    fn test_notif_handler_send_notifications_to_browser() {
        let owner = Owner::new();
        owner.set();
        static_loader! {
            static TRANSLATIONS = {
                locales: "../../locales",
                fallback_language: "en",
            };
        }
        let compound: Vec<&LazyLock<StaticLoader>> = vec![&TRANSLATIONS];
        let i18n = I18n {
            language: RwSignal::new(&LANGUAGES[0]),
            languages: LANGUAGES,
            translations: Signal::derive(move || compound.clone()),
        };

        provide_context(i18n);

        let notif_handler = NotifHandler::default();

        let notif_1 = Notification {
            notification_id: 1,
            ..Default::default()
        };
        let notif_2 = Notification {
            notification_id: 2,
            ..Default::default()
        };

        let mut notif_vec = vec![
            notif_1.clone(),
        ];
        let unread_notif_id_set = RwSignal::new([1].into());

        let expected_body = get_web_notif_text(&notif_1);
        let mock_show_fn = move |body: String| assert_eq!(body, expected_body);
        notif_handler.send_notifications_to_browser(notif_vec.clone(), unread_notif_id_set, mock_show_fn);

        unread_notif_id_set.write_untracked().insert(2);
        let expected_body =
            get_web_notif_text(&notif_1) +
                tr!(
                    "web-notif-unread-addon",
                    {"unread_notif_count" => unread_notif_id_set.read_untracked().len()}
                ).as_str();
        let mock_show_fn = move |body: String| assert_eq!(body, expected_body);
        notif_handler.send_notifications_to_browser(notif_vec.clone(), unread_notif_id_set, mock_show_fn);

        notif_vec.push(notif_2);
        let expected_body = tr!("multi-web-notif", {"new_notif_count" => notif_vec.len()});
        let mock_show_fn = move |body: String| assert_eq!(body, expected_body);
        notif_handler.send_notifications_to_browser(notif_vec.clone(), unread_notif_id_set, mock_show_fn);

        unread_notif_id_set.write_untracked().insert(4);
        let expected_body = tr!(
            "multi-web-notif-with-unread",
            {"new_notif_count" => notif_vec.len(), "unread_notif_count" => unread_notif_id_set.read_untracked().len()}
        );
        let mock_show_fn = move |body: String| assert_eq!(body, expected_body);
        notif_handler.send_notifications_to_browser(notif_vec, unread_notif_id_set, mock_show_fn);
    }
}