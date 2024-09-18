use std::collections::BTreeSet;

use chrono::SecondsFormat;
use leptos::html;
use leptos::prelude::*;
use leptos_use::signal_debounced;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::app::{GlobalState, LoginWindow};
use crate::content::Content;
use crate::editor::FormTextEditor;
use crate::error_template::ErrorDisplay;
use crate::forum::{Forum, ForumState};
use crate::icons::{DeleteIcon, EditIcon, MagnifierIcon, PlusIcon, SaveIcon};
use crate::moderation::{get_moderation_info, ModerationInfoDialog};
use crate::role::{AuthorizedShow, PermissionLevel, SetUserForumRole};
use crate::unpack::{SuspenseUnpack, TransitionUnpack};
use crate::user::get_matching_username_set;
use crate::widget::{EnumDropdown, ModalDialog, ModalFormButtons};
#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::{check_user, reload_user},
};

pub const MANAGE_FORUM_ROUTE: &str = "/manage";
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
    pub post_id: i64,
    pub comment_id: Option<i64>,
    pub infringed_rule_id: i64,
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

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModerationInfo {
    pub rule: Rule,
    pub content: Content,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::errors::AppError;
    use crate::forum_management::{Rule, UserBan};
    use crate::role::{AdminRole, PermissionLevel};
    use crate::user::User;

    pub async fn is_user_forum_moderator(
        user_id: i64,
        forum: &str,
        db_pool: &PgPool,
    ) -> Result<bool, AppError> {
        match User::get(user_id, db_pool).await {
            Some(user) => Ok(user.check_permissions(forum, PermissionLevel::Moderate).is_ok()),
            None => Err(AppError::InternalServerError(format!("Could not find user with id = {user_id}"))),
        }
    }
    
    pub async fn load_rule_by_id(
        rule_id: i64,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        let rule = sqlx::query_as!(
            Rule,
            "SELECT * FROM rules
            WHERE rule_id = $1",
            rule_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(rule)
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
        username_prefix: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans
            WHERE forum_name = $1 AND
                  username like $2
            ORDER BY until_timestamp DESC",
            forum_name,
            format!("{username_prefix}%"),
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
}

#[server]
pub async fn get_rule_by_id(
    rule_id: i64
) -> Result<Rule, ServerFnError> {
    let db_pool = get_db_pool()?;
    let rule = ssr::load_rule_by_id(rule_id, &db_pool).await?;
    Ok(rule)
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
    forum_name: String,
    username_prefix: String,
) -> Result<Vec<UserBan>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let ban_vec = ssr::get_forum_ban_vec(&forum_name, &username_prefix, &db_pool).await?;
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

/// Component to guard the forum cockpit
#[component]
pub fn ForumCockpitGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_name = expect_context::<ForumState>().forum_name;
    view! {
        <SuspenseUnpack resource=state.user let:user>
        {
            match user {
                Some(user) => {
                    match forum_name.with_untracked(|forum_name| user.check_permissions(forum_name, PermissionLevel::Moderate)) {
                        Ok(_) => view! { <ForumCockpit/> }.into_any(),
                        Err(error) => view! { <ErrorDisplay error/> }.into_any(),
                    }
                },
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </SuspenseUnpack>
    }.into_any()
}

/// Component to manage a forum
#[component]
pub fn ForumCockpit() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-5 overflow-y-auto w-full 2xl:w-1/2 mx-auto">
            <div class="text-2xl text-center">"Forum Cockpit"</div>
            <ForumDescriptionDialog/>
            <ModeratorPanel/>
            <ForumRulesPanel/>
            <BanPanel/>
        </div>
    }.into_any()
}

/// Component to edit a forum's description
#[component]
pub fn ForumDescriptionDialog() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <AuthorizedShow permission_level=PermissionLevel::Manage>
            <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit max-h-full overflow-y-auto bg-base-200 p-2 rounded">
                <div class="text-xl text-center">"Forum description"</div>
                <SuspenseUnpack resource=forum_state.forum_resource let:forum>
                    <ForumDescriptionForm forum=forum.clone()/>
                </SuspenseUnpack>
            </div>
        </AuthorizedShow>
    }
}

/// Component to edit a forum's description
#[component]
pub fn ForumDescriptionForm(
    forum: Forum,
) -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let description = RwSignal::new(forum.description.clone());
    let disable_submit = move || description.with(|description| description.is_empty());
    view! {
        <ActionForm
            action=forum_state.update_forum_desc_action
            attr:class="flex flex-col gap-1"
        >
            <input
                name="forum_name"
                class="hidden"
                value=forum_state.forum_name
            />
            <FormTextEditor
                name="description"
                placeholder="Description"
                content=description
            />
            <button
                type="submit"
                class="btn btn-secondary btn-sm p-1 self-end"
                disabled=disable_submit
            >
                <SaveIcon/>
            </button>
        </ActionForm>
    }
}

/// Component to manage moderators
#[component]
pub fn ModeratorPanel() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let username_input = RwSignal::new(String::default());
    let select_ref = NodeRef::<html::Select>::new();

    let set_role_action = ServerAction::<SetUserForumRole>::new();

    view! {
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit max-h-full overflow-y-auto bg-base-200 p-2 rounded">
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
                                let username = StoredValue::new(role.username);
                                view! {
                                    <div
                                        class="flex gap-1 py-1 rounded hover:bg-base-content/20 active:scale-95 transition duration-250"
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
    set_role_action: ServerAction<SetUserForumRole>
) -> impl IntoView {
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);
    let matching_user_resource = Resource::new(
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
                                                {child.1.clone()}
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
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit max-h-full overflow-y-auto bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Rules"</div>
            <TransitionUnpack resource=forum_state.forum_rules_resource let:forum_rule_vec>
            {
                let forum_rule_vec = forum_rule_vec.clone();
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="border-b border-base-content/20 pl-1">
                            <div class="w-5/6 flex gap-1">
                                <div class="w-1/12 py-2 font-bold">"N°"</div>
                                <div class="w-5/12 py-2 font-bold">"Title"</div>
                                <div class="w-6/12 py-2 font-bold">"Description"</div>
                            </div>
                        </div>
                        <For
                            each= move || forum_rule_vec.clone().into_iter().enumerate()
                            key=|(_index, rule)| rule.rule_id
                            children=move |(_, rule)| {
                                let rule = StoredValue::new(rule);
                                let show_edit_form = RwSignal::new(false);
                                view! {
                                    <div class="flex gap-1 justify-between rounded pl-1">
                                        <div class="w-5/6 flex gap-1">
                                            <div class="w-1/12 select-none">{rule.get_value().priority}</div>
                                            <div class="w-5/12 select-none">{rule.get_value().title}</div>
                                            <div class="w-6/12 select-none">{rule.get_value().description}</div>
                                        </div>
                                        <div class="flex gap-1 justify-end">
                                            <button
                                                class="h-fit p-1 text-sm bg-secondary rounded-sm hover:bg-secondary/75 active:scale-90 transition duration-250"
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
            <ActionForm
                action=forum_state.remove_rule_action
                attr:class="h-fit flex justify-center"
            >
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
                <button class="p-1 rounded-sm bg-error hover:bg-error/75 active:scale-90 transition duration-250">
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
    let priority = RwSignal::new(rule.priority.to_string());
    let title = RwSignal::new(rule.title);
    let description = RwSignal::new(rule.description);
    let invalid_inputs = move || priority.read().is_empty() || title.read().is_empty() || description.read().is_empty();

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
    let show_dialog = RwSignal::new(false);
    let priority = RwSignal::new(String::default());
    let title = RwSignal::new(String::default());
    let description = RwSignal::new(String::default());
    let invalid_inputs = move || priority.read().is_empty() || title.read().is_empty() || description.read().is_empty();

    view! {
        <button
            class="self-end p-1 bg-secondary rounded-sm hover:bg-secondary/75 active:scale-90 transition duration-250"
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
                <ActionForm
                    action=forum_state.add_rule_action
                    on:submit=move |_| show_dialog.set(false)
                >
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
    let username_input = RwSignal::new(String::default());
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);

    let unban_action = ServerAction::<RemoveUserBan>::new();
    let banned_users_resource = Resource::new(
        move || (forum_name.get(), username_debounced.get(), unban_action.version().get()),
        move |(forum_name, username, _)| get_forum_ban_vec(forum_name, username)
    );

    view! {
        <div class="shrink-0 flex flex-col gap-1 content-center w-full max-h-full overflow-y-auto bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Banned users"</div>
            <div class="flex flex-col gap-1">
                <div class="flex flex-col border-b border-base-content/20">
                    <div class="flex">
                        <input
                            class="input input-bordered input-primary px-6 w-2/5"
                            placeholder="Username"
                            value=username_input
                            on:input=move |ev| username_input.update(|user_prefix: &mut String| *user_prefix = event_target_value(&ev))
                        />
                        <div class="w-2/5 px-6 py-2 text-left font-bold">Until</div>
                    </div>
                </div>
                <TransitionUnpack resource=banned_users_resource let:banned_user_vec>
                {
                    let banned_user_vec = banned_user_vec.clone();
                    view! {
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
                                <div class="w-1/5 flex justify-end gap-1">
                                    <BanInfoButton
                                        post_id=child.1.post_id
                                        comment_id=child.1.comment_id
                                    />
                                    <AuthorizedShow permission_level=PermissionLevel::Ban>
                                        <ActionForm action=unban_action>
                                            <input
                                                name="ban_id"
                                                class="hidden"
                                                value=child.1.ban_id
                                            />
                                            <button class="p-1 h-full rounded-sm bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                                                <DeleteIcon/>
                                            </button>
                                        </ActionForm>
                                    </AuthorizedShow>
                                </div>
                            </div>
                        </For>
                    }
                }
                </TransitionUnpack>
            </div>
        </div>
    }
}

/// Component to display a button opening a modal dialog with a ban's details
#[component]
pub fn BanInfoButton(
    post_id: i64,
    comment_id: Option<i64>,
) -> impl IntoView {
    let show_dialog = RwSignal::new(false);

    view! {
        <button
            class="p-1 h-full bg-secondary rounded-sm hover:bg-secondary/75 active:scale-90 transition duration-250"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <MagnifierIcon/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            {
                let ban_detail_resource = Resource::new(
                    move || (),
                    move |_| get_moderation_info(post_id, comment_id)
                );
                view! {
                    <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                        <SuspenseUnpack resource=ban_detail_resource let:moderation_info>
                            <ModerationInfoDialog
                                content=moderation_info.content.clone()
                                rule=moderation_info.rule.clone()
                            />
                            <button
                                type="button"
                                class="p-1 h-full rounded-sm bg-error hover:bg-error/75 active:scale-95 transition duration-250"
                                on:click=move |_| show_dialog.set(false)
                            >
                                "Close"
                            </button>
                        </SuspenseUnpack>
                    </div>
                }
            }
        </ModalDialog>
    }
}