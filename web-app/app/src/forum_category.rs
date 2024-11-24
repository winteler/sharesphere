use crate::editor::{FormTextEditor, TextareaData};
use crate::errors::AppError;
use crate::form::FormCheckbox;
use crate::forum::ForumState;
use crate::icons::{DeleteIcon, PauseIcon, PlayIcon, SaveIcon};
use crate::role::{AuthorizedShow, PermissionLevel};
use crate::unpack::TransitionUnpack;
use leptos::html;
use leptos::prelude::*;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::check_user,
};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ForumCategory {
    pub category_id: i64,
    pub forum_id: i64,
    pub forum_name: String,
    pub category_name: String,
    pub description: String,
    pub is_active: bool,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::errors::AppError;
    use crate::forum_category::ForumCategory;
    use crate::role::PermissionLevel;
    use crate::user::User;
    use sqlx::PgPool;

    pub const CATEGORY_NOT_DELETED_STR: &str = "Category was not deleted, it either doesn't exist or is used.";
    
    pub async fn get_forum_category_vec(
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<ForumCategory>, AppError> {
        let forum_category_vec = sqlx::query_as!(
            ForumCategory,
            "SELECT * FROM forum_categories
            WHERE forum_name = $1
            ORDER BY is_active DESC, category_name",
            forum_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_category_vec)
    }

    pub async fn set_forum_category(
        forum_name: &str,
        category_name: &str,
        description: &str,
        is_active: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<ForumCategory, AppError> {
        user.check_permissions(forum_name, PermissionLevel::Manage)?;

        let category = sqlx::query_as!(
            ForumCategory,
            "INSERT INTO forum_categories
            (forum_id, forum_name, category_name, description, is_active, creator_id)
            VALUES (
                (SELECT forum_id FROM forums WHERE forum_name = $1),
                $1, $2, $3, $4, $5
            ) ON CONFLICT (forum_id, category_name) DO UPDATE
                SET description = EXCLUDED.description,
                    is_active = EXCLUDED.is_active,
                    timestamp = CURRENT_TIMESTAMP
            RETURNING *",
            forum_name,
            category_name,
            description,
            is_active,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(category)
    }

    pub async fn delete_forum_category(
        forum_name: &str,
        category_name: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(forum_name, PermissionLevel::Manage)?;

        let result = sqlx::query!(
            "DELETE FROM forum_categories c
             WHERE forum_name = $1 AND category_name = $2 AND NOT EXISTS (
                SELECT 1 FROM posts p WHERE p.category_id = c.category_id
             )",
            forum_name,
            category_name,
        )
            .execute(db_pool)
            .await?;
        
        match result.rows_affected() {
            0 => Err(AppError::InternalServerError(String::from(CATEGORY_NOT_DELETED_STR))),
            1 => Ok(()),
            count => Err(AppError::InternalServerError(format!("Expected 1 category to be deleted, got {count} instead"))),
        }
    }
}

#[server]
pub async fn get_forum_category_vec(
    forum_name: String,
) -> Result<Vec<ForumCategory>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let forum_category_vec = ssr::get_forum_category_vec(&forum_name, &db_pool).await?;
    Ok(forum_category_vec)
}

#[server]
pub async fn set_forum_category(
    forum_name: String,
    category_name: String,
    description: String,
    is_active: bool,
) -> Result<ForumCategory, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let forum_category = ssr::set_forum_category(&forum_name, &category_name, &description, is_active, &user, &db_pool).await?;
    Ok(forum_category)
}

#[server]
pub async fn delete_forum_category(
    forum_name: String,
    category_name: String,
) -> Result<(), ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::delete_forum_category(&forum_name, &category_name, &user, &db_pool).await?;
    Ok(())
}

/// Component to manage forum categories
#[component]
pub fn ForumCategoriesDialog() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;

    let category_input = RwSignal::new(String::new());
    let activated_input = RwSignal::new(true);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let description_autosize = use_textarea_autosize(textarea_ref);
    let description_data = TextareaData {
        content: description_autosize.content,
        set_content: description_autosize.set_content,
        textarea_ref
    };
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
                <div class="text-xl text-center">"Forum categories"</div>
                <div class="flex flex-col">
                    <div class="border-b border-base-content/20 pl-2">
                        <div class="w-5/6 flex gap-1">
                            <div class="w-2/6 py-2 font-bold">"Category"</div>
                            <div class="w-3/6 py-2 font-bold">"Description"</div>
                            <div class="w-20 py-2 font-bold text-center">"Activated"</div>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1 pl-2 py-1">
                        <TransitionUnpack resource=forum_state.forum_categories_resource let:forum_category_vec>
                        {
                            forum_category_vec.iter().map(|forum_category| {
                                let category_name = forum_category.category_name.clone();
                                let description = forum_category.description.clone();
                                let is_active = forum_category.is_active;
                                view! {
                                    <div
                                        class="flex justify-between align-center"
                                    >
                                        <div
                                            class="w-5/6 flex gap-1 p-1 rounded hover:bg-base-content/20 active:scale-95 transition duration-250"
                                            on:click=move |_| {
                                                category_input.set(category_name.clone());
                                                description_data.set_content.set(description.clone());
                                                if let Some(textarea_ref) = textarea_ref.get() {
                                                    textarea_ref.set_value(&description);
                                                }
                                                activated_input.set(is_active);
                                            }
                                        >
                                            <div class="w-2/6 select-none">{category_name.clone()}</div>
                                            <div class="w-3/6 select-none whitespace-pre-wrap">{description.clone()}</div>
                                            <div class="w-20 flex justify-center">
                                                {
                                                    match is_active {
                                                        true => view! { <PlayIcon/> }.into_any(),
                                                        false => view! { <PauseIcon/> }.into_any(),
                                                    }
                                                }
                                            </div>
                                        </div>
                                        <DeleteCategoryButton category_name=forum_category.category_name.clone()/>
                                    </div>
                                }
                            }).collect_view()
                        }
                        </TransitionUnpack>
                    </div>
                    <SetCategoryForm category_input activated_input description_data/>
                </div>
            </div>
        </AuthorizedShow>
    }
}

/// Component to set permission levels for a forum
#[component]
pub fn SetCategoryForm(
    category_input: RwSignal<String>,
    activated_input: RwSignal<bool>,
    description_data: TextareaData,
) -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let disable_submit = move || category_input.read().is_empty() && description_data.content.read().is_empty();

    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            <ActionForm action=forum_state.set_forum_category_action>
                <input
                    name="forum_name"
                    class="hidden"
                    value=forum_name
                />
                <div class="w-full flex gap-1 justify-between">
                    <div class="flex gap-1 content-center w-5/6">
                        <input
                            tabindex="0"
                            type="text"
                            name="category_name"
                            placeholder="Category"
                            autocomplete="off"
                            class="input input-bordered input-primary w-2/6"
                            on:input=move |ev| {
                                category_input.set(event_target_value(&ev));
                            }
                            prop:value=category_input
                        />
                        <FormTextEditor
                            name="description"
                            placeholder="Description"
                            data=description_data
                            class="w-3/6"
                        />
                        <FormCheckbox name="is_active" is_checked=activated_input class="w-20 self-center flex justify-center"/>
                    </div>
                    <button
                        type="submit"
                        disabled=disable_submit
                        class="btn btn-secondary btn-sm p-1 self-center"
                    >
                        <SaveIcon/>
                    </button>
                </div>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to delete a forum category
#[component]
pub fn DeleteCategoryButton(
    category_name: String,
) -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let category_name = StoredValue::new(category_name);
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            <ActionForm
                action=forum_state.delete_forum_category_action
                attr:class="h-fit flex justify-center"
            >
                <input
                    name="forum_name"
                    class="hidden"
                    value=forum_state.forum_name
                />
                <input
                    name="category_name"
                    class="hidden"
                    value=category_name.get_value()
                />
                <button class="p-1 rounded-sm bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                    <DeleteIcon/>
                </button>
            </ActionForm>
        </AuthorizedShow>
    }
}