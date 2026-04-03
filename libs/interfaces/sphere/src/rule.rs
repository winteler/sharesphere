use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    sharesphere_core_common::checks::{check_sphere_name, check_string_length},
    sharesphere_core_common::constants::{MAX_MOD_MESSAGE_LENGTH, MAX_TITLE_LENGTH},
    sharesphere_core_common::db_utils::ssr::get_db_pool,
    sharesphere_core_common::editor::ssr::get_html_and_markdown_strings,
    sharesphere_core_sphere::rule::*,
    sharesphere_core_user::auth::ssr::check_user,
};

use sharesphere_core_common::common::Rule;
use sharesphere_core_common::errors::AppError;

#[server]
pub async fn get_rule_by_id(
    rule_id: i64
) -> Result<Rule, AppError> {
    let db_pool = get_db_pool()?;
    let rule = ssr::load_rule_by_id(rule_id, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn get_rule_vec(
    sphere_name: Option<String>
) -> Result<Vec<Rule>, AppError> {
    if let Some(sphere_name) = &sphere_name {
        check_sphere_name(&sphere_name)?;
    }
    let db_pool = get_db_pool()?;
    let rule_vec = ssr::get_rule_vec(sphere_name.as_deref(), &db_pool).await?;
    Ok(rule_vec)
}

#[server]
pub async fn add_rule(
    sphere_name: String,
    priority: i16,
    title: String,
    description: String,
    is_markdown: bool,
) -> Result<Rule, AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&title, "Title", MAX_TITLE_LENGTH as usize, false)?;
    check_string_length(&description, "Description", MAX_MOD_MESSAGE_LENGTH, true)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let (description, markdown_description) = get_html_and_markdown_strings(description, is_markdown).await?;

    let rule = ssr::add_rule(
        sphere_name.as_ref(),
        priority,
        &title,
        &description,
        markdown_description.as_deref(),
        &user,
        &db_pool
    ).await?;

    Ok(rule)
}

#[server]
pub async fn update_rule(
    sphere_name: String,
    current_priority: i16,
    priority: i16,
    title: String,
    description: String,
    is_markdown: bool,
) -> Result<Rule, AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&title, "Title", MAX_TITLE_LENGTH as usize, false)?;
    check_string_length(&description, "Description", MAX_MOD_MESSAGE_LENGTH, true)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let (description, markdown_description) = get_html_and_markdown_strings(description, is_markdown).await?;

    let rule = ssr::update_rule(
        sphere_name.as_ref(),
        current_priority,
        priority,
        &title,
        &description,
        markdown_description.as_deref(),
        &user,
        &db_pool
    ).await?;

    Ok(rule)
}

#[server]
pub async fn remove_rule(
    sphere_name: String,
    priority: i16,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::remove_rule(sphere_name.as_ref(), priority, &user, &db_pool).await?;
    Ok(())
}