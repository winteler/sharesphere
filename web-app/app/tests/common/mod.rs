use std::env;
use std::sync::Mutex;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use app::auth::User;

pub const TEST_DB_URL_ENV: &str = "TEST_DATABASE_URL";
static DB_NUM: Mutex<i32> = Mutex::new(0);
async fn get_main_db_pool() -> PgPool {
    let main_db = "postgres";
    let main_db_url = env::var(TEST_DB_URL_ENV).expect("Could not get test DB address.") + main_db;
    PgPoolOptions::new()
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

    sqlx::query(format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)").as_str())
        .execute(&main_db_pool)
        .await
        .expect(format!("Could not delete database: {db_name}").as_str());

    sqlx::query(format!("CREATE DATABASE {db_name}").as_str())
        .execute(&main_db_pool)
        .await
        .expect("Could not create database");

    let test_db_url =
        env::var(TEST_DB_URL_ENV).expect("Could not get test DB address.") + db_name.as_str();

    *db_num = *db_num + 1;

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test DB");

    sqlx::migrate!("../migrations/")
        .run(&db_pool)
        .await
        .expect("could not run SQLx migrations");

    db_pool
}

pub async fn create_test_user(db_pool: &PgPool) -> User {
    create_user("test", "test", "test@test.com", db_pool).await
}

pub async fn create_user(
    oidc_user_id: &str,
    username: &str,
    email: &str,
    db_pool: &PgPool
) -> User {
    let sql_user = app::auth::ssr::create_user(oidc_user_id, username, email, db_pool)
        .await
        .expect("Could not create user.");
    User::get(sql_user.user_id, db_pool).await.expect("Could not fetch newly created user.")
}