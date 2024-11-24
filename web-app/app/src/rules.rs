use leptos::html;
use leptos::prelude::*;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};

use crate::editor::{FormTextEditor, TextareaData};
use crate::errors::AppError;
use crate::forum::ForumState;
use crate::icons::{DeleteIcon, EditIcon, PlusIcon};
use crate::role::{AuthorizedShow, PermissionLevel};
use crate::unpack::ArcTransitionUnpack;
use crate::widget::{ModalDialog, ModalFormButtons};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::check_user,
};


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
    use crate::errors::AppError;
    use crate::role::{AdminRole, PermissionLevel};
    use crate::rules::Rule;
    use crate::user::User;
    use sqlx::PgPool;

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
}

#[server]
pub async fn get_rule_by_id(
    rule_id: i64
) -> Result<Rule, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let rule = ssr::load_rule_by_id(rule_id, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn get_forum_rule_vec(
    forum_name: String
) -> Result<Vec<Rule>, ServerFnError<AppError>> {
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
) -> Result<Rule, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
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
) -> Result<Rule, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let rule = ssr::update_rule(forum_name.as_ref().map(String::as_str), current_priority, priority, &title, &description, &user, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn remove_rule(
    forum_name: Option<String>,
    priority: i16,
) -> Result<(), ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::remove_rule(forum_name.as_deref(), priority, &user, &db_pool).await?;
    Ok(())
}

/// Component to manage forum rules
#[component]
pub fn ForumRulesPanel() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Rules"</div>
            <ArcTransitionUnpack resource=forum_state.forum_rules_resource let:forum_rule_vec>
                <div class="flex flex-col gap-1">
                    <div class="border-b border-base-content/20 pl-1">
                        <div class="w-5/6 flex gap-1">
                            <div class="w-1/12 py-2 font-bold">"N°"</div>
                            <div class="w-5/12 py-2 font-bold">"Title"</div>
                            <div class="w-6/12 py-2 font-bold">"Description"</div>
                        </div>
                    </div>
                    <For
                        each= move || (*forum_rule_vec).clone().into_iter().enumerate()
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
            </ArcTransitionUnpack>
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
    let forum_name = forum_state.forum_name;
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
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
                    value=rule.with_value(|rule| rule.priority)
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
    let rule_priority = rule.with_value(|rule| rule.priority);
    let priority = RwSignal::new(rule_priority.to_string());
    let title_ref = NodeRef::<html::Textarea>::new();
    let title_autosize = use_textarea_autosize(title_ref);
    let title_data = TextareaData {
        content: title_autosize.content,
        set_content: title_autosize.set_content,
        textarea_ref: title_ref,
    };
    let description_ref = NodeRef::<html::Textarea>::new();
    let desc_autosize = use_textarea_autosize(description_ref);
    let description_data = TextareaData {
        content: desc_autosize.content,
        set_content: desc_autosize.set_content,
        textarea_ref: description_ref,
    };
    let invalid_inputs = Signal::derive(move || {
        priority.read().is_empty() || title_autosize.content.read().is_empty() || description_data.content.read().is_empty()
    });

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
                    value=rule_priority
                />
                <div class="flex flex-col gap-3 w-full">
                    <RuleInputs priority title_data description_data/>
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
    let title_ref = NodeRef::<html::Textarea>::new();
    let title_autosize = use_textarea_autosize(title_ref);
    let title_data = TextareaData {
        content: title_autosize.content,
        set_content: title_autosize.set_content,
        textarea_ref: title_ref,
    };
    let description_ref = NodeRef::<html::Textarea>::new();
    let desc_autosize = use_textarea_autosize(description_ref);
    let description_data = TextareaData {
        content: desc_autosize.content,
        set_content: desc_autosize.set_content,
        textarea_ref: description_ref,
    };
    let invalid_inputs = Signal::derive(move || {
        priority.read().is_empty() || title_autosize.content.read().is_empty() || description_data.content.read().is_empty()
    });

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
                        <RuleInputs priority title_data description_data/>
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
    title_data: TextareaData,
    description_data: TextareaData,
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
                data=title_data
                class="w-5/12"
            />
            <FormTextEditor
                name="description"
                placeholder="Description"
                data=description_data
                class="w-6/12"
            />
        </div>
    }
}
