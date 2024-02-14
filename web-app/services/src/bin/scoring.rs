use anyhow::Context;
use std::env;
use sqlx::{postgres::{PgPoolOptions}};

/*use app::{
    forum::get_all_forum_names
};*/

pub const DB_URL_ENV : &str = "DATABASE_URL";

#[tokio::main]
async fn main() {
    let _pool = get_db_pool().await.unwrap();

    println!("Hello, world!");
}

async fn get_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var(DB_URL_ENV)?)
        .await
        .with_context(|| format!("Failed to connect to DB"))
}