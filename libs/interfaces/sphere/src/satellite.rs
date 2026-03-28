use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
    sharesphere_core_common::checks::{check_satellite_name, check_sphere_name, check_string_length},
    sharesphere_core_common::constants::MAX_CONTENT_LENGTH,
    sharesphere_core_common::editor::ssr::get_html_and_markdown_strings,
    crate::satellite::ssr::get_active_satellite_vec_by_sphere_name,
};

#[server]
pub async fn get_satellite_by_id(
    satellite_id: i64,
) -> Result<Satellite, AppError> {
    let db_pool = get_db_pool()?;
    let satellite = ssr::get_satellite_by_id(satellite_id, &db_pool).await?;
    Ok(satellite)
}

#[server]
pub async fn get_satellite_vec_by_sphere_name(
    sphere_name: String,
    only_active: bool,
) -> Result<Vec<Satellite>, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let satellite_vec = match only_active {
        true => get_active_satellite_vec_by_sphere_name(&sphere_name, &db_pool).await?,
        false => ssr::get_satellite_vec_by_sphere_name(&sphere_name, &db_pool).await?,
    };
    Ok(satellite_vec)
}

#[server]
pub async fn create_satellite(
    sphere_name: String,
    satellite_name: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> Result<Satellite, AppError> {
    check_sphere_name(&sphere_name)?;
    check_satellite_name(&satellite_name)?;
    check_string_length(&body, "Satellite body", MAX_CONTENT_LENGTH as usize, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let (body, markdown_body) = get_html_and_markdown_strings(body, is_markdown).await?;

    let satellite = ssr::create_satellite(
        &sphere_name,
        &satellite_name,
        &body,
        markdown_body.as_deref(),
        is_nsfw,
        is_spoiler,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

#[server]
pub async fn update_satellite(
    satellite_id: i64,
    satellite_name: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> Result<Satellite, AppError> {
    check_satellite_name(&satellite_name)?;
    check_string_length(&body, "Satellite body", MAX_CONTENT_LENGTH as usize, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let (body, markdown_body) = get_html_and_markdown_strings(body, is_markdown).await?;

    let satellite = ssr::update_satellite(
        satellite_id,
        &satellite_name,
        &body,
        markdown_body.as_deref(),
        is_nsfw,
        is_spoiler,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

#[server]
pub async fn disable_satellite(
    satellite_id: i64,
) -> Result<Satellite, AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let satellite = ssr::disable_satellite(
        satellite_id,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}