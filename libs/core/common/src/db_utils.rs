#[cfg(feature = "ssr")]
pub mod ssr {
    use std::env;
    use anyhow::Context;
    use sqlx::{postgres::PgPoolOptions, PgPool};
    use leptos::prelude::{use_context};

    use crate::errors::AppError;

    pub const DB_URL_ENV: &str = "DATABASE_URL";
    pub fn get_db_pool() -> Result<PgPool, AppError> {
        use_context::<PgPool>().ok_or_else(|| AppError::new("DB pool missing."))
    }

    pub async fn create_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var(DB_URL_ENV)?)
            .await
            .with_context(|| "Failed to connect to DB")
    }
}