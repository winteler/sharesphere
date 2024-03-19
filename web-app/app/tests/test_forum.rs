use leptos::ServerFnError;

use app::forum;
use app::ranking::{SortType};
use app::post::{Post, PostSortType};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

#[tokio::test]
async fn test_get_forum_contents() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let num_posts = 20usize;
    let mut expected_post_vec = Vec::<Post>::with_capacity(num_posts);

    let (_, mut expected_forum_post_vec) = create_forum_with_posts(
        forum_name,
        num_posts,
        Some((0..num_posts).map(|i| (i as i32)/2).collect()),
        &test_user,
        db_pool.clone()
    ).await?;

    expected_post_vec.append(&mut expected_forum_post_vec);

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

        test_post_vec(
            &posts,
            &expected_post_vec,
            sort_type,
            test_user.user_id,
        );
    }

    let (forum_with_subscription, _) = forum::ssr::get_forum_contents(
        forum_name,
        SortType::Post(PostSortType::Hot),
        Some(test_user.user_id),
        db_pool.clone(),
    ).await?;
    assert!(forum_with_subscription.subscription_id.is_none());

    forum::ssr::subscribe(expected_post_vec.first().expect("Expected post").forum_id, test_user.user_id, db_pool.clone()).await?;
    let (forum_with_subscription, _) = forum::ssr::get_forum_contents(
        forum_name,
        SortType::Post(PostSortType::Hot),
        Some(test_user.user_id),
        db_pool.clone(),
    ).await?;
    assert!(forum_with_subscription.subscription_id.is_some());

    forum::ssr::unsubscribe(expected_post_vec.first().expect("Expected post").forum_id, test_user.user_id, db_pool.clone()).await?;
    let (forum_with_subscription, _) = forum::ssr::get_forum_contents(
        forum_name,
        SortType::Post(PostSortType::Hot),
        Some(test_user.user_id),
        db_pool.clone(),
    ).await?;
    assert!(forum_with_subscription.subscription_id.is_none());

    Ok(())
}