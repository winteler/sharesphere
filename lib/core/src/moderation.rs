use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use sharesphere_utils::errors::AppError;
use crate::post::Post;

#[cfg(feature = "ssr")]
use {
    sharesphere_utils::{
        checks::check_string_length,
        constants::MAX_MOD_MESSAGE_LENGTH,
    },
    sharesphere_auth::{
        auth::ssr::{check_user, reload_user},
        session::ssr::get_db_pool,
    },
    crate::{
        post::ssr::get_post_by_id,
        comment::ssr::{get_comment_by_id, get_comment_sphere},
        rule::ssr::load_rule_by_id,
    }
};
use sharesphere_utils::icons::HammerIcon;
use sharesphere_utils::widget::ContentBody;
use crate::comment::Comment;
use crate::rule::Rule;

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
                "UPDATE posts SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = NOW(),
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    post_id = $5
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(post_id)
                .fetch_one(db_pool)
                .await?
        } else {
            sqlx::query_as::<_, Post>(
                "UPDATE posts p SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = NOW(),
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    p.post_id = $5 AND
                    EXISTS (
                        SELECT * FROM user_sphere_roles r
                        WHERE
                            r.sphere_id = p.sphere_id AND
                            r.user_id = $3 AND
                            r.permission_level != 'None'
                    )
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(post_id)
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
                "UPDATE comments SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = NOW(),
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    comment_id = $5
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(comment_id)
                .fetch_one(db_pool)
                .await?
        } else {
            // check if the user has at least the moderate permission for this sphere
            sqlx::query_as::<_, Comment>(
                "UPDATE comments c SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = NOW(),
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    c.comment_id = $5 AND
                    EXISTS (
                        SELECT * FROM user_sphere_roles r
                        JOIN posts p ON p.sphere_id = r.sphere_id
                        WHERE
                            p.post_id = c.post_id AND
                            r.user_id = $3  AND
                            r.permission_level != 'None'
                    )
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(comment_id)
                .fetch_one(db_pool)
                .await?
        };

        Ok(comment)
    }

    pub async fn ban_user_from_sphere(
        user_id: i64,
        sphere_name: &String,
        post_id: i64,
        comment_id: Option<i64>,
        rule_id: i64,
        user: &User,
        ban_duration_days: Option<usize>,
        db_pool: &PgPool,
    ) -> Result<Option<UserBan>, AppError> {
        if user.check_permissions(&sphere_name, PermissionLevel::Moderate).is_ok() &&
            user.user_id != user_id &&
            !is_user_sphere_moderator(user_id, sphere_name, &db_pool).await? 
        {
            let user_ban = match ban_duration_days {
                Some(0) => None,
                ban_duration => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "WITH ban AS (
                            INSERT INTO user_bans (user_id, sphere_id, sphere_name, post_id, comment_id, infringed_rule_id, moderator_id, until_timestamp)
                             VALUES (
                                $1,
                                (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                                $2, $3, $4, $5, $6, NOW() + $7 * interval '1 day'
                            ) RETURNING *
                        )
                        SELECT b.*, u.username FROM ban b
                        JOIN users u ON u.user_id = b.user_id",
                        user_id,
                        sphere_name,
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
        &post.sphere_name,
        post.post_id,
        None,
        rule_id,
        &user,
        ban_duration_days,
        &db_pool,
    ).await?;

    reload_user(post.creator_id)?;

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
        &sphere.sphere_name,
        comment.post_id,
        Some(comment.comment_id),
        rule_id,
        &user,
        ban_duration_days,
        &db_pool
    ).await?;

    reload_user(comment.creator_id)?;

    Ok(comment)
}

/// Displays the body of a moderated post or comment
#[component]
pub fn ModeratedBody(
    infringed_rule_title: String,
    moderator_message: String,
) -> impl IntoView {
    view! {
        <div class="flex items-stretch w-fit">
            <div class="flex justify-center items-center p-2 rounded-l bg-base-content/20">
                <HammerIcon/>
            </div>
            <div class="p-2 rounded-r bg-base-300 whitespace-pre text-wrap align-middle">
                <div class="flex flex-col gap-1">
                    <div>{moderator_message}</div>
                    <div>{format!("Infringed rule: {infringed_rule_title}")}</div>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Component to display the details of a moderation instance
#[component]
pub fn ModerationInfoDialog<'a>(
    moderation_info: &'a ModerationInfo,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <h1 class="text-center font-bold text-2xl">"Ban details"</h1>
            {
                match &moderation_info.content {
                    Content::Post(post) => view! {
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-2xl pl-6">"Content"</h1>
                            <div>{post.title.clone()}</div>
                            <ContentBody
                                body=post.body.clone()
                                is_markdown=post.markdown_body.is_some()
                            />
                        </div>
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-2xl pl-6">"Moderator message"</h1>
                            <div>{post.moderator_message.clone()}</div>
                        </div>
                    }.into_any(),
                    Content::Comment(comment) => {
                        view! {
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-2xl pl-6">"Content"</div>
                                <ContentBody
                                    body=comment.body.clone()
                                    is_markdown=comment.markdown_body.is_some()
                                />
                            </div>
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-2xl pl-6">"Moderator message"</div>
                                <div>{comment.moderator_message.clone()}</div>
                            </div>
                        }.into_any()
                    }
                }
            }
            <div class="flex flex-col gap-1 p-2">
                <h1 class="font-bold text-2xl pl-6">"Infringed rule"</h1>
                <div class="text-lg font-semibold">{moderation_info.rule.title.clone()}</div>
                <div>{moderation_info.rule.description.clone()}</div>
            </div>
        </div>
    }
}