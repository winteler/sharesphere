use std::cmp::min;
use std::collections::BTreeSet;

use leptos::ServerFnError;
use rand::Rng;
use sqlx::PgPool;

use app::app::ssr::create_db_pool;
use app::forum;
use app::forum::Forum;

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

async fn set_forum_num_members(
    forum_id: i64,
    num_members: i32,
    db_pool: PgPool,
) -> Result<Forum, ServerFnError> {
    let forum = sqlx::query_as!(
        Forum,
        "UPDATE forums \
        SET num_members = $1 \
        WHERE forum_id = $2 \
        RETURNING *",
        num_members,
        forum_id
    )
    .fetch_one(&db_pool)
    .await?;

    Ok(forum)
}

#[tokio::test]
async fn test_is_forum_available() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        test_user.user_id,
        db_pool.clone(),
    )
    .await?;

    forum::ssr::is_forum_available(forum_name, db_pool.clone()).await?;

    assert_eq!(
        forum::ssr::is_forum_available(forum_name, db_pool.clone()).await?,
        false
    );
    assert_eq!(
        forum::ssr::is_forum_available("AvailableForum", db_pool).await?,
        true
    );

    Ok(())
}

#[tokio::test]
async fn test_get_forum_by_name() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let expected_forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        test_user.user_id,
        db_pool.clone(),
    )
    .await?;

    let forum = forum::ssr::get_forum_by_name(forum_name, db_pool.clone()).await?;

    assert_eq!(forum, expected_forum);

    assert!(forum::ssr::get_forum_by_name("invalid_name", db_pool)
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn test_get_matching_forum_names() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let num_forums = 20usize;
    let mut expected_forum_name_set = BTreeSet::<String>::new();
    for i in 0..num_forums {
        expected_forum_name_set.insert(
            forum::ssr::create_forum(
                i.to_string().as_str(),
                "forum",
                false,
                test_user.user_id,
                db_pool.clone(),
            )
            .await?
            .forum_name,
        );
    }

    let forum_name_set =
        forum::ssr::get_matching_forum_names(String::from("1"), num_forums as i64, db_pool.clone())
            .await?;

    for forum_name in forum_name_set {
        assert_eq!(forum_name.chars().next().unwrap(), '1');
    }

    for i in num_forums..2 * num_forums {
        expected_forum_name_set.insert(
            forum::ssr::create_forum(
                i.to_string().as_str(),
                "forum",
                false,
                test_user.user_id,
                db_pool.clone(),
            )
            .await?
            .forum_name,
        );
    }

    let forum_name_set =
        forum::ssr::get_matching_forum_names(String::default(), num_forums as i64, db_pool).await?;

    assert_eq!(forum_name_set.len(), num_forums);

    Ok(())
}

#[tokio::test]
async fn test_get_popular_forum_names() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let num_forum = 30;
    let num_forum_fetch = 20usize;
    for i in 0..num_forum {
        let forum = forum::ssr::create_forum(
            i.to_string().as_str(),
            "forum",
            false,
            test_user.user_id,
            db_pool.clone(),
        )
        .await?;

        set_forum_num_members(forum.forum_id, i, db_pool.clone()).await?;
    }

    let popular_forum_name_vec =
        forum::ssr::get_popular_forum_names(num_forum_fetch as i64, db_pool).await?;

    assert_eq!(popular_forum_name_vec.len(), num_forum_fetch);
    let mut expected_forum_num = num_forum - 1;
    for forum_name in popular_forum_name_vec {
        assert_eq!(forum_name, expected_forum_num.to_string());
        expected_forum_num -= 1;
    }

    Ok(())
}

#[tokio::test]
async fn test_get_subscribed_forum_names() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let num_forum = 30usize;
    let num_forum_fetch = 20usize;
    let mut expected_subscribed_forum = Vec::<Forum>::with_capacity(20);
    for i in 0..num_forum {
        let forum = forum::ssr::create_forum(
            i.to_string().as_str(),
            "forum",
            false,
            test_user.user_id,
            db_pool.clone(),
        )
        .await?;

        if i % 2 == 1 {
            forum::ssr::subscribe(forum.forum_id, test_user.user_id, db_pool.clone()).await?;

            expected_subscribed_forum.push(forum);
        }
    }

    let popular_forum_name_vec =
        forum::ssr::get_subscribed_forum_names(test_user.user_id, db_pool).await?;

    assert_eq!(
        popular_forum_name_vec.len(),
        min(num_forum_fetch, expected_subscribed_forum.len())
    );
    let mut prev_forum_name: Option<String> = None;
    for forum_name in popular_forum_name_vec {
        assert_eq!(
            forum_name
                .parse::<usize>()
                .expect("Could not parse forum name.")
                % 2,
            1
        );
        if let Some(prev_forum_name) = prev_forum_name {
            assert!(prev_forum_name < forum_name);
        }
        prev_forum_name = Some(forum_name);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_forum_with_subscription() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        test_user.user_id,
        db_pool.clone(),
    )
    .await?;

    let forum_with_subscription =
        forum::ssr::get_forum_with_subscription(forum_name, None, db_pool.clone()).await?;

    assert_eq!(forum_with_subscription.forum.forum_id, forum.forum_id);
    assert_eq!(
        forum_with_subscription.forum.forum_name.as_str(),
        forum.forum_name
    );
    assert_eq!(forum_with_subscription.forum.creator_id, test_user.user_id);
    assert_eq!(forum_with_subscription.subscription_id, None);

    let forum_with_subscription = forum::ssr::get_forum_with_subscription(
        forum_name,
        Some(test_user.user_id),
        db_pool.clone(),
    )
    .await?;
    assert!(forum_with_subscription.subscription_id.is_none());

    forum::ssr::subscribe(forum.forum_id, test_user.user_id, db_pool.clone()).await?;
    let forum_with_subscription = forum::ssr::get_forum_with_subscription(
        forum_name,
        Some(test_user.user_id),
        db_pool.clone(),
    )
    .await?;
    assert!(forum_with_subscription.subscription_id.is_some());

    forum::ssr::unsubscribe(forum.forum_id, test_user.user_id, db_pool.clone()).await?;
    let forum_with_subscription = forum::ssr::get_forum_with_subscription(
        forum_name,
        Some(test_user.user_id),
        db_pool.clone(),
    )
    .await?;
    assert!(forum_with_subscription.subscription_id.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_forum() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    assert!(
        forum::ssr::create_forum("test1", "forum", false, test_user.user_id, db_pool.clone(),)
            .await
            .is_ok()
    );

    assert!(
        forum::ssr::create_forum("Test", "forum", false, test_user.user_id, db_pool.clone(),)
            .await
            .is_err()
    );

    assert!(
        forum::ssr::create_forum("", "forum", false, test_user.user_id, db_pool.clone(),)
            .await
            .is_err()
    );

    assert!(
        forum::ssr::create_forum("test-2", "forum", false, test_user.user_id, db_pool.clone(),)
            .await
            .is_err()
    );

    Ok(())
}

#[tokio::test]
#[ignore]
/// "fake" test used to easily populate dev DB
async fn populate_dev_db() -> Result<(), ServerFnError> {
    let db_pool = create_db_pool().await.expect("Failed to get DB pool");
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "test";
    let num_posts = 500usize;

    let mut rng = rand::thread_rng();

    // generate forum with many posts
    let (_forum, _forum_post_vec) = create_forum_with_posts(
        forum_name,
        num_posts,
        Some((0..num_posts).map(|_| rng.gen_range(-100..101)).collect()),
        &test_user,
        db_pool.clone(),
    )
    .await?;

    // generate post with many comment
    let num_comments = 200;
    let mut rng = rand::thread_rng();

    let _post = create_post_with_comments(
        forum_name,
        "Post with comments",
        num_comments,
        (0..num_comments).map(|i| match i % 2 {
            1 => Some(rng.gen_range(0..(i as i64))),
            _ => None,
        }).collect(),
        (0..num_comments).map(|_| rng.gen_range(-100..101)).collect(),
        (0..num_comments).map(|_| None).collect(),
        &test_user,
        db_pool.clone()
    ).await?;

    Ok(())
}
