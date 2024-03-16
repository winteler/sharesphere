use app::comment;
use app::forum;
use app::post;
use leptos::ServerFnError;
use rand::Rng;
use app::comment::CommentSortType;
use app::ranking::SortType;

use crate::common::{create_test_user, get_db_pool};
use crate::data_factory::set_comment_score;

mod common;
mod data_factory;

#[tokio::test]
async fn test_comment_tree() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        test_user.user_id,
        db_pool.clone(),
    ).await?;

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        false,
        None,
        &test_user,
        db_pool.clone(),
    ).await?;

    let mut rng = rand::thread_rng();
    let comment_id_vec = Vec::<i64>::new();

    for i in 0..20 {
        let comment = i.to_string();
        let parent_id = comment_id_vec.get(i % 5);

        let comment = comment::ssr::create_comment(
            post.post_id,
            parent_id.cloned(),
            comment,
            &test_user,
            db_pool.clone()
        ).await?;

        set_comment_score(
            comment.comment_id,
            rng.gen_range(-100..101),
            db_pool.clone(),
        ).await?
    }

    let comment_sort_type_array = [
        CommentSortType::Best,
        CommentSortType::Recent,
    ];

    for sort_type in comment_sort_type_array {
        let comment_tree  = comment::ssr::get_post_comment_tree(
            post.post_id,
            SortType::Comment(sort_type),
            None,
            db_pool.clone(),
        ).await?;
    }

    Ok(())
}
