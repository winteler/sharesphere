#![allow(dead_code)]

use std::env;
use std::sync::Mutex;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use utils::user::User;

pub const TEST_DB_URL_ENV: &str = "TEST_DATABASE_URL";
static DB_NUM: Mutex<i32> = Mutex::new(0);
async fn get_main_db_pool() -> PgPool {
    let main_db = "postgres";
    let main_db_url = env::var(TEST_DB_URL_ENV).expect(&format!("Test DB address should be in env variable {TEST_DB_URL_ENV}.")) + main_db;
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&main_db_url)
        .await
        .expect("Should be able to connect to Test DB.")
}

pub async fn get_db_pool() -> PgPool {
    let mut db_num = DB_NUM.lock().unwrap();
    let db_name = format!("test{db_num}");
    let main_db_pool = get_main_db_pool().await;
    println!("Setup database: {db_name}");

    sqlx::query(format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)").as_str())
        .execute(&main_db_pool)
        .await
        .expect(format!("Should be able to delete database: {db_name}").as_str());

    sqlx::query(format!("CREATE DATABASE {db_name}").as_str())
        .execute(&main_db_pool)
        .await
        .expect(format!("Should be able to create database {db_name}").as_str());

    let test_db_url =
        env::var(TEST_DB_URL_ENV).expect(format!("Test DB address should be available in env variable {TEST_DB_URL_ENV}.").as_str()) + db_name.as_str();

    *db_num = *db_num + 1;

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Should be able to connect test DB");

    sqlx::migrate!("../migrations/")
        .run(&db_pool)
        .await
        .expect("SQLx migrations should be executed.");

    db_pool
}

pub async fn create_test_user(db_pool: &PgPool) -> User {
    create_user("test", db_pool).await
}

pub async fn create_user(
    test_id: &str,
    db_pool: &PgPool
) -> User {
    let sql_user = utils::user::ssr::create_or_update_user(test_id, test_id, test_id, db_pool)
        .await
        .expect("Should be possible to create user.");
    User::get(sql_user.user_id, db_pool).await.expect("New user should be available in DB.")
}