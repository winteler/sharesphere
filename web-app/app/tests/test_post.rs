use std::collections::HashMap;
use std::time::Duration;

use float_cmp::approx_eq;
use rand::Rng;

pub use crate::common::*;
pub use crate::data_factory::*;
use app::colors::Color;
use app::comment::ssr::create_comment;
use app::editor::get_styled_html_from_markdown;
use app::errors::AppError;
use app::moderation::ssr::moderate_post;
use app::post::ssr::{create_post, get_post_by_id, get_post_sphere, get_post_with_info_by_id, update_post, update_post_scores};
use app::post::{ssr, Post, PostSortType, PostWithSphereInfo};
use app::ranking::ssr::vote_on_content;
use app::ranking::{SortType, VoteValue};
use app::role::AdminRole;
use app::rule::ssr::add_rule;
use app::satellite::ssr::create_satellite;
use app::sphere_category::ssr::set_sphere_category;
use app::user::User;
use app::{post, sphere};

mod common;
mod data_factory;

pub fn sort_post_vec(
    post_vec: &mut [PostWithSphereInfo],
    sort_type: PostSortType,
) {
    match sort_type {
        PostSortType::Hot => post_vec.sort_by(|l, r| r.post.recommended_score.partial_cmp(&l.post.recommended_score).unwrap()),
        PostSortType::Trending => post_vec.sort_by(|l, r| r.post.trending_score.partial_cmp(&l.post.trending_score).unwrap()),
        PostSortType::Best => post_vec.sort_by(|l, r| r.post.score.partial_cmp(&l.post.score).unwrap()),
        PostSortType::Recent => post_vec.sort_by(|l, r| r.post.create_timestamp.partial_cmp(&l.post.create_timestamp).unwrap()),
    }
}

pub fn test_post_vec(
    post_vec: &[PostWithSphereInfo],
    expected_post_vec: &[PostWithSphereInfo],
    sort_type: PostSortType,
) {
    assert_eq!(post_vec.len(), expected_post_vec.iter().len());
    // Check that all expected post are present
    for (i, expected_post) in expected_post_vec.iter().enumerate() {
        let has_post = post_vec.contains(expected_post);
        if !has_post {
            println!("Missing expected post {i}: {:?}", expected_post);
        }
        assert!(has_post);
        
    }
    // Check that the elements are sorted correctly, the exact ordering could be different if the sort value is identical for multiple posts
    for (index, (post_with_info, expected_post_with_info)) in post_vec.iter().zip(expected_post_vec.iter()).enumerate() {
        let post = &post_with_info.post;
        let expected_post = &expected_post_with_info.post;
        assert!(match sort_type {
            PostSortType::Hot => post.recommended_score == expected_post.recommended_score,
            PostSortType::Trending => post.trending_score == expected_post.trending_score,
            PostSortType::Best => post.score == expected_post.score,
            PostSortType::Recent => post.create_timestamp == expected_post.create_timestamp,
        });
        if index > 0 {
            let previous_post = &post_vec[index - 1].post;
            assert!(match sort_type {
                PostSortType::Hot => post.recommended_score <= previous_post.recommended_score,
                PostSortType::Trending => post.trending_score <= previous_post.trending_score,
                PostSortType::Best => post.score <= previous_post.score,
                PostSortType::Recent => post.create_timestamp <= previous_post.create_timestamp,
            });
        }
    }
}

pub fn test_post_score(post: &Post) {
    let second_delta = post
        .scoring_timestamp
        .signed_duration_since(post.create_timestamp)
        .num_milliseconds();
    let num_days_old = (post
        .scoring_timestamp
        .signed_duration_since(post.create_timestamp)
        .num_milliseconds() as f64)
        / 86400000.0;

    println!(
        "Scoring timestamp: {}, create timestamp: {}, second delta: {second_delta}, num_days_old: {num_days_old}",
        post.scoring_timestamp,
        post.create_timestamp,
    );

    let expected_recommended_score = (post.score as f64) * f64::powf(2.0, 3.0 * (2.0 - num_days_old));
    let expected_trending_score = (post.score as f64) * f64::powf(2.0, 8.0 * (1.0 - num_days_old));

    println!("Recommended: {}, expected: {}", post.recommended_score, expected_recommended_score);
    assert!(approx_eq!(f32, post.recommended_score, expected_recommended_score as f32, epsilon = f32::EPSILON, ulps = 5));
    println!("Trending: {}, expected: {}", post.trending_score, expected_trending_score);
    assert!(approx_eq!(f32, post.trending_score, expected_trending_score as f32, epsilon = f32::EPSILON, ulps = 5));
}

#[tokio::test]
async fn test_get_post_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let sphere = sphere::ssr::create_sphere("a", "sphere", false, &user, &db_pool).await?;

    let post_1_title = "1";
    let post_1_body = "test";
    let expected_post_1 = create_post(&sphere.sphere_name, None, post_1_title, post_1_body, None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 1.");

    let post_2_title = "1";
    let post_2_body = "test";
    let post_2_markdown_body = "test";
    let expected_post_2 = create_post(&sphere.sphere_name, None, post_2_title, post_2_body, Some(post_2_markdown_body), false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 2.");

    let post_1 = get_post_by_id(expected_post_1.post_id, &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_1, expected_post_1);
    let post_2 = get_post_by_id(expected_post_2.post_id, &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_2, expected_post_2);

    Ok(())
}

#[tokio::test]
async fn test_get_post_with_info_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    
    let sphere = sphere::ssr::create_sphere("a", "sphere", false, &user, &db_pool).await?;

    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let sphere_category = set_sphere_category(
        &sphere.sphere_name,
        "b",
        Color::Orange, 
        "test", 
        true,
        &user,
        &db_pool
    ).await.expect("Should be able to set sphere category.");

    let post_1_title = "1";
    let post_1_body = "test";
    let post_1 = create_post(
        &sphere.sphere_name,
        None,
        post_1_title,
        post_1_body,
        None, 
        false, 
        false, 
        false,
        Some(sphere_category.category_id),
        &user,
        &db_pool
    ).await.expect("Should be able to create post 1.");

    let post_2_title = "1";
    let post_2_body = "test";
    let post_2_markdown_body = "test";
    let post_2 = create_post(
        &sphere.sphere_name,
        None,
        post_2_title,
        post_2_body,
        Some(post_2_markdown_body), 
        false, 
        false, 
        false,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to create post 2.");

    let post_1_without_vote = get_post_with_info_by_id(post_1.post_id, None, &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_1_without_vote.post, post_1);
    assert_eq!(post_1_without_vote.sphere_category.expect("Should have category"), sphere_category.clone().into());
    assert_eq!(post_1_without_vote.vote, None);

    let post_1_without_vote = get_post_with_info_by_id(post_1.post_id, Some(&user), &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_1_without_vote.post, post_1);
    assert_eq!(post_1_without_vote.sphere_category.expect("Should have category"), sphere_category.into());
    assert_eq!(post_1_without_vote.vote, None);

    let post_2_without_vote = get_post_with_info_by_id(post_2.post_id, None, &db_pool).await.expect("Should be able to load post 2.");
    assert_eq!(post_2_without_vote.post, post_2);
    assert_eq!(post_2_without_vote.sphere_category, None);
    assert_eq!(post_2_without_vote.vote, None);

    let post_2_without_vote = get_post_with_info_by_id(post_2.post_id, Some(&user), &db_pool).await.expect("Should be able to load post 2.");
    assert_eq!(post_2_without_vote.post, post_2);
    assert_eq!(post_2_without_vote.sphere_category, None);
    assert_eq!(post_2_without_vote.vote, None);

    let post_1_vote = vote_on_content(VoteValue::Up, post_1.post_id, None, None, &user, &db_pool).await.expect("Should be possible to vote on post_1.");
    let post_2_vote = vote_on_content(VoteValue::Down, post_2.post_id, None, None, &user, &db_pool).await.expect("Should be possible to vote on post_2.");

    let post_1_with_vote = get_post_with_info_by_id(post_1.post_id, Some(&user), &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_1_with_vote.post.post_id, post_1.post_id);
    assert_eq!(post_1_with_vote.post.creator_id, user.user_id);
    assert_eq!(post_1_with_vote.post.creator_name, user.username);
    assert_eq!(post_1_with_vote.post.title, post_1_title);
    assert_eq!(post_1_with_vote.post.body, post_1_body);
    assert_eq!(post_1_with_vote.post.markdown_body, None);
    assert_eq!(post_1_with_vote.post.score, 1);
    assert_eq!(post_1_with_vote.vote, post_1_vote);

    let post_2_with_vote = get_post_with_info_by_id(post_2.post_id, Some(&user), &db_pool).await.expect("Should be able to load post 2.");
    assert_eq!(post_2_with_vote.post.post_id, post_2.post_id);
    assert_eq!(post_2_with_vote.post.creator_id, user.user_id);
    assert_eq!(post_2_with_vote.post.creator_name, user.username);
    assert_eq!(post_2_with_vote.post.title, post_2_title);
    assert_eq!(post_2_with_vote.post.body, post_2_body);
    assert_eq!(post_2_with_vote.post.markdown_body, Some(String::from(post_2_markdown_body)));
    assert_eq!(post_2_with_vote.post.score, -1);
    assert_eq!(post_2_with_vote.vote, post_2_vote);

    Ok(())
}

#[tokio::test]
async fn test_get_post_sphere() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let sphere = sphere::ssr::create_sphere("a", "sphere", false, &user, &db_pool).await?;
    let post = create_post(&sphere.sphere_name, None, "1", "test", None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create post.");

    let result_sphere = get_post_sphere(post.post_id, &db_pool).await.expect("Post sphere should be available.");
    assert_eq!(result_sphere, sphere);

    Ok(())
}

#[tokio::test]
async fn test_get_subscribed_post_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let sphere1_name = "1";
    let sphere2_name = "2";
    let num_post = 10usize;

    let (sphere1, _, mut expected_post_vec) = create_sphere_with_posts(
        sphere1_name,
        Some("url"),
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        (0..num_post).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await.expect("Should create sphere 1 with posts.");

    // create post in satellite to make sure it doesn't get included in the results
    let satellite = create_satellite(
        "a",
        &sphere1.sphere_name,
        "satellite",
        false,
        false,
        &user,
        &db_pool,
    ).await.expect("Should be able to insert satellite.");
    
    create_post(
        &satellite.sphere_name,
        Some(satellite.satellite_id),
        "satellite",
        "satellite",
        None,
        false,
        false,
        false,
        None,
        &user,
        &db_pool,
    ).await.expect("Should create satellite post.");
    

    create_sphere_with_posts(
        sphere2_name,
        None,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        (0..num_post).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await.expect("Should create sphere 2 with posts.");

    let post_vec = ssr::get_subscribed_post_vec(
        user.user_id,
        SortType::Post(PostSortType::Hot),
        num_post as i64,
        0,
        &db_pool,
    ).await?;
    assert!(post_vec.is_empty());

    sphere::ssr::subscribe(sphere1.sphere_id, user.user_id, &db_pool).await?;

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_subscribed_post_vec(
            user.user_id,
            SortType::Post(sort_type),
            num_post as i64,
            0,
            &db_pool,
        ).await?;
        sort_post_vec(&mut expected_post_vec, sort_type);
        test_post_vec(&post_vec, &expected_post_vec, sort_type);
    }

    // test banned post are not returned
    user.admin_role = AdminRole::Admin;
    let rule = add_rule(None, 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");
    let moderated_post = moderate_post(
        expected_post_vec.first().expect("First post should be accessible.").post.post_id,
        rule.rule_id,
        "test",
        &user,
        &db_pool,
    ).await.expect("Post should be moderated.");

    let post_vec = ssr::get_subscribed_post_vec(
        user.user_id,
        SortType::Post(PostSortType::Hot),
        num_post as i64,
        0,
        &db_pool,
    ).await?;
    let moderated_post = PostWithSphereInfo::from_post(moderated_post, None, None);
    assert!(!post_vec.contains(&moderated_post));

    // test no posts are returned after unsubscribing
    sphere::ssr::unsubscribe(sphere1.sphere_id, user.user_id, &db_pool).await?;
    let post_vec = ssr::get_subscribed_post_vec(
        user.user_id,
        SortType::Post(PostSortType::Hot),
        num_post as i64,
        0,
        &db_pool,
    ).await?;
    assert!(post_vec.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_sorted_post_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let sphere1_name = "1";
    let sphere2_name = "2";
    let num_post = 10;
    let mut expected_post_vec = Vec::<PostWithSphereInfo>::new();

    let (_, _, mut expected_sphere1_post_vec) = create_sphere_with_posts(
        sphere1_name,
        Some("url"),
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        (0..num_post).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await?;
    expected_post_vec.append(&mut expected_sphere1_post_vec);

    let (_, _, mut expected_sphere2_post_vec) = create_sphere_with_posts(
        sphere2_name,
        None,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        (0..num_post).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await?;
    expected_post_vec.append(&mut expected_sphere2_post_vec);
    
    // create nsfw post to check it's filtered from result
    create_post(
        sphere1_name,
        None,
        "nsfw",
        "nsfw",
        None,
        false,
        true,
        false,
        None,
        &user,
        &db_pool,
    ).await.expect("nsfw_post should be created.");

    // create post in satellite to make sure it doesn't get included in the results
    let satellite = create_satellite(
        "a",
        sphere1_name,
        "satellite",
        false,
        false,
        &user,
        &db_pool,
    ).await.expect("Should be able to insert satellite.");

    create_post(
        &satellite.sphere_name,
        Some(satellite.satellite_id),
        "satellite",
        "satellite",
        None,
        false,
        false,
        false,
        None,
        &user,
        &db_pool,
    ).await.expect("Should create satellite post.");

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_sorted_post_vec(
            SortType::Post(sort_type),
            num_post as i64,
            0,
            &db_pool
        ).await.expect("First post vec should be loaded");
        let second_post_vec = ssr::get_sorted_post_vec(
            SortType::Post(sort_type),
            num_post as i64,
            num_post as i64,
            &db_pool
        ).await.expect("Second post vec should be loaded");
        sort_post_vec(&mut expected_post_vec, sort_type);
        test_post_vec(&post_vec, &expected_post_vec[..num_post], sort_type);
        test_post_vec(&second_post_vec, &expected_post_vec[num_post..2*num_post], sort_type);
    }

    // Moderate post, test that it is no longer in the result
    user.admin_role = AdminRole::Admin;
    let rule = add_rule(None, 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");
    let moderated_post = moderate_post(
        expected_post_vec.first().expect("First post should be accessible.").post.post_id,
        rule.rule_id,
        "test",
        &user,
        &db_pool,
    ).await.expect("Post should be moderated.");

    let post_vec = ssr::get_sorted_post_vec(SortType::Post(PostSortType::Hot), num_post as i64, 0, &db_pool).await?;
    let moderated_post = PostWithSphereInfo::from_post(moderated_post, None, None);
    assert!(!post_vec.contains(&moderated_post));

    Ok(())
}

#[tokio::test]
async fn test_get_post_vec_by_sphere_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    let num_posts = 20usize;

    let (sphere, sphere_category_1, mut expected_post_vec) = create_sphere_with_posts(
        sphere_name,
        None,
        num_posts,
        Some((0..num_posts).map(|i| i as i32).collect()),
        (0..num_posts).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await.expect("Should create sphere with posts");

    let sphere_category_2 = set_sphere_category(
        sphere_name,
        "a",
        Color::Red,
        "a",
        true,
        &user,
        &db_pool
    ).await.expect("Sphere category should be set");
    
    let category_map = HashMap::from([
        (sphere_category_1.category_id, sphere_category_1.clone()),
        (sphere_category_2.category_id, sphere_category_2.clone()),
    ]);

    let category_post_1 = create_post(
        sphere_name,
        None,
        "1",
        "1",
        None,
        true,
        true,
        false,
        Some(sphere_category_2.category_id),
        &user,
        &db_pool
    ).await.expect("Post 1 with category should be created.");

    expected_post_vec.push(PostWithSphereInfo::from_post(
        category_post_1,
        Some(sphere_category_2.clone().into()),
        sphere.icon_url.clone())
    );

    // create post in satellite to make sure it doesn't get included in the results
    let satellite = create_satellite(
        "a",
        sphere_name,
        "satellite",
        false,
        false,
        &user,
        &db_pool,
    ).await.expect("Should be able to insert satellite.");

    create_post(
        &satellite.sphere_name,
        Some(satellite.satellite_id),
        "satellite",
        "satellite",
        None,
        false,
        false,
        false,
        None,
        &user,
        &db_pool,
    ).await.expect("Should create satellite post.");

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    let load_count = 15;
    for sort_type in post_sort_type_array {
        sort_post_vec(&mut expected_post_vec, sort_type);
        let post_vec = ssr::get_post_vec_by_sphere_name(
            sphere_name,
            None,
            SortType::Post(sort_type),
            load_count as i64,
            0,
            &db_pool,
        ).await.expect("First post vec should be loaded");
        
        
        let post_vec: Vec<PostWithSphereInfo> = post_vec.into_iter().map(|post| {
            let sphere_category = post.category_id.map(|category_id| {
                category_map.get(&category_id).expect("Category should be in map").clone().into()
            });
            PostWithSphereInfo::from_post(post, sphere_category, sphere.icon_url.clone())
        }).collect();
        test_post_vec(&post_vec, &expected_post_vec[..load_count], sort_type);
        
        let second_post_vec = ssr::get_post_vec_by_sphere_name(
            sphere_name,
            None,
            SortType::Post(sort_type),
            load_count as i64,
            load_count as i64,
            &db_pool,
        ).await?;

        let second_post_vec: Vec<PostWithSphereInfo> = second_post_vec.into_iter().map(|post| {
            let sphere_category = post.category_id.map(|category_id| {
                category_map.get(&category_id).expect("Category should be in map").clone().into()
            });
            PostWithSphereInfo::from_post(post, sphere_category, sphere.icon_url.clone())
        }).collect();
        test_post_vec(&second_post_vec, &expected_post_vec[load_count..(num_posts + 1)], sort_type);
    }

    user.admin_role = AdminRole::Admin;

    let rule = add_rule(None, 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");
    let moderated_post = moderate_post(
        expected_post_vec.first().expect("First post should be accessible.").post.post_id,
        rule.rule_id,
        "test",
        &user,
        &db_pool,
    ).await.expect("Post should be moderated.");

    let post_vec = ssr::get_post_vec_by_sphere_name(
        sphere_name,
        None,
        SortType::Post(PostSortType::Hot),
        num_posts as i64,
        0,
        &db_pool,
    ).await?;

    assert!(!post_vec.contains(&moderated_post));

    Ok(())
}

#[tokio::test]
async fn test_get_post_vec_by_sphere_name_with_pinned_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    let num_posts = 20usize;

    create_sphere_with_posts(
        sphere_name,
        Some("url"),
        num_posts,
        Some((0..num_posts).map(|i| i as i32).collect()),
        (0..num_posts).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await.expect("Sphere with post should be created");
    let partial_load_num_post = num_posts / 2;

    let pinned_post = create_post(
        sphere_name,
        None,
        "pinned",
        "a",
        None,
        false,
        false,
        true,
        None,
        &user,
        &db_pool
    ).await.expect("Pinned post should be created");

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];
    
    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_post_vec_by_sphere_name(
            sphere_name,
            None,
            SortType::Post(sort_type),
            partial_load_num_post as i64,
            0,
            &db_pool,
        ).await?;

        assert_eq!(post_vec.len(), partial_load_num_post);
        assert_eq!(post_vec[0], pinned_post);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_post_vec_by_sphere_name_with_category() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    let num_posts = 10usize;

    let (sphere, _, _) = create_sphere_with_posts(
        sphere_name,
        None,
        num_posts,
        Some((0..num_posts).map(|i| i as i32).collect()),
        (0..num_posts).map(|i| (i % 2) == 0).collect(),
        &mut user,
        &db_pool,
    ).await.expect("Sphere with post should be created");
    
    let sphere_category = set_sphere_category(
        sphere_name,
        "a",
        Color::Green,
        "a",
        true,
        &user,
        &db_pool
    ).await.expect("Sphere category should be set.");

    let category_post_1 = create_post(
        sphere_name,
        None,
        "1",
        "1",
        None,
        false,
        false,
        false,
        Some(sphere_category.category_id),
        &user,
        &db_pool
    ).await.expect("Post 1 with category should be created.");

    let category_post_2 = create_post(
        sphere_name,
        None,
        "2",
        "2",
        None,
        false,
        false,
        false,
        Some(sphere_category.category_id),
        &user,
        &db_pool
    ).await.expect("Post 2 with category should be created.");

    let mut expected_post_vec = vec![
        PostWithSphereInfo::from_post(category_post_1, Some(sphere_category.clone().into()), sphere.icon_url.clone()),
        PostWithSphereInfo::from_post(category_post_2, Some(sphere_category.clone().into()), sphere.icon_url.clone()),
    ];

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let category_post_vec = ssr::get_post_vec_by_sphere_name(
            sphere_name,
            Some(sphere_category.category_id),
            SortType::Post(sort_type),
            num_posts as i64,
            0,
            &db_pool,
        ).await?;
        let category_post_vec: Vec<PostWithSphereInfo> = category_post_vec.into_iter().map(|post| {
            let sphere_category = post.category_id.map(|_| sphere_category.clone().into());
            PostWithSphereInfo::from_post(post, sphere_category, sphere.icon_url.clone())
        }).collect();
        sort_post_vec(&mut expected_post_vec, sort_type);
        test_post_vec(&category_post_vec, &expected_post_vec, sort_type);
    }

    Ok(())
}

#[tokio::test]
async fn test_create_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let sphere_1 = sphere::ssr::create_sphere("a", "sphere", false, &user, &db_pool).await?;
    let sphere_2 = sphere::ssr::create_sphere("b", "sphere", true, &user, &db_pool).await?;

    let post_1_title = "1";
    let post_1_body = "test";
    let post_1 = create_post(
        &sphere_1.sphere_name,
        None,
        post_1_title,
        post_1_body,
        None,
        false,
        false,
        false,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to create post 1.");

    assert_eq!(post_1.title, post_1_title);
    assert_eq!(post_1.body, post_1_body);
    assert_eq!(post_1.markdown_body, None);
    assert_eq!(post_1.is_nsfw, false);
    assert_eq!(post_1.is_spoiler, false);
    assert_eq!(post_1.category_id, None);
    assert_eq!(post_1.is_edited, false);
    assert_eq!(post_1.sphere_id, sphere_1.sphere_id);
    assert_eq!(post_1.sphere_name, sphere_1.sphere_name);
    assert_eq!(post_1.satellite_id, None);
    assert_eq!(post_1.creator_id, user.user_id);
    assert_eq!(post_1.creator_name, user.username);
    assert_eq!(post_1.is_creator_moderator, false); // user not refreshed yet
    assert_eq!(post_1.moderator_message, None);
    assert_eq!(post_1.infringed_rule_id, None);
    assert_eq!(post_1.infringed_rule_title, None);
    assert_eq!(post_1.moderator_id, None);
    assert_eq!(post_1.moderator_name, None);
    assert_eq!(post_1.num_comments, 0);
    assert_eq!(post_1.is_pinned, false);
    assert_eq!(post_1.score, 0);

    // cannot create pinned comment without moderator permissions (need to reload user to actualize them)
    assert_eq!(
        create_post(&sphere_1.sphere_name, None, post_1_title, post_1_body, None, false, false, true, None, &user, &db_pool).await,
        Err(AppError::InsufficientPrivileges),
    );

    let user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded.");
    let post_2_title = "1";
    let post_2_body = "test";
    let post_2_markdown_body = "test";
    let post_2 = create_post(&sphere_1.sphere_name, None, post_2_title, post_2_body, Some(post_2_markdown_body), true, true, true, None, &user, &db_pool).await.expect("Should be able to create post 2.");

    assert_eq!(post_2.title, post_2_title);
    assert_eq!(post_2.body, post_2_body);
    assert_eq!(post_2.markdown_body, Some(String::from(post_2_markdown_body)));
    assert_eq!(post_2.is_nsfw, true);
    assert_eq!(post_2.is_spoiler, true);
    assert_eq!(post_2.category_id, None);
    assert_eq!(post_2.is_edited, false);
    assert_eq!(post_2.sphere_id, sphere_1.sphere_id);
    assert_eq!(post_2.sphere_name, sphere_1.sphere_name);
    assert_eq!(post_2.satellite_id, None);
    assert_eq!(post_2.creator_id, user.user_id);
    assert_eq!(post_2.creator_name, user.username);
    assert_eq!(post_2.is_creator_moderator, true);
    assert_eq!(post_2.moderator_message, None);
    assert_eq!(post_2.infringed_rule_id, None);
    assert_eq!(post_2.infringed_rule_title, None);
    assert_eq!(post_2.moderator_id, None);
    assert_eq!(post_2.moderator_name, None);
    assert_eq!(post_2.num_comments, 0);
    assert_eq!(post_2.is_pinned, true);
    assert_eq!(post_2.score, 0);

    let nsfw_post_title = "1";
    let nsfw_post_body = "test";
    let nsfw_post = create_post(&sphere_2.sphere_name, None, nsfw_post_title, nsfw_post_body, None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create nsfw post.");

    assert_eq!(nsfw_post.title, nsfw_post_title);
    assert_eq!(nsfw_post.body, nsfw_post_body);
    assert_eq!(nsfw_post.markdown_body, None);
    assert_eq!(nsfw_post.is_nsfw, true);
    assert_eq!(nsfw_post.is_spoiler, false);
    assert_eq!(nsfw_post.category_id, None);
    assert_eq!(nsfw_post.is_edited, false);
    assert_eq!(nsfw_post.sphere_id, sphere_2.sphere_id);
    assert_eq!(nsfw_post.sphere_name, sphere_2.sphere_name);
    assert_eq!(nsfw_post.satellite_id, None);
    assert_eq!(nsfw_post.creator_id, user.user_id);
    assert_eq!(nsfw_post.creator_name, user.username);
    assert_eq!(nsfw_post.is_creator_moderator, true);
    assert_eq!(nsfw_post.moderator_message, None);
    assert_eq!(nsfw_post.infringed_rule_id, None);
    assert_eq!(nsfw_post.infringed_rule_title, None);
    assert_eq!(nsfw_post.moderator_id, None);
    assert_eq!(nsfw_post.moderator_name, None);
    assert_eq!(nsfw_post.num_comments, 0);
    assert_eq!(nsfw_post.is_pinned, false);
    assert_eq!(nsfw_post.score, 0);

    let post_1_with_info = get_post_with_info_by_id(post_1.post_id, None, &db_pool).await.expect("Should be able to load post 1.");

    assert_eq!(post_1_with_info.post, post_1);
    assert_eq!(post_1_with_info.vote, None);

    let post_2_with_info = get_post_with_info_by_id(post_2.post_id, None, &db_pool).await.expect("Should be able to load post 2.");

    assert_eq!(post_2_with_info.post, post_2);
    assert_eq!(post_2_with_info.vote, None);

    let nsfw_post_with_info = get_post_with_info_by_id(nsfw_post.post_id, None, &db_pool).await.expect("Should be able to load post 2.");

    assert_eq!(nsfw_post_with_info.post, nsfw_post);
    assert_eq!(nsfw_post_with_info.vote, None);

    Ok(())
}

#[tokio::test]
async fn test_create_post_in_satellite() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (sphere_1, satellite_1) = create_sphere_with_satellite(
        "1",
        "1",
        false,
        false,
        &mut user,
        &db_pool
    ).await.expect("Should be able to create sphere.");

    let post = create_post(
        &sphere_1.sphere_name,
        Some(satellite_1.satellite_id),
        "1",
        "1",
        None,
        false,
        false,
        false,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to create post in.");

    assert_eq!(post.title, "1");
    assert_eq!(post.body, "1");
    assert_eq!(post.markdown_body, None);
    assert_eq!(post.is_nsfw, false);
    assert_eq!(post.is_spoiler, false);
    assert_eq!(post.category_id, None);
    assert_eq!(post.is_edited, false);
    assert_eq!(post.sphere_id, sphere_1.sphere_id);
    assert_eq!(post.sphere_name, sphere_1.sphere_name);
    assert_eq!(post.satellite_id, Some(satellite_1.satellite_id));
    assert_eq!(post.creator_id, user.user_id);
    assert_eq!(post.creator_name, user.username);
    assert_eq!(post.is_creator_moderator, true);
    assert_eq!(post.moderator_message, None);
    assert_eq!(post.infringed_rule_id, None);
    assert_eq!(post.infringed_rule_title, None);
    assert_eq!(post.moderator_id, None);
    assert_eq!(post.moderator_name, None);
    assert_eq!(post.num_comments, 0);
    assert_eq!(post.is_pinned, false);
    assert_eq!(post.score, 0);

    let (sphere_2, satellite_2) = create_sphere_with_satellite(
        "2",
        "2",
        true,
        true,
        &mut user,
        &db_pool
    ).await.expect("Should be able to create sphere.");

    let post = create_post(
        &sphere_2.sphere_name,
        Some(satellite_2.satellite_id),
        "2",
        "2",
        None,
        false,
        false,
        true,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to create post in.");

    assert_eq!(post.title, "2");
    assert_eq!(post.body, "2");
    assert_eq!(post.markdown_body, None);
    assert_eq!(post.is_nsfw, true);
    assert_eq!(post.is_spoiler, true);
    assert_eq!(post.category_id, None);
    assert_eq!(post.is_edited, false);
    assert_eq!(post.sphere_id, sphere_2.sphere_id);
    assert_eq!(post.sphere_name, sphere_2.sphere_name);
    assert_eq!(post.satellite_id, Some(satellite_2.satellite_id));
    assert_eq!(post.creator_id, user.user_id);
    assert_eq!(post.creator_name, user.username);
    assert_eq!(post.is_creator_moderator, true);
    assert_eq!(post.moderator_message, None);
    assert_eq!(post.infringed_rule_id, None);
    assert_eq!(post.infringed_rule_title, None);
    assert_eq!(post.moderator_id, None);
    assert_eq!(post.moderator_name, None);
    assert_eq!(post.num_comments, 0);
    assert_eq!(post.is_pinned, true);
    assert_eq!(post.score, 0);

    // cannot create post for non-existent satellite
    assert!(
        matches!(
            create_post(
                &sphere_1.sphere_name,
                Some(-1),
                "a",
                "b",
                None,
                false,
                false,
                false,
                None,
                &user,
                &db_pool
            ).await,
            Err(AppError::DatabaseError(_))
        )
    );

    Ok(())
}

#[tokio::test]
async fn test_update_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        &user,
        &db_pool,
    ).await?;
    
    let nsfw_sphere = sphere::ssr::create_sphere(
        "nsfw",
        "nsfw",
        true,
        &user,
        &db_pool,
    ).await?;

    let post = post::ssr::create_post(
        sphere_name,
        None,
        "post",
        "body",
        None,
        false,
        false,
        false,
        None,
        &user,
        &db_pool,
    ).await?;

    let updated_title = "updated post";
    let updated_markdown_body = "# Here is a post with markdown";
    let updated_html_body = get_styled_html_from_markdown(String::from(updated_markdown_body)).await.expect("Should get html from markdown.");
    let updated_post = post::ssr::update_post(
        post.post_id,
        updated_title,
        &updated_html_body,
        Some(updated_markdown_body),
        false,
        false,
        false,
        None,
        &user,
        &db_pool
    ).await?;

    assert_eq!(updated_post.title, updated_title);
    assert_eq!(updated_post.body, updated_html_body);
    assert_eq!(updated_post.markdown_body, Some(String::from(updated_markdown_body)));
    assert!(
        updated_post.edit_timestamp.is_some() &&
        updated_post.edit_timestamp.unwrap() > updated_post.create_timestamp &&
        updated_post.create_timestamp == post.create_timestamp
    );

    let nsfw_sphere_post = post::ssr::create_post(
        &nsfw_sphere.sphere_name,
        None,
        "post",
        "body",
        None,
        false,
        true,
        false,
        None,
        &user,
        &db_pool,
    ).await?;

    let updated_nsfw_post = post::ssr::update_post(
        nsfw_sphere_post.post_id,
        updated_title,
        &updated_html_body,
        Some(updated_markdown_body),
        false,
        false,
        false,
        None,
        &user,
        &db_pool
    ).await?;

    assert_eq!(updated_nsfw_post.title, updated_title);
    assert_eq!(updated_nsfw_post.body, updated_html_body);
    assert_eq!(updated_nsfw_post.markdown_body, Some(String::from(updated_markdown_body)));
    // a post in a nsfw sphere is always nsfw, input of the update is ignored
    assert_eq!(updated_nsfw_post.is_nsfw, true);
    assert!(
        updated_nsfw_post.edit_timestamp.is_some() &&
            updated_nsfw_post.edit_timestamp.unwrap() > updated_nsfw_post.create_timestamp &&
            updated_nsfw_post.create_timestamp == nsfw_sphere_post.create_timestamp
    );

    Ok(())
}

#[tokio::test]
async fn test_update_post_in_satellite() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (sphere_1, satellite_1) = create_sphere_with_satellite(
        "1",
        "1",
        false,
        false,
        &mut user,
        &db_pool
    ).await.expect("Should be able to create sphere");

    let post = create_post(
        &sphere_1.sphere_name,
        Some(satellite_1.satellite_id),
        "1",
        "1",
        None,
        true,
        true,
        false,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to create post in");

    let updated_title = "updated post";
    let updated_markdown_body = "# Here is a post with markdown";
    let updated_html_body = get_styled_html_from_markdown(String::from(updated_markdown_body)).await.expect("Should get html from markdown");
    
    let updated_post = update_post(
        post.post_id,
        updated_title,
        &updated_html_body,
        Some(updated_markdown_body),
        false,
        false,
        true,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to update post");

    assert_eq!(updated_post.title, updated_title);
    assert_eq!(updated_post.body, updated_html_body);
    assert_eq!(updated_post.markdown_body, Some(String::from(updated_markdown_body)));
    assert_eq!(updated_post.satellite_id, Some(satellite_1.satellite_id));
    assert_eq!(updated_post.is_spoiler, false);
    assert_eq!(updated_post.is_nsfw, false);
    assert_eq!(updated_post.is_pinned, true);
    assert!(
        updated_post.edit_timestamp.is_some() &&
            updated_post.edit_timestamp.unwrap() > updated_post.create_timestamp &&
            updated_post.create_timestamp == post.create_timestamp
    );

    let (sphere_2, satellite_2) = create_sphere_with_satellite(
        "2",
        "2",
        true,
        true,
        &mut user,
        &db_pool
    ).await.expect("Should be able to create sphere");

    let post = create_post(
        &sphere_2.sphere_name,
        Some(satellite_2.satellite_id),
        "2",
        "2",
        None,
        true,
        true,
        false,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to create post in");

    let updated_post = update_post(
        post.post_id,
        updated_title,
        &updated_html_body,
        Some(updated_markdown_body),
        false,
        false,
        false,
        None,
        &user,
        &db_pool
    ).await.expect("Should be able to update post");

    assert_eq!(updated_post.title, updated_title);
    assert_eq!(updated_post.body, updated_html_body);
    assert_eq!(updated_post.markdown_body, Some(String::from(updated_markdown_body)));
    assert_eq!(updated_post.satellite_id, Some(satellite_2.satellite_id));
    assert_eq!(updated_post.is_spoiler, true);
    assert_eq!(updated_post.is_nsfw, true);
    assert_eq!(updated_post.is_pinned, false);
    assert!(
        updated_post.edit_timestamp.is_some() &&
            updated_post.edit_timestamp.unwrap() > updated_post.create_timestamp &&
            updated_post.create_timestamp == post.create_timestamp
    );

    Ok(())
}

#[tokio::test]
async fn increment_post_comment_count() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;
    assert_eq!(post.num_comments, 0);

    let _comment = create_comment(post.post_id, None, "a", None, false, &user, &db_pool).await.expect("Should create comment.");

    let post = get_post_by_id(post.post_id, &db_pool).await.expect("Should get post.");
    assert_eq!(post.num_comments, 1);

    Ok(())
}

#[tokio::test]
async fn test_update_post_scores() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;
    let post = set_post_score(post.post_id, 10, &db_pool).await.expect("Post score should be set.");

    // wait to have a meaningful difference in scores after update
    tokio::time::sleep(Duration::from_secs(2)).await;

    update_post_scores(&db_pool).await.expect("Post scores should be updatable.");

    let updated_post = get_post_with_info_by_id(post.post_id, None, &db_pool).await.expect("Should be able to get updated post.");

    test_post_score(&post);
    test_post_score(&updated_post.post);
    assert_eq!(post.score, updated_post.post.score);
    assert_eq!(post.create_timestamp, updated_post.post.create_timestamp);
    assert!(post.scoring_timestamp < updated_post.post.scoring_timestamp);
    assert!(post.recommended_score > updated_post.post.recommended_score);
    assert!(post.trending_score > updated_post.post.trending_score);

    Ok(())
}

#[tokio::test]
async fn test_post_scores() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;

    let mut rng = rand::thread_rng();

    // wait to have a meaningful impact of elapsed time on the score
    tokio::time::sleep(Duration::from_secs(2)).await;

    set_post_score(post.post_id, rng.gen_range(-100..101), &db_pool).await?;

    let post_with_vote = post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool).await?;

    test_post_score(&post_with_vote.post);
    Ok(())
}
