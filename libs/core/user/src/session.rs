#[cfg(feature = "ssr")]
pub mod ssr {
    use std::env;
    use std::sync::{Arc, LazyLock};
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use axum_session_sqlx::SessionPgPool;
    use leptos::prelude::{use_context, config};

    use sharesphere_core_common::errors::AppError;

    use crate::user::ssr::UserLockCache;
    use crate::user::User;

    pub const DB_URL_ENV: &str = "DATABASE_URL";
    pub const LEPTOS_ENV_KEY: &str = "LEPTOS_ENV";

    static LEPTOS_ENV: LazyLock<Result<Env, AppError>> = LazyLock::new(|| {
        let leptos_env = env::var(LEPTOS_ENV_KEY).unwrap_or("dev").to_lowercase();
        match leptos_env.as_ref() {
            "dev" | "development" => Ok(Env::DEV),
            "prod" | "production" => Ok(Env::PROD),
            _ => Err(AppError::new(format!(
                "{input} is not a supported leptos environment. Use either `dev` or `prod`.",
            ))),
        }
    });

    pub type AuthSession = axum_session_auth::AuthSession<User, i64, SessionPgPool, PgPool>;

    pub fn get_session() -> Result<AuthSession, AppError> {
        use_context::<AuthSession>().ok_or_else(|| AppError::new("Auth session missing."))
    }

    pub fn get_user_lock_cache() -> Result<Arc<UserLockCache>, AppError> {
        use_context::<Arc<UserLockCache>>().ok_or_else(|| AppError::new("User lock cache missing."))
    }
}