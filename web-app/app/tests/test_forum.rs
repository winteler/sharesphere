use std::collections::BTreeSet;

use rand::Rng;
use sqlx::PgPool;

use app::app::ssr::create_db_pool;
use app::errors::AppError;
use app::forum;
use app::forum::Forum;
use app::forum::ssr::{subscribe, unsubscribe};
use app::role::PermissionLevel;
use app::user::User;

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

async fn set_forum_num_members(
    forum_id: i64,
    num_members: i32,
    db_pool: &PgPool,
) -> Result<Forum, AppError> {
    let forum = sqlx::query_as!(
        Forum,
        "UPDATE forums \
        SET num_members = $1 \
        WHERE forum_id = $2 \
        RETURNING *",
        num_members,
        forum_id
    )
    .fetch_one(db_pool)
    .await?;

    Ok(forum)
}

#[tokio::test]
async fn test_is_forum_available() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
        &db_pool,
    )
    .await?;

    forum::ssr::is_forum_available(forum_name, &db_pool).await?;

    assert_eq!(
        forum::ssr::is_forum_available(forum_name, &db_pool).await?,
        false
    );
    assert_eq!(
        forum::ssr::is_forum_available("AvailableForum", &db_pool).await?,
        true
    );

    Ok(())
}

#[tokio::test]
async fn test_get_forum_by_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let expected_forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
        &db_pool,
    )
    .await?;

    let forum = forum::ssr::get_forum_by_name(forum_name, &db_pool).await?;

    assert_eq!(forum, expected_forum);

    assert!(forum::ssr::get_forum_by_name("invalid_name", &db_pool)
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn test_get_matching_forum_name_set() -> Result<(), AppError> {
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
                &test_user,
                &db_pool,
            )
            .await?
            .forum_name,
        );
    }

    let forum_name_set = forum::ssr::get_matching_forum_name_set("1", num_forums as i64, &db_pool).await?;

    let mut previous_forum_name = None;
    for forum_name in forum_name_set {
        assert_eq!(forum_name.chars().next().unwrap(), '1');
        if let Some(previous_forum_name) = previous_forum_name {
            assert!(previous_forum_name < forum_name)
        }
        previous_forum_name = Some(forum_name);
    }

    for i in num_forums..2 * num_forums {
        expected_forum_name_set.insert(
            forum::ssr::create_forum(
                i.to_string().as_str(),
                "forum",
                false,
                &test_user,
                &db_pool,
            )
            .await?
            .forum_name,
        );
    }

    let forum_name_set =
        forum::ssr::get_matching_forum_name_set("", num_forums as i64, &db_pool).await?;

    assert_eq!(forum_name_set.len(), num_forums);

    Ok(())
}

#[tokio::test]
async fn test_get_popular_forum_names() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let num_forum = 30;
    let num_forum_fetch = 20usize;
    for i in 0..num_forum {
        let forum = forum::ssr::create_forum(
            i.to_string().as_str(),
            "forum",
            false,
            &test_user,
            &db_pool,
        )
        .await?;

        set_forum_num_members(forum.forum_id, i, &db_pool).await?;
    }

    let popular_forum_name_vec =
        forum::ssr::get_popular_forum_names(num_forum_fetch as i64, &db_pool).await?;

    assert_eq!(popular_forum_name_vec.len(), num_forum_fetch);
    let mut expected_forum_num = num_forum - 1;
    for forum_name in popular_forum_name_vec {
        assert_eq!(forum_name, expected_forum_num.to_string());
        expected_forum_num -= 1;
    }

    Ok(())
}

#[tokio::test]
async fn test_get_subscribed_forum_names() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    // use two users to make sure behaviour is correct both for forum creator and other users
    let creator_user = create_user("creator", &db_pool).await;
    let member_user = create_user("user", &db_pool).await;

    let num_forum = 30usize;
    let mut expected_create_sub_forum_vec = Vec::<String>::new();
    let mut expected_member_sub_forum_vec = Vec::<String>::new();
    for i in 0..num_forum {
        let forum = forum::ssr::create_forum(
            i.to_string().as_str(),
            "forum",
            false,
            &creator_user,
            &db_pool,
        )
        .await?;

        if i % 2 == 1 {
            forum::ssr::subscribe(forum.forum_id, creator_user.user_id, &db_pool).await?;
            expected_create_sub_forum_vec.push(forum.forum_name);
        } else {
            forum::ssr::subscribe(forum.forum_id, member_user.user_id, &db_pool).await?;
            expected_member_sub_forum_vec.push(forum.forum_name);
        }
    }

    let create_sub_forum_name_vec = forum::ssr::get_subscribed_forum_names(creator_user.user_id, &db_pool).await?;
    let member_sub_forum_name_vec = forum::ssr::get_subscribed_forum_names(member_user.user_id, &db_pool).await?;

    assert_eq!(
        create_sub_forum_name_vec.len(),
        expected_create_sub_forum_vec.len()
    );
    assert_eq!(
        member_sub_forum_name_vec.len(),
        expected_member_sub_forum_vec.len()
    );

    expected_create_sub_forum_vec.sort();
    expected_member_sub_forum_vec.sort();

    assert_eq!(create_sub_forum_name_vec, expected_create_sub_forum_vec);
    assert_eq!(member_sub_forum_name_vec, expected_member_sub_forum_vec);

    Ok(())
}

#[tokio::test]
async fn test_get_forum_with_user_info() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
        &db_pool,
    )
    .await?;

    let forum_with_subscription =
        forum::ssr::get_forum_with_user_info(forum_name, None, &db_pool).await?;

    assert_eq!(forum_with_subscription.forum.forum_id, forum.forum_id);
    assert_eq!(
        forum_with_subscription.forum.forum_name.as_str(),
        forum.forum_name
    );
    assert_eq!(forum_with_subscription.forum.creator_id, test_user.user_id);
    assert_eq!(forum_with_subscription.subscription_id, None);

    let forum_with_subscription = forum::ssr::get_forum_with_user_info(
        forum_name,
        Some(test_user.user_id),
        &db_pool,
    )
    .await?;
    assert!(forum_with_subscription.subscription_id.is_none());

    forum::ssr::subscribe(forum.forum_id, test_user.user_id, &db_pool).await?;
    let forum_with_subscription = forum::ssr::get_forum_with_user_info(
        forum_name,
        Some(test_user.user_id),
        &db_pool,
    )
    .await?;
    assert!(forum_with_subscription.subscription_id.is_some());

    forum::ssr::unsubscribe(forum.forum_id, test_user.user_id, &db_pool).await?;
    let forum_with_subscription = forum::ssr::get_forum_with_user_info(
        forum_name,
        Some(test_user.user_id),
        &db_pool,
    )
    .await?;
    assert!(forum_with_subscription.subscription_id.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_forum() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "a";
    let forum_description = "a";
    let forum = forum::ssr::create_forum(
        forum_name,
        forum_description,
        false,
        &test_user,
        &db_pool,
    ).await.expect("Should be possible to create forum.");

    assert_eq!(forum.forum_name, forum_name);
    assert_eq!(forum.creator_id, test_user.user_id);
    assert_eq!(forum.description, forum_description);
    assert_eq!(forum.is_nsfw, false);

    // Check new permissions were created
    let test_user = User::get(test_user.user_id, &db_pool).await.expect("User should be available in DB.");
    assert_eq!(test_user.permission_by_forum_map.len(), 1);
    let forum_permission = test_user.permission_by_forum_map.get(forum_name).expect("User should have leader role after forum creation.");
    assert_eq!(*forum_permission, PermissionLevel::Lead);

    assert!(
        forum::ssr::create_forum("A", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        forum::ssr::create_forum("", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        forum::ssr::create_forum("-", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        forum::ssr::create_forum("b", "b", false, &test_user, &db_pool)
            .await
            .is_ok()
    );

    Ok(())
}

#[tokio::test]
async fn test_subscribe() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "a";
    let forum_description = "a";
    let forum = forum::ssr::create_forum(
        forum_name,
        forum_description,
        false,
        &test_user,
        &db_pool,
    ).await.expect("Should be possible to create forum.");

    subscribe(forum.forum_id, test_user.user_id, &db_pool).await.expect("User should be able to subscribe to forum");

    // duplicated subscription fails
    assert!(subscribe(forum.forum_id, test_user.user_id, &db_pool).await.is_err());
    // Subscribe to non-existent forum fails
    assert!(subscribe(forum.forum_id + 1, test_user.user_id, &db_pool).await.is_err());
    // Subscribe with non-existent user fails
    assert!(subscribe(forum.forum_id, test_user.user_id + 1, &db_pool).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_unsubscribe() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "a";
    let forum_description = "a";
    let forum = forum::ssr::create_forum(
        forum_name,
        forum_description,
        false,
        &test_user,
        &db_pool,
    ).await.expect("Should be possible to create forum.");

    // unsubscribe without subscription fails
    assert!(unsubscribe(forum.forum_id, test_user.user_id, &db_pool).await.is_err());

    subscribe(forum.forum_id, test_user.user_id, &db_pool).await.expect("User should be able to subscribe to forum.");
    unsubscribe(forum.forum_id, test_user.user_id, &db_pool).await.expect("User should be able to unsubscribe to forum.");

    Ok(())
}

#[tokio::test]
#[ignore]
/// "fake" test used to easily populate dev DB
async fn populate_dev_db() -> Result<(), AppError> {
    let db_pool = create_db_pool().await.expect("DB pool should be available.");
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
        &db_pool,
    )
    .await?;

    // generate post with many comment
    let num_comments = 200;
    let mut rng = rand::thread_rng();

    let post = create_post_with_comments(
        forum_name,
        "Post with comments",
        num_comments,
        (1..num_comments+1).map(|i| match i {
            i if i > 2 && (i % 2 == 0) => Some(rng.gen_range(0..((i-1) as i64))+1),
            _ => None,
        }).collect(),
        (0..num_comments).map(|_| rng.gen_range(-100..101)).collect(),
        (0..num_comments).map(|_| None).collect(),
        &test_user,
        &db_pool
    ).await?;

    set_post_score(post.post_id, 200, &db_pool).await?;

    Ok(())
}
