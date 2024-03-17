use leptos::ServerFnError;
use rand::Rng;

use app::{forum, post};

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

    let post_with_vote =
        post::ssr::get_post_with_vote_by_id(post.post_id, Some(test_user.user_id), db_pool).await?;

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

    Ok(())
}
