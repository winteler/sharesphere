use const_format::concatcp;
use leptos::*;
use leptos_router::{ActionForm, use_params_map};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::ssr::check_user, auth::ssr::reload_user, comment::ssr::get_comment_forum};
use crate::app::ModerateState;
use crate::comment::Comment;
use crate::editor::FormTextEditor;
use crate::forum::get_forum_name_memo;
use crate::icons::HammerIcon;
use crate::post::Post;
use crate::role::{get_forum_role_vec, PermissionLevel, SetUserForumRole};
use crate::unpack::TransitionUnpack;
use crate::widget::{ActionError, EnumDropdown, ModalDialog, ModalFormButtons};

pub const MANAGE_FORUM_SUFFIX: &str = "manage";
pub const MANAGE_FORUM_ROUTE: &str = concatcp!("/", MANAGE_FORUM_SUFFIX);
pub const NONE_STR: &str = "None";
pub const DAY_STR: &str = "day";
pub const DAYS_STR: &str = "days";
pub const PERMANENT_STR: &str = "Permanent";

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct UserBan {
    pub ban_id: i64,
    pub user_id: i64,
    pub username: String,
    pub forum_id: Option<i64>,
    pub forum_name: Option<String>,
    pub moderator_id: i64,
    pub until_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::auth::User;
    use crate::comment::Comment;
    use crate::errors::AppError;
    use crate::forum_management::UserBan;
    use crate::post::Post;

    pub async fn moderate_post(
        post_id: i64,
        moderated_body: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = if user.check_is_global_moderator().is_ok() {
            sqlx::query_as!(
                Post,
                "UPDATE posts SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    post_id = $4
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                post_id,
            )
                .fetch_one(db_pool)
                .await?
        } else {
            sqlx::query_as!(
                Post,
                "UPDATE posts p SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    p.post_id = $4 AND
                    EXISTS (
                        SELECT * FROM user_forum_roles r
                        WHERE
                            r.forum_id = p.forum_id AND
                            r.user_id = $2
                    )
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                post_id,
            )
                .fetch_one(db_pool)
                .await?
        };

        Ok(post)
    }

    pub async fn moderate_comment(
        comment_id: i64,
        moderated_body: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let comment = if user.check_is_global_moderator().is_ok() {
            sqlx::query_as!(
                Comment,
                "UPDATE comments SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    comment_id = $4
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                comment_id,
            )
                .fetch_one(db_pool)
                .await?
        } else {
            // check if the user has at least the moderate permission for this forum
            sqlx::query_as!(
                Comment,
                "UPDATE comments c SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    c.comment_id = $4 AND
                    EXISTS (
                        SELECT * FROM user_forum_roles r
                        JOIN posts p ON p.forum_id = r.forum_id
                        WHERE
                            p.post_id = c.post_id AND
                            r.user_id = $2
                    )
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                comment_id,
            )
                .fetch_one(db_pool)
                .await?
        };

        Ok(comment)
    }

    pub async fn ban_user_from_forum(
        user_id: i64,
        forum_name: &String,
        user: &User,
        ban_duration_days: Option<usize>,
        db_pool: &PgPool,
    ) -> Result<Option<UserBan>, AppError> {
        if user.check_can_moderate_forum(&forum_name).is_ok() && user.user_id != user_id && !is_user_forum_moderator(user_id, forum_name, &db_pool).await? {
            let user_ban = match ban_duration_days {
                Some(0) => None,
                Some(ban_duration) => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "INSERT INTO user_bans (user_id, username, forum_id, forum_name, moderator_id, until_timestamp)
                         VALUES (
                            $1,
                            (SELECT username FROM users WHERE user_id = $1),
                            (SELECT forum_id FROM forums WHERE forum_name = $2),
                            $2, $3, CURRENT_TIMESTAMP + $4 * interval '1 day'
                        ) RETURNING *",
                        user_id,
                        forum_name,
                        user.user_id,
                        ban_duration as f64,
                    )
                        .fetch_one(db_pool)
                        .await?)
                }
                None => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "INSERT INTO user_bans (user_id, username, forum_id, forum_name, moderator_id)
                         VALUES (
                            $1,
                            (SELECT username FROM users WHERE user_id = $1),
                            (SELECT forum_id FROM forums WHERE forum_name = $2),
                            $2, $3
                        ) RETURNING *",
                        user_id,
                        forum_name,
                        user.user_id,
                    )
                        .fetch_one(db_pool)
                        .await?)
                }
            };
            Ok(user_ban)
        } else {
            Err(AppError::InternalServerError(format!("Error while trying to ban user {user_id}. Insufficient permissions or user is a moderator of the forum.")))
        }
    }

    pub async fn is_user_forum_moderator(
        user_id: i64,
        forum: &String,
        db_pool: &PgPool,
    ) -> Result<bool, AppError> {
        match User::get(user_id, db_pool).await {
            Some(user) => Ok(user.check_can_moderate_forum(&forum).is_ok()),
            None => Err(AppError::InternalServerError(format!("Could not find user with id = {user_id}"))),
        }
    }
}

/// Function to moderate a post and optionally ban its author
///
/// The ban is performed for the forum of the given post and the duration is given by `ban_num_days`.
/// If `ban_num_days == None`, the duration of the ban is permanent.
#[server]
pub async fn moderate_post(
    post_id: i64,
    moderated_body: String,
    ban_duration_days: Option<usize>,
) -> Result<Post, ServerFnError> {
    log::info!("Moderate post {post_id}, ban duration = {ban_duration_days:?}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let post = ssr::moderate_post(
        post_id,
        moderated_body.as_str(),
        &user,
        &db_pool
    ).await?;

    ssr::ban_user_from_forum(
        post.creator_id,
        &post.forum_name,
        &user,
        ban_duration_days,
        &db_pool,
    ).await?;

    reload_user(post.creator_id)?;

    Ok(post)
}

#[server]
pub async fn moderate_comment(
    comment_id: i64,
    moderated_body: String,
    ban_duration_days: Option<usize>,
) -> Result<Comment, ServerFnError> {
    log::trace!("Moderate comment {comment_id}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let comment = ssr::moderate_comment(
        comment_id,
        moderated_body.as_str(),
        &user,
        &db_pool
    ).await?;

    let forum = get_comment_forum(comment_id, &db_pool).await?;

    ssr::ban_user_from_forum(
        comment.creator_id,
        &forum.forum_name,
        &user,
        ban_duration_days,
        &db_pool
    ).await?;

    reload_user(comment.creator_id)?;

    Ok(comment)
}

/// Component to manage a forum
#[component]
pub fn ForumCockpit() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1 content-center">
            <div class="text-2xl text-center">"Forum Cockpit"</div>
            <ModeratorPanel/>
        </div>
    }
}

/// Component to manage moderators
#[component]
pub fn ModeratorPanel() -> impl IntoView {
    let params = use_params_map();
    let forum_name = get_forum_name_memo(params);
    let username_input = create_rw_signal(String::default());

    let set_role_action = create_server_action::<SetUserForumRole>();
    let forum_roles_resource = create_resource(
        move || (forum_name.get(), set_role_action.version().get()),
        move |(forum_name, _)| get_forum_role_vec(forum_name)
    );
    view! {
        <div class="flex flex-col gap-1 content-center w-fit bg-base-200 p-2 rounded">
            <div class="text-xl">"Moderator list"</div>
            <TransitionUnpack resource=forum_roles_resource let:forum_role_vec>
            {
                let forum_role_vec = forum_role_vec.clone();
                view! {
                    <For
                        each= move || forum_role_vec.clone().into_iter().enumerate()
                        key=|(_index, role)| (role.user_id, role.permission_level)
                        let:child
                    >
                        <div class="w-fit">{format!("{}: {}", child.1.username, child.1.permission_level.to_string())}</div>
                    </For>
                }
            }
            </TransitionUnpack>
            // menu to set permissions: input with suggestion for username, dropdown (generic for enums) for permission levels, publish button
            // on click in list, set username and permission level in inputs
            <ActionForm action=set_role_action>
                <input
                    name="forum_name"
                    class="hidden"
                    value=forum_name
                />
                <div class="flex gap-1 content-center">
                    <div class="dropdown dropdown-end">
                        <input
                            tabindex="0"
                            type="text"
                            name="username"
                            placeholder="Username"
                            autocomplete="off"
                            class="input input-bordered input-primary w-full"
                            on:input=move |ev| {
                                username_input.update(|name: &mut String| *name = event_target_value(&ev).to_lowercase());
                            }
                            prop:value=username_input
                        />
                    </div>
                    <EnumDropdown name="ban_duration" enum_iter=PermissionLevel::iter()/>
                    <button
                        type="submit"
                        class="btn btn-active btn-secondary"
                    >
                        "Assign"
                    </button>
                </div>
            </ActionForm>
        </div>
    }
}

/// Component to moderate a post
#[component]
pub fn ModerateButton(show_dialog: RwSignal<bool>) -> impl IntoView {
    let moderate_state = use_context::<ModerateState>();
    let edit_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <Show when=move || match moderate_state {
            Some(user_state) => user_state.can_moderate.get(),
            None => false,
        }>
            <button
                class=edit_button_class
                aria-expanded=move || show_dialog.get().to_string()
                aria-haspopup="dialog"
                on:click=move |_| show_dialog.set(true)
            >
                <HammerIcon/>
            </button>
        </Show>
    }
}

/// Component to access a post's moderation dialog
#[component]
pub fn ModeratePostButton(post_id: i64) -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <div>
            <ModerateButton show_dialog/>
            <ModeratePostDialog
                post_id
                show_dialog
            />
        </div>
    }
}

/// Component to access a comment's moderation dialog
#[component]
pub fn ModerateCommentButton(comment_id: i64, comment: RwSignal<Comment>) -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <div>
            <ModerateButton show_dialog/>
            <ModerateCommentDialog
                comment_id
                comment
                show_dialog
            />
        </div>
    }
}

/// Dialog to moderate a post
#[component]
pub fn ModeratePostDialog(
    post_id: i64,
    show_dialog: RwSignal<bool>
) -> impl IntoView {
    let moderate_state = expect_context::<ModerateState>();

    let moderate_text = create_rw_signal(String::new());
    let is_text_empty = move || moderate_text.with(|moderate_text: &String| moderate_text.is_empty());

    let moderate_result = moderate_state.moderate_post_action.value();
    let has_error = move || moderate_result.with(|val| matches!(val, Some(Err(_))));

    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a post"</div>
                <ActionForm action=moderate_state.moderate_post_action>
                    <div class="flex flex-col gap-3 w-full">
                        <input
                            type="text"
                            name="post_id"
                            class="hidden"
                            value=post_id
                        />
                        <FormTextEditor
                            name="moderated_body"
                            placeholder="Message"
                            content=moderate_text
                        />
                        <BanMenu/>
                        <ModalFormButtons
                            disable_publish=is_text_empty
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
                <ActionError has_error/>
            </div>
        </ModalDialog>
    }
}

/// Dialog to moderate a comment
#[component]
pub fn ModerateCommentDialog(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let moderate_text = create_rw_signal(String::new());
    let is_text_empty = move || moderate_text.with(|comment: &String| comment.is_empty());

    let moderate_comment_action = create_server_action::<ModerateComment>();

    let moderate_result = moderate_comment_action.value();
    let has_error = move || moderate_result.with(|val| matches!(val, Some(Err(_))));

    create_effect(move |_| {
        if let Some(Ok(moderated_comment)) = moderate_result.get() {
            comment.set(moderated_comment);
            show_dialog.set(false);
        }
    });
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a comment"</div>
                <ActionForm action=moderate_comment_action>
                    <div class="flex flex-col gap-3 w-full">
                        <input
                            type="text"
                            name="comment_id"
                            class="hidden"
                            value=comment_id
                        />
                        <FormTextEditor
                            name="moderated_body"
                            placeholder="Message"
                            content=moderate_text
                        />
                        <BanMenu/>
                        <ModalFormButtons
                            disable_publish=is_text_empty
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
                <ActionError has_error/>
            </div>
        </ModalDialog>
    }
}

/// Dialog to input number of banned days
#[component]
pub fn BanMenu() -> impl IntoView {
    let moderate_state = expect_context::<ModerateState>();
    let ban_value = create_rw_signal(0);
    let is_permanent_ban = create_rw_signal(false);

    view! {
        <input
            type="number"
            name="ban_duration_days"
            class="hidden"
            value=ban_value
            disabled=is_permanent_ban
        />
        <Show when=move || moderate_state.can_ban.get()>
            <div class="flex items-center justify-between w-full">
                <span class="text-xl font-semibold">"Ban duration (days):"</span>
                <select
                    class="select select-bordered"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        if let Ok(num_days_banned) = value.parse::<i32>() {
                            ban_value.set(num_days_banned);
                            is_permanent_ban.set(false);
                        } else {
                            ban_value.set(0);
                            is_permanent_ban.set(true);
                        }
                    }
                >
                    <option>"0"</option>
                    <option>"1"</option>
                    <option>"7"</option>
                    <option>"30"</option>
                    <option>"180"</option>
                    <option>"365"</option>
                    <option value="">"Permanent"</option>
                </select>
            </div>
        </Show>
    }
}