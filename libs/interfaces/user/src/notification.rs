use std::collections::{BTreeMap, HashMap, HashSet};
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
use sharesphere_utils::unpack::{SuspenseUnpack};
use sharesphere_utils::widget::{RefreshResourceButton, TimeSinceWidget};
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