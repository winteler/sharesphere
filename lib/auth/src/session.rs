#[cfg(feature = "ssr")]
pub mod ssr {
    use anyhow::Context;
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use std::env;
    use std::sync::Arc;
    use axum_session_sqlx::SessionPgPool;
    use leptos::prelude::use_context;

    use sharesphere_utils::errors::AppError;

    use crate::user::ssr::UserLockCache;
    use crate::user::User;

    pub const DB_URL_ENV: &str = "DATABASE_URL";
    pub const LEPTOS_ENV_KEY: &str = "LEPTOS_ENV";

    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;

    pub fn get_session() -> Result<AuthSession, AppError> {
        use_context::<AuthSession>().ok_or_else(|| AppError::new("Auth session missing."))
    }

    pub fn get_db_pool() -> Result<PgPool, AppError> {
        use_context::<PgPool>().ok_or_else(|| AppError::new("DB pool missing."))
    }

    pub fn get_user_lock_cache() -> Result<Arc<UserLockCache>, AppError> {
        use_context::<Arc<UserLockCache>>().ok_or_else(|| AppError::new("User lock cache missing."))
    }

    pub fn is_prod_mode() -> bool {
        env::var(LEPTOS_ENV_KEY).is_ok_and(|leptos_env| leptos_env == "PROD" )
    }

    pub async fn create_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var(DB_URL_ENV)?)
            .await
            .with_context(|| "Failed to connect to DB")
    }
}