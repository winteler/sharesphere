use std::cmp::max;
use std::collections::{HashMap};
use std::default::Default;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::{NsfwIcon, UserIcon};

use crate::auth::Login;
use crate::role::{AdminRole, PermissionLevel};

#[cfg(feature = "ssr")]
use crate::{
    auth::ssr::{check_user, delete_user_in_oidc_provider, reload_user},
    session::ssr::get_db_pool
};

#[server]
pub async fn delete_user() -> Result<(), AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    ssr::delete_user(&user, &db_pool).await?;
    if let Err(e) = delete_user_in_oidc_provider(&user).await {
        log::error!("Failed to delete user ({}, {}): {e}", user.user_id, user.oidc_id);
    }

    Ok(())
}

#[server]
pub async fn set_user_settings(
    is_nsfw: bool,
    show_nsfw: bool,
    days_hide_spoilers: u32,
) -> Result<(), AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let days_hide_spoilers = match days_hide_spoilers {
        x if x > 0 => Some(x as i32),
        _ => None,
    };
    ssr::set_user_settings(is_nsfw, show_nsfw, days_hide_spoilers, &user, &db_pool).await?;
    reload_user(user.user_id)?;
    Ok(())
}