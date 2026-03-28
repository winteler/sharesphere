use leptos::prelude::*;
use leptos_fluent::move_tr;
use sharesphere_core_common::errors::AppError;

#[cfg(feature = "ssr")]
use {
    crate::ranking::{ssr::vote_on_content, VoteValue},
    crate::notification::{ssr::create_notification, NotificationType},
    sharesphere_auth::{
        auth::{get_user, ssr::check_user},
        session::ssr::get_db_pool,
    },
    sharesphere_core_common::{
        checks::check_string_length,
        constants::MAX_CONTENT_LENGTH,
        editor::ssr::get_html_and_markdown_strings,
    },
};

#[server]
pub async fn get_post_comment_tree(
    post_id: i64,
    sort_type: SortType,
    max_depth: Option<usize>,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithChildren>, AppError> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_post_comment_tree(
        post_id,
        sort_type,
        max_depth,
        user_id,
        COMMENT_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(comment_tree)
}

#[server]
pub async fn get_comment_tree_by_id(
    comment_id: i64,
    sort_type: SortType,
    max_depth: Option<usize>,
) -> Result<CommentWithChildren, AppError> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_comment_tree_by_id(
        comment_id,
        sort_type,
        max_depth,
        user_id,
        &db_pool,
    ).await?;

    Ok(comment_tree)
}

#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
    is_markdown: bool,
    is_pinned: Option<bool>,
) -> Result<CommentWithChildren, AppError> {
    log::trace!("Create comment for post {post_id}");
    check_string_length(&comment, "Comment", MAX_CONTENT_LENGTH as usize, false)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = get_html_and_markdown_strings(comment, is_markdown).await?;

    let mut comment = ssr::create_comment(
        post_id,
        parent_comment_id,
        comment.as_str(),
        markdown_comment.as_deref(),
        is_pinned.unwrap_or(false),
        &user,
        &db_pool,
    )
        .await?;

    let vote = vote_on_content(
        VoteValue::Up,
        comment.post_id,
        Some(comment.comment_id),
        None,
        &user,
        &db_pool,
    ).await?;

    comment.score = 1;

    let notif_type = match parent_comment_id {
        Some(_) => NotificationType::CommentReply,
        None => NotificationType::PostReply,
    };
    create_notification(post_id, comment.parent_id, Some(comment.comment_id), user.user_id, notif_type, &db_pool).await?;

    Ok(CommentWithChildren {
        comment,
        vote,
        child_comments: Vec::<CommentWithChildren>::default(),
    })
}

#[server]
pub async fn edit_comment(
    comment_id: i64,
    comment: String,
    is_markdown: bool,
    is_pinned: Option<bool>,
) -> Result<Comment, AppError> {
    log::trace!("Edit comment {comment_id}");
    check_string_length(&comment, "Comment", MAX_CONTENT_LENGTH as usize, false)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = get_html_and_markdown_strings(comment, is_markdown).await?;

    let comment = ssr::update_comment(
        comment_id,
        comment.as_str(),
        markdown_comment.as_deref(),
        is_pinned.unwrap_or(false),
        &user,
        &db_pool,
    )
        .await?;

    Ok(comment)
}

#[server]
pub async fn delete_comment(
    comment_id: i64,
) -> Result<(), AppError> {
    log::trace!("Edit comment {comment_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::delete_comment(
        comment_id,
        &user,
        &db_pool,
    ).await?;

    Ok(())
}