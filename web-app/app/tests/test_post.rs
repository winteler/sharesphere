use leptos::ServerFnError;
use rand::Rng;

use app::{forum, post, ranking};
use app::ranking::{VoteInfo, VoteValue};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

#[tokio::test]
async fn test_post_scores() -> Result<(), ServerFnError> {
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

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        false,
        None,
        &test_user,
        db_pool.clone(),
    )
    .await?;

    let mut rng = rand::thread_rng();

    set_post_score(post.post_id, rng.gen_range(-100..101), db_pool.clone()).await?;

    let post_with_vote = post::ssr::get_post_with_vote_by_id(post.post_id, Some(test_user.user_id), db_pool.clone()).await?;

    let post = post_with_vote.post;
    let post_num_days_old = (post
        .scoring_timestamp
        .signed_duration_since(post.create_timestamp)
        .num_seconds() as f32)
        / 86400.0;
    let expected_recommended_score =
        (post.score as f32) * f32::powf(2.0, 3.0 * (2.0 - post_num_days_old));
    let expected_trending_score =
        (post.score as f32) * f32::powf(2.0, 8.0 * (1.0 - post_num_days_old));

    assert_eq!(post.recommended_score, expected_recommended_score);
    assert_eq!(post.trending_score, expected_trending_score);
    assert_eq!(post_with_vote.vote, None);

    Ok(())
}

#[tokio::test]
async fn test_post_votes() -> Result<(), ServerFnError> {
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

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        false,
        None,
        &test_user,
        db_pool.clone(),
    )
        .await?;

    let post_with_vote = post::ssr::get_post_with_vote_by_id(post.post_id, Some(test_user.user_id), db_pool.clone()).await?;
    assert!(post_with_vote.vote.is_none());

    let vote_value = VoteValue::Up;
    ranking::ssr::vote_on_content(
        vote_value,
        post.post_id,
        None,
        None,
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_with_vote = post::ssr::get_post_with_vote_by_id(post.post_id, Some(test_user.user_id), db_pool.clone()).await?;
    assert!(post_with_vote.vote.is_some());
    let vote = post_with_vote.vote.unwrap();
    assert_eq!(vote.value, vote_value);
    assert_eq!(vote.user_id, test_user.user_id);
    assert_eq!(vote.post_id, post.post_id);
    assert_eq!(vote.comment_id, None);

    // assert error when repeating vote
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        db_pool.clone(),
    ).await.is_err());

    // assert error when existing vote is not referenced
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        None,
        &test_user,
        db_pool.clone(),
    ).await.is_err());

    ranking::ssr::vote_on_content(
        VoteValue::Down,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_with_vote = post::ssr::get_post_with_vote_by_id(post.post_id, Some(test_user.user_id), db_pool.clone()).await?;
    let vote = post_with_vote.vote.expect("Vote not found");
    assert_eq!(vote.value, VoteValue::Down);
    assert_eq!(vote.user_id, test_user.user_id);
    assert_eq!(vote.post_id, post.post_id);
    assert_eq!(vote.comment_id, None);

    ranking::ssr::vote_on_content(
        VoteValue::None,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_with_vote = post::ssr::get_post_with_vote_by_id(post.post_id, Some(test_user.user_id), db_pool.clone()).await?;
    assert!(post_with_vote.vote.is_none());

    Ok(())
}
