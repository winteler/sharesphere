use leptos::prelude::*;
use leptos::server_fn::codec::{MultipartData, MultipartFormData};
use sharesphere_core_common::errors::AppError;

#[cfg(feature = "ssr")]
use {
    std::path::Path,
    sharesphere_auth::{
        auth::{ssr::check_user, ssr::reload_user},
        session::ssr::get_db_pool,
    },
    sharesphere_core_common::checks::{check_sphere_name, check_username},
    ssr::{
        MAX_BANNER_SIZE, MAX_ICON_SIZE, OBJECT_CONTAINER_URL_ENV, SphereImageType
    }
};

#[server]
pub async fn get_sphere_ban_vec(
    sphere_name: String,
    username_prefix: String,
) -> Result<Vec<UserBan>, AppError> {
    check_sphere_name(&sphere_name)?;
    check_username(&username_prefix, true)?;
    let db_pool = get_db_pool()?;
    let ban_vec = ssr::get_sphere_ban_vec(&sphere_name, &username_prefix, &db_pool).await?;
    Ok(ban_vec)
}

#[server]
pub async fn remove_user_ban(
    ban_id: i64
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;
    let deleted_user_ban = ssr::remove_user_ban(ban_id, &user, &db_pool).await?;
    reload_user(deleted_user_ban.user_id)?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_sphere_icon(
    data: MultipartData,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let image_type = SphereImageType::ICON;
    let object_container_url = env::var(OBJECT_CONTAINER_URL_ENV)?;
    let bucket_name = image_type.get_bucket_name()?;
    let object_store = ssr::get_object_store(image_type)?;
    let (sphere_name, file_name) = ssr::store_sphere_image(data, MAX_ICON_SIZE, &object_store, &user).await?;
    // Clear previous image if it exists
    if let Err(e) = ssr::delete_sphere_image(&sphere_name, image_type, &object_store, &user, &db_pool).await {
        log::warn!("Failed to delete Sphere icon: {:?}", e);
    }

    let icon_url = file_name.map(|file_name| {
        Path::new(&object_container_url)
            .join(bucket_name)
            .join(&file_name)
            .to_string_lossy()
            .to_string()
    });
    ssr::set_sphere_icon_url(&sphere_name.clone(), icon_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_sphere_banner(
    data: MultipartData,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let image_type = SphereImageType::BANNER;
    let object_container_url = env::var(OBJECT_CONTAINER_URL_ENV)?;
    let bucket_name = image_type.get_bucket_name()?;
    let object_store = ssr::get_object_store(image_type)?;
    let (sphere_name, file_name) = ssr::store_sphere_image(data, MAX_BANNER_SIZE, &object_store, &user).await?;
    // Clear previous image if it exists
    if let Err(e) = ssr::delete_sphere_image(&sphere_name, image_type, &object_store, &user, &db_pool).await  {
        log::warn!("Failed to delete Sphere banner: {:?}", e);
    }
    let banner_url = file_name.map(|file_name| {
        Path::new(&object_container_url)
            .join(bucket_name)
            .join(&file_name)
            .to_string_lossy()
            .to_string()
    });
    ssr::set_sphere_banner_url(&sphere_name.clone(), banner_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}