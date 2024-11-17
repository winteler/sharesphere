use std::time::Duration;

use float_cmp::approx_eq;
use rand::Rng;

pub use crate::common::*;
pub use crate::data_factory::*;
use app::comment::ssr::create_comment;
use app::editor::get_styled_html_from_markdown;
use app::errors::AppError;
use app::moderation::ssr::moderate_post;
use app::post::ssr::{create_post, get_post_by_id, get_post_forum, get_post_with_info_by_id, update_post_scores};
use app::post::{ssr, Post, PostSortType};
use app::ranking::ssr::vote_on_content;
use app::ranking::{SortType, VoteValue};
use app::role::AdminRole;
use app::user::User;
use app::{forum, forum_management, post};

mod common;
mod data_factory;

pub fn test_post_vec(
    post_vec: &[Post],
    expected_post_vec: &[Post],
    sort_type: PostSortType,
    expected_user_id: i64,
) {
    for (index, post) in post_vec.iter().enumerate() {
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

    let forum = forum::ssr::create_forum("a", "forum", false, &user, &db_pool).await?;

    let post_1_title = "1";
    let post_1_body = "test";
    let expected_post_1 = create_post(&forum.forum_name, post_1_title, post_1_body, None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 1.");

    let post_2_title = "1";
    let post_2_body = "test";
    let post_2_markdown_body = "test";
    let expected_post_2 = create_post(&forum.forum_name, post_2_title, post_2_body, Some(post_2_markdown_body), false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 2.");

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

    let forum = forum::ssr::create_forum("a", "forum", false, &user, &db_pool).await?;

    let post_1_title = "1";
    let post_1_body = "test";
    let post_1 = create_post(&forum.forum_name, post_1_title, post_1_body, None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 1.");

    let post_2_title = "1";
    let post_2_body = "test";
    let post_2_markdown_body = "test";
    let post_2 = create_post(&forum.forum_name, post_2_title, post_2_body, Some(post_2_markdown_body), false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 2.");

    let post_1_without_vote = get_post_with_info_by_id(post_1.post_id, None, &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_1_without_vote.post, post_1);
    assert_eq!(post_1_without_vote.vote, None);

    let post_1_without_vote = get_post_with_info_by_id(post_1.post_id, Some(&user), &db_pool).await.expect("Should be able to load post 1.");
    assert_eq!(post_1_without_vote.post, post_1);
    assert_eq!(post_1_without_vote.vote, None);

    let post_2_without_vote = get_post_with_info_by_id(post_2.post_id, None, &db_pool).await.expect("Should be able to load post 2.");
    assert_eq!(post_2_without_vote.post, post_2);
    assert_eq!(post_2_without_vote.vote, None);

    let post_2_without_vote = get_post_with_info_by_id(post_2.post_id, Some(&user), &db_pool).await.expect("Should be able to load post 2.");
    assert_eq!(post_2_without_vote.post, post_2);
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
async fn test_get_post_forum() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let forum = forum::ssr::create_forum("a", "forum", false, &user, &db_pool).await?;
    let post = create_post(&forum.forum_name, "1", "test", None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create post.");

    let result_forum = get_post_forum(post.post_id, &db_pool).await.expect("Post forum should be available.");
    assert_eq!(result_forum, forum);

    Ok(())
}

#[tokio::test]
async fn test_get_subscribed_post_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let forum1_name = "1";
    let forum2_name = "2";
    let num_post = 10usize;
    let mut expected_post_vec = Vec::<Post>::new();

    let (forum1, mut expected_forum1_post_vec) = create_forum_with_posts(
        forum1_name,
        10,
        Some((0..10).map(|i| i).collect()),
        &user,
        &db_pool,
    ).await?;
    expected_post_vec.append(&mut expected_forum1_post_vec);

    create_forum_with_posts(
        forum2_name,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        &user,
        &db_pool,
    ).await?;

    let post_vec = ssr::get_subscribed_post_vec(
        user.user_id,
        SortType::Post(PostSortType::Hot),
        num_post as i64,
        0,
        &db_pool,
    ).await?;
    assert!(post_vec.is_empty());

    forum::ssr::subscribe(forum1.forum_id, user.user_id, &db_pool).await?;

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
        test_post_vec(&post_vec, &expected_post_vec, sort_type, user.user_id);
    }

    // test banned post are not returned
    user.admin_role = AdminRole::Admin;
    let rule = forum_management::ssr::add_rule(None, 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");
    let moderated_post = moderate_post(
        expected_post_vec.first().expect("First post should be accessible.").post_id,
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

    assert!(!post_vec.contains(&moderated_post));

    // test no posts are returned after unsubscribing
    forum::ssr::unsubscribe(forum1.forum_id, user.user_id, &db_pool).await?;
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

    let forum1_name = "1";
    let forum2_name = "2";
    let num_post = 10usize;
    let mut expected_post_vec = Vec::<Post>::new();

    let (_, mut expected_forum1_post_vec) = create_forum_with_posts(
        forum1_name,
        10,
        Some((0..10).map(|i| i).collect()),
        &user,
        &db_pool,
    ).await?;
    expected_post_vec.append(&mut expected_forum1_post_vec);

    let (_, mut expected_forum2_post_vec) = create_forum_with_posts(
        forum2_name,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        &user,
        &db_pool,
    ).await?;
    expected_post_vec.append(&mut expected_forum2_post_vec);

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_sorted_post_vec(SortType::Post(sort_type), num_post as i64, 0, &db_pool).await?;
        test_post_vec(&post_vec, &expected_post_vec, sort_type, user.user_id);
    }

    // Moderate post, test that it is no longer in the result
    user.admin_role = AdminRole::Admin;
    let rule = forum_management::ssr::add_rule(None, 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");
    let moderated_post = moderate_post(
        expected_post_vec.first().expect("First post should be accessible.").post_id,
        rule.rule_id,
        "test",
        &user,
        &db_pool,
    ).await.expect("Post should be moderated.");

    let post_vec = ssr::get_sorted_post_vec(SortType::Post(PostSortType::Hot), num_post as i64, 0, &db_pool).await?;

    assert!(!post_vec.contains(&moderated_post));

    Ok(())
}

#[tokio::test]
async fn test_get_post_vec_by_forum_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let num_posts = 20usize;
    let mut expected_post_vec = Vec::<Post>::with_capacity(num_posts);

    let (_, mut expected_forum_post_vec) = create_forum_with_posts(
        forum_name,
        num_posts,
        Some((0..num_posts).map(|i| (i as i32) / 2).collect()),
        &user,
        &db_pool,
    ).await?;

    expected_post_vec.append(&mut expected_forum_post_vec);

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_post_vec_by_forum_name(
            forum_name,
            None,
            SortType::Post(sort_type),
            num_posts as i64,
            0,
            &db_pool,
        ).await?;

        test_post_vec(&post_vec, &expected_post_vec, sort_type, user.user_id);
    }

    let partial_load_num_post = num_posts / 2;
    // Reload user to refresh moderator permission to create pinned post
    let mut user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded.");
    let pinned_post = create_post(
        forum_name,
        "pinned",
        "a",
        None,
        false,
        false,
        true,
        None,
        &user,
        &db_pool
    ).await.expect("Pinned post should be created.");

    let post_vec = ssr::get_post_vec_by_forum_name(
        forum_name,
        None,
        SortType::Post(PostSortType::Hot),
        partial_load_num_post as i64,
        0,
        &db_pool,
    ).await?;

    assert_eq!(post_vec.len(), partial_load_num_post);
    assert_eq!(post_vec[0], pinned_post);

    user.admin_role = AdminRole::Admin;
    let rule = forum_management::ssr::add_rule(None, 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");
    let moderated_post = moderate_post(
        post_vec.first().expect("First post should be accessible.").post_id,
        rule.rule_id,
        "test",
        &user,
        &db_pool,
    ).await.expect("Post should be moderated.");

    let post_vec = ssr::get_post_vec_by_forum_name(
        forum_name,
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
async fn test_create_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let forum = forum::ssr::create_forum("a", "forum", false, &user, &db_pool).await?;

    let post_1_title = "1";
    let post_1_body = "test";
    let post_1 = create_post(&forum.forum_name, post_1_title, post_1_body, None, false, false, false, None, &user, &db_pool).await.expect("Should be able to create post 1.");

    assert_eq!(post_1.title, post_1_title);
    assert_eq!(post_1.body, post_1_body);
    assert_eq!(post_1.markdown_body, None);
    assert_eq!(post_1.is_nsfw, false);
    assert_eq!(post_1.is_spoiler, false);
    assert_eq!(post_1.category_id, None);
    assert_eq!(post_1.is_edited, false);
    assert_eq!(post_1.meta_post_id, None);
    assert_eq!(post_1.forum_id, forum.forum_id);
    assert_eq!(post_1.forum_name, forum.forum_name);
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
        create_post(&forum.forum_name, post_1_title, post_1_body, None, false, false, true, None, &user, &db_pool).await,
        Err(AppError::InsufficientPrivileges),
    );

    let user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded.");
    let post_2_title = "1";
    let post_2_body = "test";
    let post_2_markdown_body = "test";
    let post_2 = create_post(&forum.forum_name, post_2_title, post_2_body, Some(post_2_markdown_body), false, false, true, None, &user, &db_pool).await.expect("Should be able to create post 2.");

    assert_eq!(post_2.title, post_2_title);
    assert_eq!(post_2.body, post_2_body);
    assert_eq!(post_2.markdown_body, Some(String::from(post_2_markdown_body)));
    assert_eq!(post_2.is_nsfw, false);
    assert_eq!(post_2.is_spoiler, false);
    assert_eq!(post_2.category_id, None);
    assert_eq!(post_2.is_edited, false);
    assert_eq!(post_2.meta_post_id, None);
    assert_eq!(post_2.forum_id, forum.forum_id);
    assert_eq!(post_2.forum_name, forum.forum_name);
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

    let post_1_with_info = get_post_with_info_by_id(post_1.post_id, None, &db_pool).await.expect("Should be able to load post 1.");

    assert_eq!(post_1_with_info.post, post_1);
    assert_eq!(post_1_with_info.vote, None);

    let post_2_with_info = get_post_with_info_by_id(post_2.post_id, None, &db_pool).await.expect("Should be able to load post 2.");

    assert_eq!(post_2_with_info.post, post_2);
    assert_eq!(post_2_with_info.vote, None);

    Ok(())
}

#[tokio::test]
async fn test_update_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &user,
        &db_pool,
    ).await?;

    let post = post::ssr::create_post(
        forum_name,
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

    Ok(())
}

#[tokio::test]
async fn increment_post_comment_count() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_, post) = create_forum_with_post("forum", &user, &db_pool).await;
    assert_eq!(post.num_comments, 0);

    let _comment = create_comment(post.post_id, None, "a", None, false, &user, &db_pool).await.expect("Should create comment.");

    let post = get_post_by_id(post.post_id, &db_pool).await.expect("Should get post.");
    assert_eq!(post.num_comments, 1);

    Ok(())
}

#[tokio::test]
async fn test_update_post_scores() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_, post) = create_forum_with_post("forum", &user, &db_pool).await;
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
    let user = create_test_user(&db_pool).await;

    let (_, post) = create_forum_with_post("forum", &user, &db_pool).await;

    let mut rng = rand::thread_rng();

    // wait to have a meaningful impact of elapsed time on the score
    tokio::time::sleep(Duration::from_secs(2)).await;

    set_post_score(post.post_id, rng.gen_range(-100..101), &db_pool).await?;

    let post_with_vote = post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool).await?;

    test_post_score(&post_with_vote.post);
    Ok(())
}
