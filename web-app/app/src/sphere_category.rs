use leptos::html;
use leptos::prelude::*;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};

use crate::colors::{Color, ColorIndicator, ColorSelect};
use crate::editor::{FormTextEditor, TextareaData};
use crate::errors::AppError;
use crate::form::FormCheckbox;
use crate::icons::{DeleteIcon, PauseIcon, PlayIcon, SaveIcon};
use crate::role::{AuthorizedShow, PermissionLevel};
use crate::sphere::SphereState;
use crate::unpack::TransitionUnpack;


#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::check_user,
};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategoryHeader {
    pub category_name: String,
    pub category_color: Color,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategory {
    pub category_id: i64,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub category_name: String,
    pub category_color: Color,
    pub description: String,
    pub is_active: bool,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<SphereCategory> for SphereCategoryHeader {
    fn from(sphere_category: SphereCategory) -> Self {
        SphereCategoryHeader {
            category_name: sphere_category.category_name,
            category_color: sphere_category.category_color,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::errors::AppError;
    use crate::role::PermissionLevel;
    use crate::sphere_category::{Color, SphereCategory};
    use crate::user::User;
    use sqlx::PgPool;

    pub const CATEGORY_NOT_DELETED_STR: &str = "Category was not deleted, it either doesn't exist or is used.";
    
    pub async fn get_sphere_category_vec(
        sphere_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereCategory>, AppError> {
        let sphere_category_vec = sqlx::query_as!(
            SphereCategory,
            "SELECT * FROM sphere_categories
            WHERE sphere_name = $1
            ORDER BY is_active DESC, category_name",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_category_vec)
    }

    pub async fn set_sphere_category(
        sphere_name: &str,
        category_name: &str,
        category_color: Color,
        description: &str,
        is_active: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<SphereCategory, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;

        let category = sqlx::query_as!(
            SphereCategory,
            "INSERT INTO sphere_categories
            (sphere_id, sphere_name, category_name, category_color, description, is_active, creator_id)
            VALUES (
                (SELECT sphere_id FROM spheres WHERE sphere_name = $1),
                $1, $2, $3, $4, $5, $6
            ) ON CONFLICT (sphere_id, category_name) DO UPDATE
                SET description = EXCLUDED.description,
                    category_color = EXCLUDED.category_color,
                    is_active = EXCLUDED.is_active,
                    timestamp = CURRENT_TIMESTAMP
            RETURNING *",
            sphere_name,
            category_name,
            category_color as i32,
            description,
            is_active,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(category)
    }

    pub async fn delete_sphere_category(
        sphere_name: &str,
        category_name: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;

        let result = sqlx::query!(
            "DELETE FROM sphere_categories c
             WHERE sphere_name = $1 AND category_name = $2 AND NOT EXISTS (
                SELECT 1 FROM posts p WHERE p.category_id = c.category_id
             )",
            sphere_name,
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
pub async fn get_sphere_category_vec(
    sphere_name: String,
) -> Result<Vec<SphereCategory>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_category_vec = ssr::get_sphere_category_vec(&sphere_name, &db_pool).await?;
    Ok(sphere_category_vec)
}

#[server]
pub async fn set_sphere_category(
    sphere_name: String,
    category_name: String,
    category_color: Color,
    description: String,
    is_active: bool,
) -> Result<SphereCategory, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let sphere_category = ssr::set_sphere_category(&sphere_name, &category_name, category_color, &description, is_active, &user, &db_pool).await?;
    Ok(sphere_category)
}

#[server]
pub async fn delete_sphere_category(
    sphere_name: String,
    category_name: String,
) -> Result<(), ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::delete_sphere_category(&sphere_name, &category_name, &user, &db_pool).await?;
    Ok(())
}

/// Component to display a badge with sphere category's name
#[component]
pub fn SphereCategoryBadge(
    category_header: SphereCategoryHeader,
) -> impl IntoView {
    let class = format!(
        "flex items-center {} px-2 pt-1 pb-1.5 rounded-full text-sm leading-none",
        category_header.category_color.to_bg_class()
    );
    view! {
        <div class=class>{category_header.category_name}</div>
    }
}

/// Component to manage sphere categories
#[component]
pub fn SphereCategoriesDialog() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;

    let category_input = RwSignal::new(String::new());
    let color_input = RwSignal::new(Color::None);
    let activated_input = RwSignal::new(true);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let description_autosize = use_textarea_autosize(textarea_ref);
    let description_data = TextareaData {
        content: description_autosize.content,
        set_content: description_autosize.set_content,
        textarea_ref
    };
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
                <div class="text-xl text-center">"Sphere categories"</div>
                <div class="flex flex-col">
                    <div class="border-b border-base-content/20 pl-2">
                        <div class="w-5/6 flex gap-1">
                            <div class="w-3/12 py-2 font-bold">"Category"</div>
                            <div class="w-1/12 py-2 font-bold">"Color"</div>
                            <div class="w-3/6 py-2 font-bold">"Description"</div>
                            <div class="w-20 py-2 font-bold text-center">"Activated"</div>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1 pl-2 py-1">
                        <TransitionUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
                        {
                            sphere_category_vec.iter().map(|sphere_category| {
                                let category_name = sphere_category.category_name.clone();
                                let color = sphere_category.category_color;
                                let description = sphere_category.description.clone();
                                let is_active = sphere_category.is_active;
                                view! {
                                    <div
                                        class="flex justify-between items-center"
                                    >
                                        <div
                                            class="w-5/6 flex items-center gap-1 p-1 rounded hover:bg-base-content/20 active:scale-95 transition duration-250"
                                            on:click=move |_| {
                                                category_input.set(category_name.clone());
                                                color_input.set(color);
                                                description_data.set_content.set(description.clone());
                                                if let Some(textarea_ref) = textarea_ref.get() {
                                                    textarea_ref.set_value(&description);
                                                }
                                                activated_input.set(is_active);
                                            }
                                        >
                                            <div class="w-3/12 select-none">{category_name.clone()}</div>
                                            <div class="w-1/12 h-fit"><ColorIndicator color/></div>
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
                                        <DeleteCategoryButton category_name=sphere_category.category_name.clone()/>
                                    </div>
                                }
                            }).collect_view()
                        }
                        </TransitionUnpack>
                    </div>
                    <SetCategoryForm category_input color_input activated_input description_data/>
                </div>
            </div>
        </AuthorizedShow>
    }
}

/// Component to set permission levels for a sphere
#[component]
pub fn SetCategoryForm(
    category_input: RwSignal<String>,
    color_input: RwSignal<Color>,
    activated_input: RwSignal<bool>,
    description_data: TextareaData,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let disable_submit = move || category_input.read().is_empty() && description_data.content.read().is_empty();

    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm action=sphere_state.set_sphere_category_action>
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_name
                />
                <div class="w-full flex gap-1 justify-between items-stretch pl-2">
                    <div class="flex gap-1 items-center w-5/6 p-1">
                        <input
                            tabindex="0"
                            type="text"
                            name="category_name"
                            placeholder="Category"
                            autocomplete="off"
                            class="input input-bordered input-primary w-3/12"
                            on:input=move |ev| {
                                category_input.set(event_target_value(&ev));
                            }
                            prop:value=category_input
                        />
                        <ColorSelect name="category_color" color_input class="w-1/12"/>
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

/// Component to delete a sphere category
#[component]
pub fn DeleteCategoryButton(
    category_name: String,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let category_name = StoredValue::new(category_name);
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm
                action=sphere_state.delete_sphere_category_action
                attr:class="h-fit flex justify-center"
            >
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_state.sphere_name
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