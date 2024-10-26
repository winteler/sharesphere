use app::user::ssr::UserLockCache;
use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use leptos_axum::AxumRouteListing;
use sqlx::PgPool;
use std::sync::Arc;

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db_pool: PgPool,
    pub user_lock_cache: Arc<UserLockCache>,
    pub routes: Vec<AxumRouteListing>,
}