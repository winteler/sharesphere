use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::{check_user},
        session::ssr::get_db_pool,
    },
    sharesphere_utils::constants::{MAX_CATEGORY_DESCRIPTION_LENGTH, MAX_CATEGORY_NAME_LENGTH},
    sharesphere_utils::checks::{check_sphere_name, check_string_length},
};

#[server]
pub async fn get_sphere_category_vec(
    sphere_name: String,
) -> Result<Vec<SphereCategory>, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let sphere_category_vec = ssr::get_sphere_category_vec(&sphere_name, &db_pool).await?;
    Ok(sphere_category_vec)
}

#[server]
pub async fn set_sphere_category(
    sphere_name: String,
    category_name: String,
    category_color: Color,
    description: String,
    is_active: bool,
) -> Result<SphereCategory, AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&category_name, "Category name", MAX_CATEGORY_NAME_LENGTH, false)?;
    check_string_length(&description, "Category description", MAX_CATEGORY_DESCRIPTION_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let sphere_category = ssr::set_sphere_category(&sphere_name, &category_name, category_color, &description, is_active, &user, &db_pool).await?;
    Ok(sphere_category)
}

#[server]
pub async fn delete_sphere_category(
    sphere_name: String,
    category_name: String,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&category_name, "Category name", MAX_CATEGORY_NAME_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::delete_sphere_category(&sphere_name, &category_name, &user, &db_pool).await?;
    Ok(())
}