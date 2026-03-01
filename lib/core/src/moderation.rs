use leptos::prelude::*;
use leptos_fluent::{move_tr, tr};
use serde::{Deserialize, Serialize};
use sharesphere_utils::errors::AppError;
use crate::post::Post;

#[cfg(feature = "ssr")]
use {
    crate::{
        comment::ssr::{get_comment_by_id, get_comment_sphere},
        post::ssr::get_post_by_id,
        rule::ssr::load_rule_by_id,
    },
    sharesphere_auth::{
        auth::ssr::{check_user, reload_user},
        session::ssr::get_db_pool,
    },
    sharesphere_utils::{
        checks::check_string_length,
        constants::MAX_MOD_MESSAGE_LENGTH,
    }
};
use sharesphere_utils::icons::HammerIcon;
use sharesphere_utils::widget::ContentBody;
use crate::comment::Comment;
#[cfg(feature = "ssr")]
use crate::notification::{ssr::create_notification, NotificationType};
use crate::rule::{get_rule_description, get_rule_title, Rule};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Content {
    Post(Post),
    Comment(Comment),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModerationInfo {
    pub rule: Rule,
    pub content: Content,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::{AdminRole, PermissionLevel};
    use sharesphere_auth::role::ssr::is_user_sphere_moderator;
    use sharesphere_auth::user::{User, UserBan};
    use sharesphere_utils::errors::AppError;
    use crate::comment::Comment;
    use crate::post::Post;

    pub async fn moderate_post(
        post_id: i64,
        rule_id: i64,
        moderator_message: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = if user.check_admin_role(AdminRole::Moderator).is_ok() {
            sqlx::query_as::<_, Post>(
                "WITH moderated_post AS (
                    UPDATE posts SET
                        moderator_message = $1,
                        infringed_rule_id = $2,
                        edit_timestamp = NOW(),
                        moderator_id = $3
                    WHERE
                        post_id = $4
                    RETURNING *
                )
                SELECT
                    p.*,
                    u.username as creator_name,
                    $5 as moderator_name,
                    r.title as infringed_rule_title,
                    r.sphere_id IS NOT NULL AS is_sphere_rule
                FROM moderated_post p
                JOIN users u ON u.user_id = p.creator_id
                JOIN rules r ON r.rule_id = p.infringed_rule_id",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(post_id)
                .bind(user.username.clone())
                .fetch_one(db_pool)
                .await?
        } else {
            sqlx::query_as::<_, Post>(
                "WITH moderated_post AS (
                    UPDATE posts p SET
                        moderator_message = $1,
                        infringed_rule_id = $2,
                        edit_timestamp = NOW(),
                        moderator_id = $3
                    WHERE
                        p.post_id = $4 AND
                        EXISTS (
                            SELECT * FROM user_sphere_roles r
                            WHERE
                                r.sphere_id = p.sphere_id AND
                                r.user_id = $3 AND
                                r.permission_level != 'None'
                        )
                    RETURNING *
                )
                SELECT
                    p.*,
                    u.username as creator_name,
                    $5 as moderator_name,
                    r.title as infringed_rule_title,
                    r.sphere_id IS NOT NULL AS is_sphere_rule
                FROM moderated_post p
                JOIN users u ON u.user_id = p.creator_id
                JOIN rules r ON r.rule_id = p.infringed_rule_id",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(post_id)
                .bind(user.username.clone())
                .fetch_one(db_pool)
                .await?
        };

        Ok(post)
    }

    pub async fn moderate_comment(
        comment_id: i64,
        rule_id: i64,
        moderator_message: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let comment = if user.check_admin_role(AdminRole::Moderator).is_ok() {
            sqlx::query_as::<_, Comment>(
                "WITH moderated_comment AS (
                        UPDATE comments SET
                        moderator_message = $1,
                        infringed_rule_id = $2,
                        edit_timestamp = NOW(),
                        moderator_id = $3
                    WHERE
                        comment_id = $4
                    RETURNING *
                )
                SELECT
                    c.*,
                    u.username as creator_name,
                    $5 as moderator_name,
                    r.title as infringed_rule_title,
                    r.sphere_id IS NOT NULL AS is_sphere_rule
                FROM moderated_comment c
                JOIN users u ON u.user_id = c.creator_id
                JOIN rules r ON r.rule_id = c.infringed_rule_id",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(comment_id)
                .bind(user.username.clone())
                .fetch_one(db_pool)
                .await?
        } else {
            // check if the user has at least the moderate permission for this sphere
            sqlx::query_as::<_, Comment>(
                "WITH moderated_comment AS (
                    UPDATE comments c SET
                        moderator_message = $1,
                        infringed_rule_id = $2,
                        edit_timestamp = NOW(),
                        moderator_id = $3
                    WHERE
                        c.comment_id = $4 AND
                        EXISTS (
                            SELECT * FROM user_sphere_roles r
                            JOIN posts p ON p.sphere_id = r.sphere_id
                            WHERE
                                p.post_id = c.post_id AND
                                r.user_id = $3  AND
                                r.permission_level != 'None'
                        )
                    RETURNING *
                )
                SELECT
                    c.*,
                    u.username as creator_name,
                    $5 as moderator_name,
                    r.title as infringed_rule_title,
                    r.sphere_id IS NOT NULL AS is_sphere_rule
                FROM moderated_comment c
                JOIN users u ON u.user_id = c.creator_id
                JOIN rules r ON r.rule_id = c.infringed_rule_id",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(comment_id)
                .bind(user.username.clone())
                .fetch_one(db_pool)
                .await?
        };

        Ok(comment)
    }

    pub async fn ban_user_from_sphere(
        user_id: i64,
        sphere_id: i64,
        post_id: i64,
        comment_id: Option<i64>,
        rule_id: i64,
        user: &User,
        ban_duration_days: Option<usize>,
        db_pool: &PgPool,
    ) -> Result<Option<UserBan>, AppError> {
        if user.check_sphere_permissions_by_id(sphere_id, PermissionLevel::Moderate).is_ok() &&
            user.user_id != user_id &&
            !is_user_sphere_moderator(user_id, sphere_id, &db_pool).await?
        {
            let user_ban = match ban_duration_days {
                Some(0) => None,
                ban_duration => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "WITH ban AS (
                            INSERT INTO user_bans (user_id, sphere_id, post_id, comment_id, infringed_rule_id, moderator_id, until_timestamp)
                             VALUES (
                                $1, $2, $3, $4, $5, $6, NOW() + $7 * interval '1 day'
                            ) RETURNING *
                        )
                        SELECT b.*, u.username, s.sphere_name FROM ban b
                        JOIN users u ON u.user_id = b.user_id
                        JOIN spheres s ON s.sphere_id = b.sphere_id",
                        user_id,
                        sphere_id,
                        post_id,
                        comment_id,
                        rule_id,
                        user.user_id,
                        ban_duration.map(|duration| duration as f64),
                    )
                        .fetch_one(db_pool)
                        .await?)
                }
            };
            Ok(user_ban)
        } else {
            Err(AppError::InternalServerError(format!("Error while trying to ban user {user_id}. Insufficient permissions or user is a moderator of the sphere.")))
        }
    }
}

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

/// Displays the body of a moderated post or comment
#[component]
pub fn ModeratedBody(
    infringed_rule_title: String,
    moderator_message: String,
    is_sphere_rule: bool,
) -> impl IntoView {
    let infringed_rule_title = get_rule_title(&infringed_rule_title, is_sphere_rule);
    view! {
        <div class="flex items-stretch w-fit">
            <div class="flex justify-center items-center p-2 rounded-l bg-base-content/20">
                <HammerIcon/>
            </div>
            <div class="p-2 rounded-r bg-base-300 whitespace-pre text-wrap align-middle">
                <div class="flex flex-col gap-1">
                    <div>{moderator_message}</div>
                    <div>{move || format!("{}: {}", tr!("infringed-rule"), infringed_rule_title.read())}</div>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Component to display the details of a moderation instance
#[component]
pub fn ModerationInfoDialog(
    moderated_content: Content,
    rule_title: String,
    rule_description: String,
    is_sphere_rule: bool,
) -> impl IntoView {
    let title = get_rule_title(&rule_title, is_sphere_rule);
    let description = get_rule_description(&rule_title, &rule_description, is_sphere_rule);
    view! {
        <div class="flex flex-col gap-3">
            <h1 class="text-center font-bold text-2xl">"Ban details"</h1>
            {
                match &moderated_content {
                    Content::Post(post) => view! {
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-xl">{move_tr!("content")}</h1>
                            <div>{post.title.clone()}</div>
                            <ContentBody
                                body=post.body.clone()
                                is_markdown=post.markdown_body.is_some()
                            />
                        </div>
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-xl">{move_tr!("moderator-message")}</h1>
                            <div>{post.moderator_message.clone()}</div>
                        </div>
                    }.into_any(),
                    Content::Comment(comment) => {
                        view! {
                            <div class="flex flex-col gap-1 p-2 border-b border-base-content/20">
                                <div class="font-bold text-xl">{move_tr!("content")}</div>
                                <ContentBody
                                    body=comment.body.clone()
                                    is_markdown=comment.markdown_body.is_some()
                                />
                            </div>
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-xl">{move_tr!("moderator-message")}</div>
                                <div>{comment.moderator_message.clone()}</div>
                            </div>
                        }.into_any()
                    }
                }
            }
            <div class="flex flex-col gap-1 p-2">
                <h1 class="font-bold text-xl">{move_tr!("infringed-rule")}</h1>
                <div class="text-lg font-semibold">{title}</div>
                <div>{description}</div>
            </div>
        </div>
    }
}