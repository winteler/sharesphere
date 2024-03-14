use crate::common::{create_test_user, get_db_pool};

mod common;

#[tokio::test]
async fn test_comment_tree() {
    let db_pool = get_db_pool().await;

    sqlx::migrate!("../migrations/")
        .run(&db_pool)
        .await
        .expect("could not run SQLx migrations");

    let test_user = create_test_user(&db_pool).await;
}