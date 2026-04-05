use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_core_common::checks::check_sphere_name,
    sharesphere_core_common::constants::POST_BATCH_SIZE,
    sharesphere_core_common::db_utils::ssr::get_db_pool,
    sharesphere_core_common::routes::get_post_path,
    sharesphere_core_common::{
        editor::clear_newlines,
        editor::ssr::get_html_and_markdown_strings,
    },
    sharesphere_core_content::post::*,
    sharesphere_core_content::ranking::{ssr::vote_on_content, VoteValue},
    sharesphere_core_user::auth::{ssr::check_user, ssr::get_user},
    validator::Validate,
};

use sharesphere_core_common::errors::AppError;
use sharesphere_core_content::filter::SphereCategoryFilter;
use sharesphere_core_content::post::{Post, PostDataInputs, PostInheritedAttributes, PostLocation, PostWithInfo, PostWithSphereInfo};
use sharesphere_core_content::ranking::SortType;

#[server]
pub async fn get_post_with_info_by_id(post_id: i64) -> Result<PostWithInfo, AppError> {
    let db_pool = get_db_pool()?;
    let user = get_user().await?;
    Ok(ssr::get_post_with_info_by_id(post_id, user.as_ref(), &db_pool).await?)
}

#[server]
pub async fn get_post_inherited_attributes(post_id: i64) -> Result<PostInheritedAttributes, AppError> {
    let db_pool = get_db_pool()?;
    Ok(ssr::get_post_inherited_attributes(post_id, &db_pool).await?)
}

#[server]
pub async fn get_sorted_post_vec(
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, AppError> {
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_sorted_post_vec(
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_subscribed_post_vec(
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let post_vec = ssr::get_subscribed_post_vec(
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &user,
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_post_vec_by_sphere_name(
    sphere_name: String,
    sphere_category_set: SphereCategoryFilter,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, AppError> {
    check_sphere_name(&sphere_name)?;
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;
    let post_vec = ssr::get_post_vec_by_sphere_name(
        sphere_name.as_str(),
        sphere_category_set,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
        &db_pool,
    ).await?;
    Ok(post_vec)
}

#[server]
pub async fn get_post_vec_by_satellite_id(
    satellite_id: i64,
    sphere_category_id: Option<i64>,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<Post>, AppError> {
    let user = get_user().await.unwrap_or(None);
    let db_pool = get_db_pool()?;
    let post_vec = ssr::get_post_vec_by_satellite_id(
        satellite_id,
        sphere_category_id,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        user.as_ref(),
        &db_pool,
    )
        .await?;
    Ok(post_vec)
}

#[server]
pub async fn create_post(
    post_location: PostLocation,
    post_inputs: PostDataInputs
) -> Result<(), AppError> {
    post_location.validate()?;
    post_inputs.validate()?;

    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = get_html_and_markdown_strings(post_inputs.body, post_inputs.is_markdown).await?;

    let link = ssr::process_embed_link(post_inputs.embed_type, post_inputs.link).await;

    let post = ssr::create_post(
        post_location.sphere.as_str(),
        post_location.satellite_id,
        clear_newlines(post_inputs.title, true).as_str(),
        body.as_str(),
        markdown_body.as_deref(),
        link,
        post_inputs.post_tags,
        &user,
        &db_pool,
    ).await?;

    let _vote = vote_on_content(VoteValue::Up, post.post_id, None, None, &user, &db_pool).await?;

    log::trace!("Created post with id: {}", post.post_id);
    let new_post_path = get_post_path(&post_location.sphere, post_location.satellite_id, post.post_id);

    leptos_axum::redirect(new_post_path.as_str());
    Ok(())
}

#[server]
pub async fn edit_post(
    post_id: i64,
    post_inputs: PostDataInputs,
) -> Result<Post, AppError> {
    post_inputs.validate()?;
    log::trace!("Edit post {post_id}, title = {}", post_inputs.title);
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (body, markdown_body) = get_html_and_markdown_strings(
        post_inputs.body,
        post_inputs.is_markdown,
    ).await?;

    let link = ssr::process_embed_link(post_inputs.embed_type, post_inputs.link).await;

    let post = ssr::update_post(
        post_id,
        post_inputs.title.as_str(),
        body.as_str(),
        markdown_body.as_deref(),
        link,
        post_inputs.post_tags,
        &user,
        &db_pool,
    ).await?;

    log::trace!("Updated post with id: {}", post.post_id);
    Ok(post)
}

#[server]
pub async fn delete_post(
    post_id: i64,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::delete_post(post_id, &user, &db_pool).await?;

    Ok(())
}