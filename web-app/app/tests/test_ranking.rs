use leptos::ServerFnError;

use app::{forum, post, ranking};
use app::ranking::{VoteInfo, VoteValue};

pub use crate::common::*;

mod common;
#[tokio::test]
async fn test_post_votes() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
        db_pool.clone(),
    ).await?;

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        None,
        false,
        None,
        &test_user,
        &db_pool,
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), &db_pool)
            .await?;
    assert!(post_with_vote.vote.is_none());

    let vote_value = VoteValue::Up;
    ranking::ssr::vote_on_content(
        vote_value,
        post.post_id,
        None,
        None,
        &test_user,
        &db_pool,
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), &db_pool)
            .await?;
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
        &db_pool,
    )
        .await
        .is_err());

    // assert error when existing vote is not referenced
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        None,
        &test_user,
        &db_pool,
    )
        .await
        .is_err());

    ranking::ssr::vote_on_content(
        VoteValue::Down,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        &db_pool,
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), &db_pool)
            .await?;
    let vote = post_with_vote.vote.expect("Post should have vote");
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
        &db_pool,
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), &db_pool)
            .await?;
    assert!(post_with_vote.vote.is_none());

    Ok(())
}