use leptos::server;
use server_fn::ServerFnError;
use sharesphere_utils::errors::AppError;
use crate::post::Post;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::{check_user, reload_user},
        session::ssr::get_db_pool,
    },
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::{AdminRole, PermissionLevel};
    use sharesphere_auth::role::ssr::is_user_sphere_moderator;
    use sharesphere_auth::user::{User, UserBan};
    use sharesphere_utils::errors::AppError;
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
                    edit_timestamp = CURRENT_TIMESTAMP,
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
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    p.post_id = $5 AND
                    EXISTS (
                        SELECT * FROM user_sphere_roles r
                        WHERE
                            r.sphere_id = p.sphere_id AND
                            r.user_id = $3
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
        if user.check_permissions(&sphere_name, PermissionLevel::Moderate).is_ok() && user.user_id != user_id && !is_user_sphere_moderator(user_id, sphere_name, &db_pool).await? {
            let user_ban = match ban_duration_days {
                Some(0) => None,
                Some(ban_duration) => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "INSERT INTO user_bans (user_id, username, sphere_id, sphere_name, post_id, comment_id, infringed_rule_id, moderator_id, until_timestamp)
                         VALUES (
                            $1,
                            (SELECT username FROM users WHERE user_id = $1),
                            (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                            $2, $3, $4, $5, $6, CURRENT_TIMESTAMP + $7 * interval '1 day'
                        ) RETURNING *",
                        user_id,
                        sphere_name,
                        post_id,
                        comment_id,
                        rule_id,
                        user.user_id,
                        ban_duration as f64,
                    )
                        .fetch_one(db_pool)
                        .await?)
                }
                None => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "INSERT INTO user_bans (user_id, username, sphere_id, sphere_name, post_id, comment_id, infringed_rule_id, moderator_id)
                         VALUES (
                            $1,
                            (SELECT username FROM users WHERE user_id = $1),
                            (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                            $2, $3, $4, $5, $6
                        ) RETURNING *",
                        user_id,
                        sphere_name,
                        post_id,
                        comment_id,
                        rule_id,
                        user.user_id,
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
) -> Result<Post, ServerFnError<AppError>> {
    log::debug!("Moderate post {post_id}, ban duration = {ban_duration_days:?}");
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