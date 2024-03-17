use leptos::ServerFnError;
use rand::Rng;

use app::forum;
use app::post;
use app::ranking::{SortType};
use app::post::{Post, PostSortType};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

fn test_forum_post_vec(
    post_vec: &Vec<Post>,
    expected_post_vec: &Vec<Post>,
    sort_type: PostSortType,
    expected_user_id: i64,
) {
    let mut index = 0usize;
    for post in post_vec {
        assert_eq!(post.creator_id, expected_user_id);
        assert!(expected_post_vec.contains(post));
        if index > 0 {
            let previous_post = &post_vec[index - 1];
            assert!(match sort_type {
                PostSortType::Hot => post.recommended_score <= previous_post.recommended_score,
                PostSortType::Trending => post.trending_score <= previous_post.trending_score,
                PostSortType::Best => post.score <= previous_post.score,
                PostSortType::Recent => post.create_timestamp <= previous_post.create_timestamp,
            });
        }
        index += 1;
    }
}

#[tokio::test]
async fn test_get_forum_contents() -> Result<(), ServerFnError> {
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

    let num_posts = 20usize;
    let mut expected_post_vec = Vec::<Post>::with_capacity(num_posts);
    let mut rng = rand::thread_rng();

    for i in 0..num_posts {
        let post = post::ssr::create_post(
            forum_name,
            i.to_string().as_str(),
            "body",
            false,
            None,
            &test_user,
            db_pool.clone(),
        ).await?;

        let post = set_post_score(post.post_id, rng.gen_range(-100..101), db_pool.clone()).await?;

        expected_post_vec.push(post);
    }

    let post_sort_type_array = [PostSortType::Hot, PostSortType::Trending, PostSortType::Best, PostSortType::Recent];

    for sort_type in post_sort_type_array {
        let (forum_with_subscription, posts) = forum::ssr::get_forum_contents(
            forum_name,
            SortType::Post(sort_type),
            None,
            db_pool.clone(),
        ).await?;

        assert_eq!(forum_with_subscription.forum.forum_name.as_str(), forum_name);
        assert_eq!(forum_with_subscription.forum.creator_id, test_user.user_id);
        assert_eq!(forum_with_subscription.subscription_id, None);

        test_forum_post_vec(
            &posts,
            &expected_post_vec,
            sort_type,
            test_user.user_id,
        );
    }

    Ok(())
}