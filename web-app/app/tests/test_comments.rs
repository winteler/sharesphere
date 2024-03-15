use app::comment::CommentWithChildren;
use app::post::Post;

use crate::common::{create_test_user, get_db_pool};

mod common;
mod data_factory;

/// Generate comments
async fn generate_post_with_comments(
    forum_name: &str,
    post_title: &str,
) -> (Post, Vec<CommentWithChildren>) {
}

#[tokio::test]
async fn test_comment_tree() {
    let db_pool = get_db_pool().await;

    let test_user = create_test_user(&db_pool).await;
}
