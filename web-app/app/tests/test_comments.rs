use sqlx::postgres::PgPoolOptions;
use crate::common::{get_db_pool};

mod common;

#[tokio::test]
async fn test_comment_tree() {
    let db_pool = get_db_pool().await;

    sqlx::migrate!("../migrations/")
        .run(&db_pool)
        .await
        .expect("could not run SQLx migrations");
}