use app::{post, ranking};
use app::errors::AppError;
use app::ranking::VoteValue;

use crate::common::*;
use crate::data_factory::{create_forum_with_post, create_forum_with_post_and_comment};
use crate::utils::{get_comment_by_id, get_user_comment_vote};

mod common;
mod data_factory;
mod utils;

#[tokio::test]
async fn test_vote_on_content_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_, post) = create_forum_with_post("forum", &user, &db_pool).await;

    let post_with_vote = post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool).await?;
    assert!(post_with_vote.vote.is_none());

    let vote_value = VoteValue::Up;
    ranking::ssr::vote_on_content(
        vote_value,
        post.post_id,
        None,
        None,
        &user,
        &db_pool,
    ).await.expect("Upvote should be created.");

    let post_with_vote = post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool).await?;
    let vote = post_with_vote.vote.expect("Vote should be Some");
    assert_eq!(vote.value, vote_value);
    assert_eq!(vote.user_id, user.user_id);
    assert_eq!(vote.post_id, post.post_id);
    assert_eq!(vote.comment_id, None);
    assert_eq!(post.score + 1, post_with_vote.post.score);

    // repeating vote just returns same result
    let repeat_vote = ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        Some(vote.vote_id),
        &user,
        &db_pool,
    ).await.expect("Repeat vote should be ok.").expect("Vote should be Some.");
    assert_eq!(repeat_vote, vote);

    // assert error when existing vote is not referenced
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        None,
        &user,
        &db_pool,
    )
        .await
        .is_err());

    ranking::ssr::vote_on_content(
        VoteValue::Down,
        post.post_id,
        None,
        Some(vote.vote_id),
        &user,
        &db_pool,
    ).await.expect("Downvote should be created.");

    let post_with_vote = post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool).await?;
    let vote = post_with_vote.vote.expect("Post should have vote");
    assert_eq!(vote.value, VoteValue::Down);
    assert_eq!(vote.user_id, user.user_id);
    assert_eq!(vote.post_id, post.post_id);
    assert_eq!(vote.comment_id, None);
    assert_eq!(post.score - 1, post_with_vote.post.score);

    ranking::ssr::vote_on_content(
        VoteValue::None,
        post.post_id,
        None,
        Some(vote.vote_id),
        &user,
        &db_pool,
    ).await.expect("Vote should be deleted.");

    let post_with_vote = post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool).await?;
    assert_eq!(post_with_vote.vote, None);
    assert_eq!(post.score, post_with_vote.post.score);

    Ok(())
}

#[tokio::test]
async fn test_vote_on_content_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_, _, init_comment) = create_forum_with_post_and_comment("forum", &user, &db_pool).await;

    let comment = get_comment_by_id(init_comment.comment_id, &db_pool).await?;

    let vote_value = VoteValue::Up;
    ranking::ssr::vote_on_content(
        vote_value,
        comment.post_id,
        Some(comment.comment_id),
        None,
        &user,
        &db_pool,
    ).await.expect("Upvote should be created.");

    let comment = get_comment_by_id(comment.comment_id, &db_pool).await?;
    let vote = get_user_comment_vote(&comment, user.user_id, &db_pool).await?;
    assert_eq!(vote.value, vote_value);
    assert_eq!(vote.user_id, user.user_id);
    assert_eq!(vote.post_id, comment.post_id);
    assert_eq!(vote.comment_id, Some(comment.comment_id));
    assert_eq!(init_comment.score + 1, comment.score);

    // repeating vote just returns same result
    let repeat_vote = ranking::ssr::vote_on_content(
        VoteValue::Up,
        comment.post_id,
        Some(comment.comment_id),
        Some(vote.vote_id),
        &user,
        &db_pool,
    ).await.expect("Repeat vote should be ok.").expect("Vote should be Some.");
    assert_eq!(repeat_vote, vote);

    // assert error when existing vote is not referenced
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        comment.post_id,
        Some(comment.comment_id),
        None,
        &user,
        &db_pool,
    )
        .await
        .is_err());

    ranking::ssr::vote_on_content(
        VoteValue::Down,
        comment.post_id,
        Some(comment.comment_id),
        Some(vote.vote_id),
        &user,
        &db_pool,
    ).await.expect("Downvote should be created.");

    let comment = get_comment_by_id(comment.comment_id, &db_pool).await?;
    let vote = get_user_comment_vote(&comment, user.user_id, &db_pool).await?;
    assert_eq!(vote.value, VoteValue::Down);
    assert_eq!(vote.user_id, user.user_id);
    assert_eq!(vote.post_id, comment.post_id);
    assert_eq!(vote.comment_id, Some(comment.comment_id));
    assert_eq!(init_comment.score - 1, comment.score);

    ranking::ssr::vote_on_content(
        VoteValue::None,
        comment.post_id,
        Some(comment.comment_id),
        Some(vote.vote_id),
        &user,
        &db_pool,
    ).await.expect("Vote should be deleted.");

    let comment = get_comment_by_id(comment.comment_id, &db_pool).await?;
    assert_eq!(get_user_comment_vote(&comment, user.user_id, &db_pool).await, Err(AppError::NotFound));
    assert_eq!(init_comment.score, comment.score);

    Ok(())
}