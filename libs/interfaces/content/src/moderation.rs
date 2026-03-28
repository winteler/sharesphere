use leptos::prelude::*;
use sharesphere_core_common::errors::AppError;

#[cfg(feature = "ssr")]
use {
    crate::{
        comment::ssr::{get_comment_by_id, get_comment_sphere},
        notification::{ssr::create_notification, NotificationType},
        post::ssr::get_post_by_id,
        rule::ssr::load_rule_by_id,
    },
    sharesphere_auth::{
        auth::ssr::{check_user, reload_user},
        session::ssr::get_db_pool,
    },
    sharesphere_core_common::{
        checks::check_string_length,
        constants::MAX_MOD_MESSAGE_LENGTH,
    }
};

#[server]
pub async fn get_moderation_info(
    post_id: i64,
    comment_id: Option<i64>,
) -> Result<ModerationInfo, AppError> {
    let db_pool = get_db_pool()?;
    let (rule_id, content) = match comment_id {
        Some(comment_id) => {
            let comment = get_comment_by_id(comment_id, &db_pool).await?;
            (comment.infringed_rule_id, Content::Comment(comment))
        },
        None => {
            let post = get_post_by_id(post_id, &db_pool).await?;
            (post.infringed_rule_id, Content::Post(post))
        },
    };
    let rule = match rule_id {
        Some(rule_id) => load_rule_by_id(rule_id, &db_pool).await,
        None => Err(AppError::InternalServerError(String::from("Content is not moderated, cannot find moderation info.")))
    }?;

    Ok(ModerationInfo {
        rule,
        content,
    })
}

/// Function to moderate a post and optionally ban its author
///
/// The ban is performed for the sphere of the given post and the duration is given by `ban_num_days`.
/// If `ban_num_days == None`, the duration of the ban is permanent.
#[server]
pub async fn moderate_post(
    post_id: i64,
    rule_id: i64,
    moderator_message: String,
    ban_duration_days: Option<usize>,
) -> Result<Post, AppError> {
    log::debug!("Moderate post {post_id}, ban duration = {ban_duration_days:?}");
    check_string_length(&moderator_message, "Moderator message", MAX_MOD_MESSAGE_LENGTH, true)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let post = ssr::moderate_post(
        post_id,
        rule_id,
        moderator_message.as_str(),
        &user,
        &db_pool
    ).await?;

    ssr::ban_user_from_sphere(
        post.creator_id,
        post.sphere_id,
        post.post_id,
        None,
        rule_id,
        &user,
        ban_duration_days,
        &db_pool,
    ).await?;

    reload_user(post.creator_id)?;

    create_notification(post.post_id, None, None, user.user_id, NotificationType::Moderation, &db_pool).await?;

    Ok(post)
}

/// Function to moderate a comment and optionally ban its author
///
/// The ban is performed for the sphere of the given comment and the duration is given by `ban_num_days`.
/// If `ban_num_days == None`, the duration of the ban is permanent.
#[server]
pub async fn moderate_comment(
    comment_id: i64,
    rule_id: i64,
    moderator_message: String,
    ban_duration_days: Option<usize>,
) -> Result<Comment, AppError> {
    log::trace!("Moderate comment {comment_id}");
    check_string_length(&moderator_message, "Moderation message", MAX_MOD_MESSAGE_LENGTH, false)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let comment = ssr::moderate_comment(
        comment_id,
        rule_id,
        moderator_message.as_str(),
        &user,
        &db_pool
    ).await?;

    let sphere = get_comment_sphere(comment_id, &db_pool).await?;

    ssr::ban_user_from_sphere(
        comment.creator_id,
        sphere.sphere_id,
        comment.post_id,
        Some(comment.comment_id),
        rule_id,
        &user,
        ban_duration_days,
        &db_pool
    ).await?;

    reload_user(comment.creator_id)?;

    create_notification(comment.post_id, Some(comment.comment_id), Some(comment.comment_id), user.user_id, NotificationType::Moderation, &db_pool).await?;

    Ok(comment)
}