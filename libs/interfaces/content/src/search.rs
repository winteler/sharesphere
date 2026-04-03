use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_core_common::checks::{check_sphere_name, check_sphere_name_with_options, check_string_length},
    sharesphere_core_common::constants::{COMMENT_BATCH_SIZE, MAX_SEARCH_QUERY_LENGTH, POST_BATCH_SIZE},
    sharesphere_core_common::db_utils::ssr::get_db_pool,
    sharesphere_core_content::search::*,
    sharesphere_core_user::auth::ssr::get_user,
};

use sharesphere_core_common::common::SphereHeader;
use sharesphere_core_common::errors::AppError;
use sharesphere_core_content::comment::CommentWithContext;
use sharesphere_core_content::post::PostWithSphereInfo;

#[server]
pub async fn get_matching_sphere_header_vec(
    sphere_prefix: String,
) -> Result<Vec<SphereHeader>, AppError> {
    check_sphere_name_with_options(&sphere_prefix, false)?;
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_matching_sphere_header_vec(
        &sphere_prefix,
        10,
        &db_pool
    ).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn search_spheres(
    search_query: String,
    load_count: usize,
    num_already_loaded: usize,
) -> Result<Vec<SphereHeader>, AppError> {
    check_string_length(&search_query, "Sphere search", MAX_SEARCH_QUERY_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let show_nsfw = get_user().await.unwrap_or(None).map(|user| user.show_nsfw).unwrap_or_default();
    let sphere_header_vec = ssr::search_spheres(&search_query, show_nsfw, load_count as i64, num_already_loaded as i64, &db_pool).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn search_posts(
    search_query: String,
    sphere_name: Option<String>,
    show_spoilers: bool,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, AppError> {
    if let Some(sphere_name) = &sphere_name {
        check_sphere_name(sphere_name)?;
    }
    check_string_length(&search_query, "Search query", MAX_SEARCH_QUERY_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let show_nsfw = get_user().await.unwrap_or(None).map(|user| user.show_nsfw).unwrap_or_default();
    let post_vec = ssr::search_posts(
        &search_query,
        sphere_name.as_deref(),
        show_spoilers,
        show_nsfw,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool
    ).await?;
    Ok(post_vec)
}

#[server]
pub async fn search_comments(
    search_query: String,
    sphere_name: Option<String>,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithContext>, AppError> {
    if let Some(sphere_name) = &sphere_name {
        check_sphere_name(sphere_name)?;
    }
    check_string_length(&search_query, "Search query", MAX_SEARCH_QUERY_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let comment_vec = ssr::search_comments(&search_query, sphere_name.as_deref(), COMMENT_BATCH_SIZE, num_already_loaded as i64, &db_pool).await?;
    Ok(comment_vec)
}