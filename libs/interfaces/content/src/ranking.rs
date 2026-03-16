use leptos::prelude::*;
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
};

#[server]
pub async fn vote_on_content(
    vote_value: VoteValue,
    post_id: i64,
    comment_id: Option<i64>,
    vote_id: Option<i64>,
) -> Result<Option<Vote>, AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let vote = ssr::vote_on_content(
        vote_value,
        post_id,
        comment_id,
        vote_id,
        &user,
        &db_pool,
    )
        .await?;

    Ok(vote)
}