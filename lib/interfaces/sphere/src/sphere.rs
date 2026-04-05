use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_core_common::checks::{check_sphere_name, check_string_length},
    sharesphere_core_common::constants::MAX_SPHERE_DESCRIPTION_LENGTH,
    sharesphere_core_common::db_utils::ssr::get_db_pool,
    sharesphere_core_common::routes::get_sphere_path,
    sharesphere_core_sphere::sphere::*,
    sharesphere_core_user::auth::ssr::{check_user, get_user, reload_user},
};

use sharesphere_core_common::common::SphereHeader;
use sharesphere_core_common::errors::AppError;
use sharesphere_core_sphere::sphere::{Sphere, SphereWithUserInfo};

#[server]
pub async fn is_sphere_available(sphere_name: String) -> Result<bool, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let sphere_existence = ssr::is_sphere_available(&sphere_name, &db_pool).await?;
    Ok(sphere_existence)
}

#[server]
pub async fn get_sphere_by_name(sphere_name: String) -> Result<Sphere, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let sphere = ssr::get_sphere_by_name(&sphere_name, &db_pool).await?;
    Ok(sphere)
}

#[server]
pub async fn get_subscribed_sphere_headers() -> Result<Vec<SphereHeader>, AppError> {
    let db_pool = get_db_pool()?;
    match get_user().await {
        Ok(Some(user)) => {
            let sphere_header_vec = ssr::get_subscribed_sphere_headers(user.user_id, &db_pool).await?;
            Ok(sphere_header_vec)
        }
        _ => Ok(Vec::new()),
    }
}

#[server]
pub async fn get_popular_sphere_headers() -> Result<Vec<SphereHeader>, AppError> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_popular_sphere_headers(20, &db_pool).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn get_sphere_with_user_info(
    sphere_name: String,
) -> Result<SphereWithUserInfo, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };

    let sphere_content = ssr::get_sphere_with_user_info(sphere_name.as_str(), user_id, &db_pool).await?;

    Ok(sphere_content)
}

#[server]
pub async fn create_sphere(
    sphere_name: String,
    description: String,
    is_nsfw: bool,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&description, "Sphere description", MAX_SPHERE_DESCRIPTION_LENGTH, false)?;
    log::trace!("Create Sphere '{sphere_name}', {description}, {is_nsfw}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let new_sphere_path = get_sphere_path(&sphere_name);

    let sphere = ssr::create_sphere(
        sphere_name.as_str(),
        description.as_str(),
        is_nsfw,
        &user,
        &db_pool,
    ).await?;

    ssr::subscribe(sphere.sphere_id, user.user_id, &db_pool).await?;

    reload_user(user.user_id)?;

    // Redirect to the new sphere
    leptos_axum::redirect(&new_sphere_path);
    Ok(())
}

#[server]
pub async fn update_sphere_description(
    sphere_name: String,
    description: String,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&description, "Sphere description", MAX_SPHERE_DESCRIPTION_LENGTH, false)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::update_sphere_description(&sphere_name, &description, &user, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn subscribe(sphere_id: i64) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::subscribe(sphere_id, user.user_id, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn unsubscribe(sphere_id: i64) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::unsubscribe(sphere_id, user.user_id, &db_pool).await?;
    Ok(())
}
