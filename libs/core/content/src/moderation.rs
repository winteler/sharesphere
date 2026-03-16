use leptos::prelude::*;
use leptos_fluent::{move_tr, tr};
use serde::{Deserialize, Serialize};

use sharesphere_core_common::errors::AppError;
use sharesphere_core_sphere::rule::Rule;

use crate::comment::Comment;
use crate::post::Post;

#[cfg(feature = "ssr")]
use {
    crate::{
        comment::ssr::{get_comment_by_id, get_comment_sphere},
        post::ssr::get_post_by_id,
    }
};

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

    use sharesphere_core_common::errors::AppError;
    use sharesphere_core_user::role::{AdminRole, PermissionLevel};
    use sharesphere_core_user::role::ssr::is_user_sphere_moderator;
    use sharesphere_core_user::user::{User, UserBan};

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