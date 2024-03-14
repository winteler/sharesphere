use std::env;
use std::sync::{Mutex};
use anyhow::Context;
use leptos::ServerFnError;
use sqlx::{PgPool};
use sqlx::postgres::{PgPoolOptions};
use tokio::sync::OnceCell;

pub const TEST_DB_URL_ENV : &str = "TEST_DATABASE_URL";
static DB_NUM: Mutex<i32> = Mutex::new(0);
async fn get_main_db_pool() -> PgPool {
    let main_db = "postgres";
    let main_db_url = env::var(TEST_DB_URL_ENV).expect("Could not get test DB address.") + main_db;PgPoolOptions::new()
        .max_connections(5)
        .connect(&main_db_url)
        .await
        .expect("Failed to connect to test DB")
}

pub async fn get_db_pool() -> PgPool {

    let mut db_num = DB_NUM.lock().unwrap();
    let db_name = format!("test{db_num}");
    let main_db_pool = get_main_db_pool().await;
    println!("Setup database: {db_name}");

    sqlx::query(
        format!("DROP DATABASE IF EXISTS {db_name}").as_str()
    )
        .execute(&main_db_pool)
        .await
        .expect(format!("Could not delete database: {db_name}").as_str());

    sqlx::query(
        format!("CREATE DATABASE {db_name}").as_str()
    )
        .execute(&main_db_pool)
        .await
        .expect("Could not create database");

    let test_db_url = env::var(TEST_DB_URL_ENV).expect("Could not get test DB address.") + db_name.as_str();

    *db_num = *db_num + 1;

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test DB")
}