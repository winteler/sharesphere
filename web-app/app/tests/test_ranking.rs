use leptos::ServerFnError;

use app::{post, ranking};
use app::ranking::VoteValue;

pub use crate::common::*;
use crate::data_factory::create_forum_with_post_and_comment;

mod common;
mod data_factory;

#[tokio::test]
async fn test_vote_on_content_post() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_, post, _) = create_forum_with_post_and_comment("forum", &user, &db_pool).await;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&user), &db_pool)
            .await?;
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

    Ok(())
}