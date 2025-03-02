use rand::Rng;

use app::app::ssr::create_db_pool;
use app::errors::AppError;
use app::errors::AppError::InsufficientPrivileges;
use app::role::PermissionLevel;
use app::sphere;
use app::sphere::ssr::{create_sphere, subscribe, unsubscribe};
use app::sphere::{SphereHeader};
use app::user::ssr::set_user_settings;
use app::user::User;

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

#[tokio::test]
async fn test_is_sphere_available() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let sphere_name = "sphere-";
    sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        &test_user,
        &db_pool,
    ).await.expect("Sphere should be created");

    assert_eq!(
        sphere::ssr::is_sphere_available(sphere_name, &db_pool).await?,
        false
    );
    assert_eq!(
        sphere::ssr::is_sphere_available("Sphere-", &db_pool).await?,
        false
    );
    assert_eq!(
        sphere::ssr::is_sphere_available("sphere_", &db_pool).await?,
        false
    );
    assert_eq!(
        sphere::ssr::is_sphere_available("aSphere-", &db_pool).await?,
        true
    );

    Ok(())
}

#[tokio::test]
async fn test_get_sphere_by_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    let expected_sphere = sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        &test_user,
        &db_pool,
    )
    .await?;

    let sphere = sphere::ssr::get_sphere_by_name(sphere_name, &db_pool).await?;

    assert_eq!(sphere, expected_sphere);

    assert!(sphere::ssr::get_sphere_by_name("invalid_name", &db_pool)
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn test_get_popular_sphere_headers() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let num_sphere = 30;
    let num_sphere_fetch = 20usize;
    for i in 0..num_sphere {
        let sphere = sphere::ssr::create_sphere(
            i.to_string().as_str(),
            "sphere",
            false,
            &test_user,
            &db_pool,
        ).await?;

        set_sphere_num_members(sphere.sphere_id, i, &db_pool).await?;
    }

    let popular_sphere_header_vec =
        sphere::ssr::get_popular_sphere_headers(num_sphere_fetch as i64, &db_pool).await?;
    
    assert_eq!(popular_sphere_header_vec.len(), num_sphere_fetch);
    // check nsfw sphere is excluded
    
    let mut expected_sphere_num = num_sphere - 1;
    for sphere_header in popular_sphere_header_vec {
        assert_eq!(sphere_header.sphere_name, expected_sphere_num.to_string());
        assert_eq!(sphere_header.icon_url, None);
        expected_sphere_num -= 1;
    }

    let nsfw_sphere = sphere::ssr::create_sphere(
        "nsfw",
        "sphere",
        true,
        &test_user,
        &db_pool,
    ).await?;

    let popular_sphere_header_vec =
        sphere::ssr::get_popular_sphere_headers((num_sphere + 1)  as i64, &db_pool).await?;

    assert_eq!(popular_sphere_header_vec.len(), num_sphere as usize);
    assert!(!popular_sphere_header_vec.contains(&(&nsfw_sphere).into()));

    Ok(())
}

#[tokio::test]
async fn test_get_subscribed_sphere_headers() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    // use two users to make sure behaviour is correct both for sphere creator and other users
    let creator_user = create_user("creator", &db_pool).await;
    let member_user = create_user("user", &db_pool).await;

    let num_sphere = 30usize;
    let mut expected_create_sub_sphere_vec = Vec::<SphereHeader>::new();
    let mut expected_member_sub_sphere_vec = Vec::<SphereHeader>::new();
    for i in 0..num_sphere {
        let sphere = sphere::ssr::create_sphere(
            i.to_string().as_str(),
            "sphere",
            i % 2 == 0,
            &creator_user,
            &db_pool,
        )
        .await?;

        if i % 2 == 1 {
            subscribe(sphere.sphere_id, creator_user.user_id, &db_pool).await?;
            expected_create_sub_sphere_vec.push(SphereHeader {
                sphere_name: sphere.sphere_name,
                icon_url: sphere.icon_url,
                is_nsfw: sphere.is_nsfw,
            });
        } else {
            subscribe(sphere.sphere_id, member_user.user_id, &db_pool).await?;
            expected_member_sub_sphere_vec.push(SphereHeader {
                sphere_name: sphere.sphere_name,
                icon_url: sphere.icon_url,
                is_nsfw: sphere.is_nsfw,
            });
        }
    }

    let create_sub_sphere_name_vec = sphere::ssr::get_subscribed_sphere_headers(creator_user.user_id, &db_pool).await?;
    let member_sub_sphere_name_vec = sphere::ssr::get_subscribed_sphere_headers(member_user.user_id, &db_pool).await?;

    assert_eq!(
        create_sub_sphere_name_vec.len(),
        expected_create_sub_sphere_vec.len()
    );
    assert_eq!(
        member_sub_sphere_name_vec.len(),
        expected_member_sub_sphere_vec.len()
    );

    expected_create_sub_sphere_vec.sort_by(|l, r| l.sphere_name.cmp(&r.sphere_name));
    expected_member_sub_sphere_vec.sort_by(|l, r| l.sphere_name.cmp(&r.sphere_name));

    assert_eq!(create_sub_sphere_name_vec, expected_create_sub_sphere_vec);
    assert_eq!(member_sub_sphere_name_vec, expected_member_sub_sphere_vec);

    Ok(())
}

#[tokio::test]
async fn test_get_sphere_with_user_info() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        &test_user,
        &db_pool,
    )
    .await?;

    let sphere_with_subscription =
        sphere::ssr::get_sphere_with_user_info(sphere_name, None, &db_pool).await?;

    assert_eq!(sphere_with_subscription.sphere.sphere_id, sphere.sphere_id);
    assert_eq!(
        sphere_with_subscription.sphere.sphere_name.as_str(),
        sphere.sphere_name
    );
    assert_eq!(sphere_with_subscription.sphere.creator_id, test_user.user_id);
    assert_eq!(sphere_with_subscription.subscription_id, None);

    let sphere_with_subscription = sphere::ssr::get_sphere_with_user_info(
        sphere_name,
        Some(test_user.user_id),
        &db_pool,
    )
    .await?;
    assert!(sphere_with_subscription.subscription_id.is_none());

    sphere::ssr::subscribe(sphere.sphere_id, test_user.user_id, &db_pool).await?;
    let sphere_with_subscription = sphere::ssr::get_sphere_with_user_info(
        sphere_name,
        Some(test_user.user_id),
        &db_pool,
    )
    .await?;
    assert!(sphere_with_subscription.subscription_id.is_some());

    sphere::ssr::unsubscribe(sphere.sphere_id, test_user.user_id, &db_pool).await?;
    let sphere_with_subscription = sphere::ssr::get_sphere_with_user_info(
        sphere_name,
        Some(test_user.user_id),
        &db_pool,
    )
    .await?;
    assert!(sphere_with_subscription.subscription_id.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_sphere() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let sphere_name = "camelCase_snake_case123-";
    let sphere_description = "a";
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        sphere_description,
        false,
        &test_user,
        &db_pool,
    ).await.expect("Should be possible to create sphere.");

    assert_eq!(sphere.sphere_name, sphere_name);
    assert_eq!(sphere.normalized_sphere_name, "camelcase_snake_case123_");
    assert_eq!(sphere.creator_id, test_user.user_id);
    assert_eq!(sphere.description, sphere_description);
    assert_eq!(sphere.is_nsfw, false);
    assert_eq!(sphere.timestamp, sphere.create_timestamp);

    // Check new permissions were created
    let test_user = User::get(test_user.user_id, &db_pool).await.expect("User should be available in DB.");
    assert_eq!(test_user.permission_by_sphere_map.len(), 1);
    let sphere_permission = test_user.permission_by_sphere_map.get(sphere_name).expect("User should have leader role after sphere creation.");
    assert_eq!(*sphere_permission, PermissionLevel::Lead);

    assert!(
        sphere::ssr::create_sphere(&sphere_name, "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        sphere::ssr::create_sphere("camelCase-snake-case123-", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        sphere::ssr::create_sphere("camelcase_snake_case123-", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        sphere::ssr::create_sphere("camelCase_Snake_Case123-", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        sphere::ssr::create_sphere("", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        sphere::ssr::create_sphere(" ", "a", false, &test_user, &db_pool)
            .await
            .is_err()
    );
    assert!(
        sphere::ssr::create_sphere("b", "b", false, &test_user, &db_pool)
            .await
            .is_ok()
    );

    Ok(())
}

#[tokio::test]
async fn test_update_sphere_description() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead = create_user("lead", &db_pool).await;
    let ordinary_user = create_user("user", &db_pool).await;
    let sphere = sphere::ssr::create_sphere(
        "test",
        "first",
        false,
        &lead,
        &db_pool
    ).await.expect("Should be possible to create sphere.");
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be available in DB.");

    let updated_description = "second";
    assert_eq!(
        sphere::ssr::update_sphere_description(
            &sphere.sphere_name,
            updated_description,
            &ordinary_user,
            &db_pool
        ).await,
        Err(InsufficientPrivileges),
    );
    let updated_sphere = sphere::ssr::update_sphere_description(
        &sphere.sphere_name,
        updated_description,
        &lead,
        &db_pool
    ).await.expect("Should be possible to update sphere.");

    assert_eq!(updated_sphere.sphere_id, sphere.sphere_id);
    assert_eq!(updated_sphere.creator_id, lead.user_id);
    assert_eq!(updated_sphere.description, updated_description);
    assert!(updated_sphere.timestamp > sphere.timestamp);
    assert!(updated_sphere.timestamp > updated_sphere.create_timestamp);

    Ok(())
}

#[tokio::test]
async fn test_subscribe() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let sphere_name = "a";
    let sphere_description = "a";
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        sphere_description,
        false,
        &test_user,
        &db_pool,
    ).await.expect("Should be possible to create sphere.");

    subscribe(sphere.sphere_id, test_user.user_id, &db_pool).await.expect("User should be able to subscribe to sphere");

    // duplicated subscription fails
    assert!(subscribe(sphere.sphere_id, test_user.user_id, &db_pool).await.is_err());
    // Subscribe to non-existent sphere fails
    assert!(subscribe(sphere.sphere_id + 1, test_user.user_id, &db_pool).await.is_err());
    // Subscribe with non-existent user fails
    assert!(subscribe(sphere.sphere_id, test_user.user_id + 1, &db_pool).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_unsubscribe() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let sphere_name = "a";
    let sphere_description = "a";
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        sphere_description,
        false,
        &test_user,
        &db_pool,
    ).await.expect("Should be possible to create sphere.");

    // unsubscribe without subscription fails
    assert!(unsubscribe(sphere.sphere_id, test_user.user_id, &db_pool).await.is_err());

    subscribe(sphere.sphere_id, test_user.user_id, &db_pool).await.expect("User should be able to subscribe to sphere.");
    unsubscribe(sphere.sphere_id, test_user.user_id, &db_pool).await.expect("User should be able to unsubscribe to sphere.");

    Ok(())
}

#[tokio::test]
#[ignore]
/// "fake" test used to easily populate dev DB
async fn populate_dev_db() -> Result<(), AppError> {
    let db_pool = create_db_pool().await.expect("DB pool should be available.");
    let mut test_user = create_test_user(&db_pool).await;

    let nsfw_user = create_user("nsfw", &db_pool).await;
    set_user_settings(true, false, None, &nsfw_user, &db_pool).await?;

    let sphere_name = "test";
    let num_posts = 500usize;

    let mut rng = rand::thread_rng();

    // generate sphere with many posts
    let (_sphere, _, _sphere_post_vec) = create_sphere_with_posts(
        sphere_name,
        None,
        num_posts,
        Some((0..num_posts).map(|_| rng.gen_range(-100..101)).collect()),
        (0..num_posts).map(|i| (i % 2) == 0).collect(),
        &mut test_user,
        &db_pool,
    ).await?;

    // generate post with many comment
    let num_comments = 200;
    let mut rng = rand::thread_rng();

    let (post, _, _) = create_post_with_comments(
        sphere_name,
        "Post with comments",
        num_comments,
        (0..num_comments).map(|i| match i {
            i if i > 1 && (i % 2 == 0) => Some(rng.gen_range(0..i-1)),
            _ => None,
        }).collect(),
        (0..num_comments).map(|_| rng.gen_range(-100..101)).collect(),
        (0..num_comments).map(|_| None).collect(),
        &test_user,
        &db_pool
    ).await;

    set_post_score(post.post_id, 200, &db_pool).await?;

    // create nsfw sphere
    create_sphere("nsfw", "hot_stuff", true, &nsfw_user, &db_pool).await?;

    // create other test spheres
    let num_spheres = 100;
    for i in 0..num_spheres {
        create_sphere(&format!("test-{i}"), &format!("Test sphere n°{i}"), false, &test_user, &db_pool).await?;
    }

    Ok(())
}
