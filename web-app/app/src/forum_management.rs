use std::collections::BTreeSet;

use chrono::SecondsFormat;
use const_format::concatcp;
use leptos::*;
use leptos_router::ActionForm;
use leptos_use::signal_debounced;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::comment::Comment;
use crate::editor::FormTextEditor;
use crate::forum::ForumState;
use crate::icons::{DeleteIcon, EditIcon, HammerIcon, PlusIcon};
use crate::post::Post;
use crate::role::{AuthorizedShow, PermissionLevel, SetUserForumRole, UserForumRole};
use crate::unpack::TransitionUnpack;
use crate::user::get_matching_username_set;
use crate::widget::{ActionError, EnumDropdown, ModalDialog, ModalFormButtons};
#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::ssr::check_user, auth::ssr::reload_user, comment::ssr::get_comment_forum};

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

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: i64,
    pub rule_key: i64, // business id to track rule across updates
    pub forum_id: Option<i64>,
    pub forum_name: Option<String>,
    pub priority: i16,
    pub title: String,
    pub description: String,
    pub user_id: i64,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::comment::Comment;
    use crate::errors::AppError;
    use crate::forum_management::{Rule, UserBan};
    use crate::post::Post;
    use crate::role::{AdminRole, PermissionLevel};
    use crate::user::User;

    pub async fn is_user_forum_moderator(
        user_id: i64,
        forum: &String,
        db_pool: &PgPool,
    ) -> Result<bool, AppError> {
        match User::get(user_id, db_pool).await {
            Some(user) => Ok(user.check_permissions(&forum, PermissionLevel::Moderate).is_ok()),
            None => Err(AppError::InternalServerError(format!("Could not find user with id = {user_id}"))),
        }
    }

    pub async fn get_forum_rule_vec(
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<Rule>, AppError> {
        let forum_rule_vec = sqlx::query_as!(
            Rule,
            "SELECT * FROM rules
            WHERE COALESCE(forum_name, $1) = $1 AND delete_timestamp IS NULL
            ORDER BY forum_name NULLS FIRST, priority, create_timestamp",
            forum_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_rule_vec)
    }

    pub async fn add_rule(
        forum_name: Option<&str>,
        priority: i16,
        title: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        match forum_name {
            Some(forum_name) => user.check_permissions(forum_name, PermissionLevel::Manage)?,
            None => user.check_admin_role(AdminRole::Admin)?,
        };

        sqlx::query!(
            "UPDATE rules
             SET priority = priority + 1
             WHERE forum_name IS NOT DISTINCT FROM $1 AND priority >= $2 AND delete_timestamp IS NULL",
            forum_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        let rule = sqlx::query_as!(
            Rule,
            "INSERT INTO rules
            (forum_id, forum_name, priority, title, description, user_id)
            VALUES (
                (SELECT forum_id FROM forums WHERE forum_name = $1),
                $1, $2, $3, $4, $5
            ) RETURNING *",
            forum_name,
            priority,
            title,
            description,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(rule)
    }

    pub async fn update_rule(
        forum_name: Option<&str>,
        current_priority: i16,
        priority: i16,
        title: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        match forum_name {
            Some(forum_name) => user.check_permissions(forum_name, PermissionLevel::Manage)?,
            None => user.check_admin_role(AdminRole::Admin)?,
        };

        let current_rule = sqlx::query_as!(
            Rule,
            "UPDATE rules
             SET delete_timestamp = CURRENT_TIMESTAMP
             WHERE forum_name IS NOT DISTINCT FROM $1 AND priority = $2 AND delete_timestamp IS NULL
             RETURNING *",
            forum_name,
            current_priority,
        )
            .fetch_one(db_pool)
            .await?;

        if priority > current_priority {
            sqlx::query!(
                "UPDATE rules
                SET priority = priority - 1
                WHERE forum_name IS NOT DISTINCT FROM $1 AND priority BETWEEN $2 AND $3 AND delete_timestamp IS NULL",
                forum_name,
                current_priority,
                priority,
            )
                .execute(db_pool)
                .await?;
        } else if priority < current_priority {
            sqlx::query!(
                "UPDATE rules
                SET priority = priority + 1
                WHERE forum_name IS NOT DISTINCT FROM $1 AND priority BETWEEN $3 AND $2 AND delete_timestamp IS NULL",
                forum_name,
                current_priority,
                priority,
            )
                .execute(db_pool)
                .await?;
        }

        let new_rule = sqlx::query_as!(
            Rule,
            "INSERT INTO rules
            (rule_key, forum_id, forum_name, priority, title, description, user_id)
            VALUES (
                $1,
                (SELECT forum_id FROM forums WHERE forum_name = $2),
                $2, $3, $4, $5, $6
            ) RETURNING *",
            current_rule.rule_key,
            forum_name,
            priority,
            title,
            description,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(new_rule)
    }

    pub async fn remove_rule(
        forum_name: Option<&str>,
        priority: i16,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        match forum_name {
            Some(forum_name) => user.check_permissions(forum_name, PermissionLevel::Manage)?,
            None => user.check_admin_role(AdminRole::Admin)?,
        };

        sqlx::query!(
            "UPDATE rules
             SET delete_timestamp = CURRENT_TIMESTAMP
             WHERE forum_name IS NOT DISTINCT FROM $1 AND priority = $2 AND delete_timestamp IS NULL",
            forum_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        sqlx::query!(
            "UPDATE rules
             SET priority = priority - 1
             WHERE forum_name IS NOT DISTINCT FROM $1 AND priority > $2 AND delete_timestamp IS NULL",
            forum_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn get_forum_ban_vec(
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans WHERE forum_name = $1",
            forum_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(user_ban_vec)
    }

    pub async fn remove_user_ban(
        ban_id: i64,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<UserBan, AppError> {
        let user_ban = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans WHERE ban_id = $1",
            ban_id
        )
            .fetch_one(db_pool)
            .await?;

        match &user_ban.forum_name {
            Some(forum_name) => grantor.check_permissions(forum_name, PermissionLevel::Ban),
            None => grantor.check_admin_role(AdminRole::Moderator),
        }?;

        sqlx::query!(
            "DELETE FROM user_bans WHERE ban_id = $1",
            ban_id
        )
            .execute(db_pool)
            .await?;

        Ok(user_ban)
    }

    pub async fn moderate_post(
        post_id: i64,
        moderated_body: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = if user.check_admin_role(AdminRole::Moderator).is_ok() {
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
        let comment = if user.check_admin_role(AdminRole::Moderator).is_ok() {
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
        if user.check_permissions(&forum_name, PermissionLevel::Moderate).is_ok() && user.user_id != user_id && !is_user_forum_moderator(user_id, forum_name, &db_pool).await? {
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
}

#[server]
pub async fn get_forum_rule_vec(
    forum_name: String
) -> Result<Vec<Rule>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let rule_vec = ssr::get_forum_rule_vec(&forum_name, &db_pool).await?;
    Ok(rule_vec)
}

#[server]
pub async fn add_rule(
    forum_name: Option<String>,
    priority: i16,
    title: String,
    description: String,
) -> Result<Rule, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user = check_user()?;
    let rule = ssr::add_rule(forum_name.as_ref().map(String::as_str), priority, &title, &description, &user, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn update_rule(
    forum_name: Option<String>,
    current_priority: i16,
    priority: i16,
    title: String,
    description: String,
) -> Result<Rule, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user = check_user()?;
    let rule = ssr::update_rule(forum_name.as_ref().map(String::as_str), current_priority, priority, &title, &description, &user, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn remove_rule(
    forum_name: Option<String>,
    priority: i16,
) -> Result<(), ServerFnError> {
    let db_pool = get_db_pool()?;
    let user = check_user()?;
    ssr::remove_rule(forum_name.as_ref().map(String::as_str), priority, &user, &db_pool).await?;
    Ok(())
}

#[server]
pub async fn get_forum_ban_vec(
    forum_name: String
) -> Result<Vec<UserBan>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let ban_vec = ssr::get_forum_ban_vec(&forum_name, &db_pool).await?;
    Ok(ban_vec)
}

#[server]
pub async fn remove_user_ban(
    ban_id: i64
) -> Result<(), ServerFnError> {
    let user = check_user()?;
    let db_pool = get_db_pool()?;
    let deleted_user_ban = ssr::remove_user_ban(ban_id, &user, &db_pool).await?;
    reload_user(deleted_user_ban.user_id)?;
    Ok(())
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

/// Function to moderate a comment and optionally ban its author
///
/// The ban is performed for the forum of the given comment and the duration is given by `ban_num_days`.
/// If `ban_num_days == None`, the duration of the ban is permanent.
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
        <div class="flex flex-col gap-5 w-full 2xl:w-1/2 mx-auto">
            <div class="text-2xl text-center">"Forum Cockpit"</div>
            <ModeratorPanel/>
            <ForumRulesPanel/>
            <BanPanel/>
        </div>
    }
}

/// Component to manage moderators
#[component]
pub fn ModeratorPanel() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let username_input = create_rw_signal(String::default());
    let select_ref = create_node_ref::<html::Select>();

    let set_role_action = create_server_action::<SetUserForumRole>();

    view! {
        <div class="flex flex-col gap-1 content-center w-full bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Moderators"</div>
            <TransitionUnpack resource=forum_state.forum_roles_resource let:forum_role_vec>
            {
                let forum_role_vec = forum_role_vec.clone();
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex gap-1 border-b border-base-content/20">
                            <div class="w-2/5 px-4 py-2 text-left font-bold">Username</div>
                            <div class="w-2/5 px-4 py-2 text-left font-bold">Role</div>
                        </div>
                        <For
                            each= move || forum_role_vec.clone().into_iter().enumerate()
                            key=|(_index, role)| (role.user_id, role.permission_level)
                            children=move |(_, role)| {
                                let username = store_value(role.username);
                                view! {
                                    <div
                                        class="flex gap-1 py-1 rounded hover:bg-base-content/20 transform active:scale-95 transition duration-250"
                                        on:click=move |_| {
                                            username_input.set(username.get_value());
                                            match select_ref.get_untracked() {
                                                Some(select_ref) => select_ref.set_selected_index(role.permission_level as i32),
                                                None => log::error!("Form permission level select failed to load."),
                                            };
                                        }
                                    >
                                        <div class="w-2/5 px-4 select-none">{username.get_value()}</div>
                                        <div class="w-2/5 px-4 select-none">{role.permission_level.to_string()}</div>
                                    </div>
                                }
                            }
                        />
                    </div>
                }
            }
            </TransitionUnpack>
            <AuthorizedShow permission_level=PermissionLevel::Manage>
                <PermissionLevelForm
                    forum_name
                    username_input
                    select_ref
                    set_role_action
                />
            </AuthorizedShow>
        </div>
    }
}

/// Component to set permission levels for a forum
#[component]
pub fn PermissionLevelForm(
    forum_name: Memo<String>,
    username_input: RwSignal<String>,
    select_ref: NodeRef<html::Select>,
    set_role_action: Action<SetUserForumRole, Result<UserForumRole, ServerFnError>>
) -> impl IntoView {
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);
    let matching_user_resource = create_resource(
        move || username_debounced.get(),
        move |username| async {
            if username.is_empty() {
                Ok(BTreeSet::<String>::default())
            } else {
                get_matching_username_set(username).await
            }
        },
    );
    let disable_submit = move || username_input.with(|username| username.is_empty());

    view! {
        <ActionForm action=set_role_action>
            <input
                name="forum_name"
                class="hidden"
                value=forum_name
            />
            <div class="flex gap-1 content-center">
                <div class="dropdown dropdown-end w-2/5">
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
                    <Show when=move || username_input.with(|username| !username.is_empty())>
                        <TransitionUnpack resource=matching_user_resource let:username_set>
                        {
                            let username_set = username_set.clone();
                            view! {
                                <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-200 rounded-box w-2/5">
                                    <For
                                        each= move || username_set.clone().into_iter().enumerate()
                                        key=|(_index, username)| username.clone()
                                        let:child
                                    >
                                        <li>
                                            <button type="button" value=child.1.clone() on:click=move |ev| username_input.update(|name| *name = event_target_value(&ev))>
                                                {child.1}
                                            </button>
                                        </li>
                                    </For>
                                </ul>
                            }
                        }
                        </TransitionUnpack>
                    </Show>
                </div>
                <EnumDropdown
                    name="permission_level"
                    enum_iter=PermissionLevel::iter()
                    _select_ref=select_ref
                />
                <button
                    type="submit"
                    class="btn btn-secondary"
                    disabled=disable_submit
                >
                    "Assign"
                </button>
            </div>
        </ActionForm>
    }
}

/// Component to manage forum rules
#[component]
pub fn ForumRulesPanel() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <div class="flex flex-col gap-1 content-center w-full bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Rules"</div>
            <TransitionUnpack resource=forum_state.forum_rules_resource let:forum_rule_vec>
            {
                let forum_rule_vec = forum_rule_vec.clone();
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="border-b border-base-content/20 pl-1">
                            <div class="w-11/12 flex gap-1">
                                <div class="w-1/12 py-2 font-bold">"N°"</div>
                                <div class="w-5/12 py-2 font-bold">"Title"</div>
                                <div class="w-6/12 py-2 font-bold">"Description"</div>
                            </div>
                        </div>
                        <For
                            each= move || forum_rule_vec.clone().into_iter().enumerate()
                            key=|(_index, rule)| rule.rule_id
                            children=move |(_, rule)| {
                                let rule = store_value(rule);
                                let show_edit_form = create_rw_signal(false);
                                view! {
                                    <div class="flex gap-1 py-1 rounded pl-1">
                                        <div class="w-11/12 flex gap-1">
                                            <div class="w-1/12 select-none">{rule.get_value().priority}</div>
                                            <div class="w-5/12 select-none">{rule.get_value().title}</div>
                                            <div class="w-6/12 select-none">{rule.get_value().description}</div>
                                        </div>
                                        <div class="w-1/12 flex gap-1">
                                            <button
                                                class="h-fit p-1 text-sm bg-secondary rounded-sm hover:bg-secondary/75 transform active:scale-90 transition duration-250"
                                                on:click=move |_| show_edit_form.update(|value| *value = !*value)
                                            >
                                                <EditIcon/>
                                            </button>
                                            <DeleteRuleButton rule/>
                                        </div>
                                    </div>
                                    <ModalDialog
                                        class="w-full max-w-xl"
                                        show_dialog=show_edit_form
                                    >
                                        <EditRuleForm rule show_form=show_edit_form/>
                                    </ModalDialog>
                                }
                            }
                        />
                    </div>
                }
            }
            </TransitionUnpack>
            <CreateRuleForm/>
        </div>
    }
}

/// Component to delete a forum rule
#[component]
pub fn DeleteRuleButton(
    rule: StoredValue<Rule>
) -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <AuthorizedShow permission_level=PermissionLevel::Manage>
            <ActionForm action=forum_state.remove_rule_action class="h-fit flex justify-center">
                <input
                    name="forum_name"
                    class="hidden"
                    value=forum_state.forum_name
                />
                <input
                    name="priority"
                    class="hidden"
                    value=rule.get_value().priority
                />
                <button class="p-1 rounded-sm bg-error hover:bg-error/75 transform active:scale-90 transition duration-250">
                    <DeleteIcon/>
                </button>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to edit a forum rule
#[component]
pub fn EditRuleForm(
    rule: StoredValue<Rule>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let rule = rule.get_value();
    let priority = create_rw_signal(rule.priority.to_string());
    let title = create_rw_signal(rule.title);
    let description = create_rw_signal(rule.description);
    let invalid_inputs = move || with!(|priority, title, description| priority.is_empty() || title.is_empty() || description.is_empty());

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit a rule"</div>
            <ActionForm action=forum_state.update_rule_action>
                <input
                    name="forum_name"
                    class="hidden"
                    value=forum_state.forum_name
                />
                <input
                    name="current_priority"
                    class="hidden"
                    value=rule.priority
                />
                <div class="flex flex-col gap-3 w-full">
                    <RuleInputs priority title description/>
                    <ModalFormButtons
                        disable_publish=invalid_inputs
                        show_form
                    />
                </div>
            </ActionForm>
        </div>
    }
}

/// Component to create a forum rule
#[component]
pub fn CreateRuleForm() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let show_dialog = create_rw_signal(false);
    let priority = create_rw_signal(String::default());
    let title = create_rw_signal(String::default());
    let description = create_rw_signal(String::default());
    let invalid_inputs = move || with!(|priority, title, description| priority.is_empty() || title.is_empty() || description.is_empty());

    view! {
        <button
            class="self-end w-fit h-fit p-1 text-sm bg-secondary rounded-sm hover:bg-secondary/75 transform active:scale-90 transition duration-250"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <PlusIcon/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Add a rule"</div>
                <ActionForm action=forum_state.add_rule_action>
                    <input
                        name="forum_name"
                        class="hidden"
                        value=forum_state.forum_name
                    />
                    <div class="flex flex-col gap-3 w-full">
                        <RuleInputs priority title description/>
                        <ModalFormButtons
                            disable_publish=invalid_inputs
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
            </div>
        </ModalDialog>
    }
}

/// Components with inputs to create or edit a rule
#[component]
pub fn RuleInputs(
    priority: RwSignal<String>,
    title: RwSignal<String>,
    description: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="flex gap-1 content-center">
            <input
                tabindex="0"
                type="number"
                name="priority"
                placeholder="N°"
                autocomplete="off"
                class="input input-bordered input-primary no-spinner px-1 w-1/12"
                value=priority
                on:input=move |ev| priority.set(event_target_value(&ev))
            />
            <FormTextEditor
                name="title"
                placeholder="Title"
                content=title
                class="w-5/12"
            />
            <FormTextEditor
                name="description"
                placeholder="Description"
                content=description
                class="w-6/12"
            />
        </div>
    }
}

/// Component to manage ban users
#[component]
pub fn BanPanel() -> impl IntoView {
    let forum_name = expect_context::<ForumState>().forum_name;

    let unban_action = create_server_action::<RemoveUserBan>();
    let banned_users_resource = create_resource(
        move || (forum_name.get(), unban_action.version().get()),
        move |(forum_name, _)| get_forum_ban_vec(forum_name)
    );

    view! {
        <div class="flex flex-col gap-1 content-center w-full bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Banned users"</div>
            <TransitionUnpack resource=banned_users_resource let:banned_user_vec>
            {
                let banned_user_vec = banned_user_vec.clone();
                view! {
                    <div class="flex flex-col gap-2">
                        <div class="flex border-b border-base-content/20">
                            <div class="w-2/5 px-6 py-2 text-left font-bold">Username</div>
                            <div class="w-2/5 px-6 py-2 text-left font-bold">Until</div>
                        </div>
                        <For
                            each= move || banned_user_vec.clone().into_iter().enumerate()
                            key=|(_index, ban)| (ban.user_id, ban.until_timestamp)
                            let:child
                        >
                            <div class="flex">
                                <div class="w-2/5 px-6">{child.1.username}</div>
                                <div class="w-2/5 px-6">{
                                    match child.1.until_timestamp {
                                        Some(until_timestamp) => until_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
                                        None => String::from("Permanent"),
                                    }
                                }</div>
                                <AuthorizedShow permission_level=PermissionLevel::Ban>
                                    <ActionForm action=unban_action class="w-1/5 flex justify-center">
                                        <input
                                            name="ban_id"
                                            class="hidden"
                                            value=child.1.ban_id
                                        />
                                        <button class="p-1 rounded-sm bg-error hover:bg-error/75 transform active:scale-90 transition duration-250">
                                            <DeleteIcon/>
                                        </button>
                                    </ActionForm>
                                </AuthorizedShow>
                            </div>
                        </For>
                    </div>
                }
            }
            </TransitionUnpack>
        </div>
    }
}

/// Component to moderate a post
#[component]
pub fn ModerateButton(show_dialog: RwSignal<bool>) -> impl IntoView {
    let edit_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <AuthorizedShow permission_level=PermissionLevel::Moderate>
            <button
                class=edit_button_class
                aria-expanded=move || show_dialog.get().to_string()
                aria-haspopup="dialog"
                on:click=move |_| show_dialog.set(true)
            >
                <HammerIcon/>
            </button>
        </AuthorizedShow>
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
    let forum_state = expect_context::<ForumState>();

    let moderate_text = create_rw_signal(String::new());
    let is_text_empty = move || moderate_text.with(|moderate_text: &String| moderate_text.is_empty());

    let moderate_result = forum_state.moderate_post_action.value();
    let has_error = move || moderate_result.with(|val| matches!(val, Some(Err(_))));

    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a post"</div>
                <ActionForm action=forum_state.moderate_post_action>
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
        <AuthorizedShow permission_level=PermissionLevel::Ban>
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
        </AuthorizedShow>
    }
}