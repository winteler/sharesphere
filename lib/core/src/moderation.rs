use leptos::prelude::*;
use leptos_fluent::{move_tr, tr};
use serde::{Deserialize, Serialize};

use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::HammerIcon;
use sharesphere_utils::widget::ContentBody;

use crate::comment::{Comment};
use crate::post::{Post};
use crate::rule::Rule;

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
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;
    use sharesphere_auth::role::{AdminRole, PermissionLevel};
    use sharesphere_auth::role::ssr::is_user_sphere_moderator;
    use sharesphere_auth::user::{User, UserBan};
    use sharesphere_utils::embed::{Link, LinkType};
    use sharesphere_utils::errors::AppError;
    use crate::comment::Comment;
    use crate::moderation::Content;
    use crate::post::Post;

    #[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, sqlx::FromRow)]
    struct ContentRow {
        pub post_id: i64,
        pub comment_id: Option<i64>,
        pub parent_id: Option<i64>,
        pub title: Option<String>,
        pub body: String,
        pub markdown_body: Option<String>,
        pub link_type: Option<LinkType>,
        pub link_url: Option<String>,
        pub link_embed: Option<String>,
        pub link_thumbnail_url: Option<String>,
        pub is_nsfw: Option<bool>,
        pub is_spoiler: Option<bool>,
        pub category_id: Option<i64>,
        pub is_edited: bool,
        pub sphere_id: Option<i64>,
        pub satellite_id: Option<i64>,
        pub creator_id: i64,
        pub creator_name: String,
        pub is_creator_moderator: bool,
        pub moderator_message: Option<String>,
        pub infringed_rule_id: Option<i64>,
        pub infringed_rule_title: Option<String>,
        pub moderator_id: Option<i64>,
        pub moderator_name: Option<String>,
        pub num_comments: Option<i32>,
        pub is_pinned: bool,
        pub score: i32,
        pub score_minus: i32,
        pub recommended_score: Option<f32>,
        pub trending_score: Option<f32>,
        pub create_timestamp: chrono::DateTime<chrono::Utc>,
        pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
        pub scoring_timestamp: chrono::DateTime<chrono::Utc>,
        pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl ContentRow {
        pub fn into_content(self) -> Content {
            let link = self.link_type.map(|link_type| Link {
                link_type,
                link_url: self.link_url,
                link_embed: self.link_embed,
                link_thumbnail_url: self.link_thumbnail_url,
            });
            match self.comment_id {
                Some(comment_id) => Content::Comment(
                    Comment {
                        comment_id,
                        body: self.body,
                        markdown_body: self.markdown_body,
                        is_edited: self.is_edited,
                        moderator_message: self.moderator_message,
                        infringed_rule_id: self.infringed_rule_id,
                        infringed_rule_title: self.infringed_rule_title,
                        parent_id: self.parent_id,
                        post_id: self.post_id,
                        creator_id: self.creator_id,
                        creator_name: self.creator_name,
                        is_creator_moderator: self.is_creator_moderator,
                        moderator_id: self.moderator_id,
                        moderator_name: self.moderator_name,
                        is_pinned: self.is_pinned,
                        score: self.score,
                        score_minus: self.score_minus,
                        create_timestamp: self.create_timestamp,
                        edit_timestamp: self.edit_timestamp,
                        delete_timestamp: self.delete_timestamp,
                    }
                ),
                None => Content::Post(Post {
                    post_id: self.post_id,
                    title: self.title.unwrap(),
                    body: self.body,
                    markdown_body: self.markdown_body,
                    link: link.unwrap(),
                    is_nsfw: self.is_nsfw.unwrap(),
                    is_spoiler: self.is_spoiler.unwrap(),
                    category_id: self.category_id,
                    is_edited: self.is_edited,
                    sphere_id: self.sphere_id.unwrap(),
                    satellite_id: self.satellite_id,
                    creator_id: self.creator_id,
                    creator_name: self.creator_name,
                    is_creator_moderator: self.is_creator_moderator,
                    moderator_message: self.moderator_message,
                    infringed_rule_id: self.infringed_rule_id,
                    infringed_rule_title: self.infringed_rule_title,
                    moderator_id: self.moderator_id,
                    moderator_name: self.moderator_name,
                    num_comments: self.num_comments.unwrap(),
                    is_pinned: self.is_pinned,
                    score: self.score,
                    score_minus: self.score_minus,
                    recommended_score: self.recommended_score.unwrap(),
                    trending_score: self.trending_score.unwrap(),
                    create_timestamp: self.create_timestamp,
                    edit_timestamp: self.edit_timestamp,
                    scoring_timestamp: self.scoring_timestamp,
                    delete_timestamp: self.delete_timestamp,
                })
            }
        }
    }

    pub async fn get_sphere_contents(
        sphere_name: &str,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<Content>, AppError> {
        let content_row_vec = sqlx::query_as::<_, ContentRow>(
            "SELECT * FROM (
                SELECT
                    p.post_id,
                    NULL AS comment_id,
                    NULL AS parent_id,
                    p.title,
                    p.body,
                    p.markdown_body,
                    p.link_type,
                    p.link_url,
                    p.link_embed,
                    p.link_thumbnail_url,
                    p.is_nsfw,
                    p.is_spoiler,
                    p.category_id,
                    p.is_edited,
                    p.sphere_id,
                    p.satellite_id,
                    p.creator_id,
                    u.username as creator_name,
                    p.is_creator_moderator,
                    p.moderator_message,
                    p.infringed_rule_id,
                    r.title as infringed_rule_title,
                    p.moderator_id,
                    m.username AS moderator_name,
                    p.num_comments,
                    p.is_pinned,
                    p.score,
                    p.score_minus,
                    p.recommended_score,
                    p.trending_score,
                    p.create_timestamp,
                    p.edit_timestamp,
                    p.scoring_timestamp,
                    p.delete_timestamp
                FROM posts p
                JOIN spheres s ON s.sphere_id = p.sphere_id
                JOIN users u ON u.user_id = p.creator_id
                LEFT JOIN users m ON m.user_id = p.moderator_id
                LEFT JOIN rules r ON r.rule_id = p.creator_id
                WHERE
                    s.sphere_name = $1 AND
                    p.moderator_id IS NULL AND
                    p.delete_timestamp IS NULL
                UNION ALL
                SELECT
                    c.post_id,
                    c.comment_id,
                    c.parent_id,
                    NULL AS title,
                    c.body,
                    c.markdown_body,
                    NULL AS link_type,
                    NULL AS link_url,
                    NULL AS link_embed,
                    NULL AS link_thumbnail_url,
                    NULL AS is_nsfw,
                    NULL AS spoiler,
                    NULL AS category_id,
                    c.is_edited,
                    NULL AS sphere_id,
                    NULL AS satellite_id,
                    c.creator_id,
                    u.username AS creator_name,
                    c.is_creator_moderator,
                    p.moderator_message,
                    p.infringed_rule_id,
                    r.title as infringed_rule_title,
                    p.moderator_id,
                    m.username AS moderator_name,
                    NULL AS num_comments,
                    c.is_pinned,
                    c.score,
                    c.score_minus,
                    NULL AS recommended_score,
                    NULL AS trending_score,
                    c.create_timestamp,
                    c.edit_timestamp,
                    NULL AS scoring_timestamp,
                    c.delete_timestamp
                FROM comments c
                JOIN posts p ON p.post_id = c.post_id
                JOIN spheres s ON s.sphere_id = p.sphere_id
                JOIN users u ON u.user_id = c.creator_id
                LEFT JOIN users m ON m.user_id = c.moderator_id
                LEFT JOIN rules r ON r.rule_id = c.infringed_rule_id
                WHERE
                    s.sphere_name = $1 AND
                    p.moderator_id IS NULL AND
                    p.delete_timestamp IS NULL AND
                    c.moderator_id IS NULL AND
                    c.delete_timestamp IS NULL
            ) as contents
            ORDER BY create_timestamp desc
            LIMIT $2
            OFFSET $3"
        )
            .bind(sphere_name)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool).await?;

        let content_vec = content_row_vec.into_iter().map(ContentRow::into_content).collect();

        Ok(content_vec)
    }

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
                    r.title as infringed_rule_title
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
                    r.title as infringed_rule_title
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
                    r.title as infringed_rule_title
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
                    r.title as infringed_rule_title
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
                    <div>{move || format!("{}: {infringed_rule_title}", tr!("infringed-rule"))}</div>
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
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <h1 class="text-center font-bold text-2xl">"Ban details"</h1>
            {
                match &moderated_content {
                    Content::Post(post) => view! {
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-2xl pl-6">{move_tr!("content")}</h1>
                            <div>{post.title.clone()}</div>
                            <ContentBody
                                body=post.body.clone()
                                is_markdown=post.markdown_body.is_some()
                            />
                        </div>
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-2xl pl-6">{move_tr!("moderator-message")}</h1>
                            <div>{post.moderator_message.clone()}</div>
                        </div>
                    }.into_any(),
                    Content::Comment(comment) => {
                        view! {
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-2xl pl-6">{move_tr!("content")}</div>
                                <ContentBody
                                    body=comment.body.clone()
                                    is_markdown=comment.markdown_body.is_some()
                                />
                            </div>
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-2xl pl-6">{move_tr!("moderator-message")}</div>
                                <div>{comment.moderator_message.clone()}</div>
                            </div>
                        }.into_any()
                    }
                }
            }
            <div class="flex flex-col gap-1 p-2">
                <h1 class="font-bold text-2xl pl-6">{move_tr!("infringed-rule")}</h1>
                <div class="text-lg font-semibold">{rule_title.clone()}</div>
                <div>{rule_description.clone()}</div>
            </div>
        </div>
    }
}