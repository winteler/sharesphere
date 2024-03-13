use std::env;
use std::sync::{Mutex};
use sqlx::{PgPool};
use sqlx::postgres::{PgPoolOptions};
use tokio::sync::OnceCell;

pub const TEST_DB_URL_ENV : &str = "TEST_DATABASE_URL";
static DB_NUM: Mutex<i32> = Mutex::new(0);
static MAIN_DB_POOL: OnceCell<PgPool> = OnceCell::const_new();
async fn get_main_db_pool() -> &'static PgPool {
    MAIN_DB_POOL.get_or_init(|| async {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var(TEST_DB_URL_ENV)?)
            .await
            .expect("Failed to connect to DB")
    }).await
}

pub async fn get_db_pool() -> PgPool {

    let mut db_num = DB_NUM.lock().unwrap();

    let main_db_pool = get_main_db_pool().await;

    // Determine DB name, delete if exists, create new DB, return pool to new DB

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var(TEST_DB_URL_ENV)?)
        .await
        .expect("Failed to connect to DB")
}