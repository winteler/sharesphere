use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_core_common::checks::check_username,
    sharesphere_core_common::constants::{COMMENT_BATCH_SIZE, POST_BATCH_SIZE},
    sharesphere_core_common::db_utils::ssr::get_db_pool,
    sharesphere_core_content::profile::*,
};

use sharesphere_core_common::errors::AppError;
use sharesphere_core_content::comment::CommentWithContext;
use sharesphere_core_content::post::PostWithSphereInfo;
use sharesphere_core_content::ranking::SortType;

#[server]
pub async fn get_user_post_vec(
    username: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, AppError> {
    check_username(&username, false)?;
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_user_post_vec(
        &username,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_user_comment_vec(
    username: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithContext>, AppError> {
    check_username(&username, false)?;
    let db_pool = get_db_pool()?;

    let comment_vec = ssr::get_user_comment_vec(
        &username,
        sort_type,
        COMMENT_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(comment_vec)
}