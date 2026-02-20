use leptos::either::Either;
use leptos::prelude::*;
use leptos_fluent::{move_tr};
use serde::{Deserialize, Serialize};

use sharesphere_utils::colors::Color;
use sharesphere_utils::errors::AppError;
use sharesphere_utils::unpack::TransitionUnpack;
use sharesphere_utils::widget::{DropdownButton, RotatingArrow};

use crate::filter::SphereCategoryToggle;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::{check_user},
        session::ssr::get_db_pool,
    },
    sharesphere_utils::constants::{MAX_CATEGORY_DESCRIPTION_LENGTH, MAX_CATEGORY_NAME_LENGTH},
    sharesphere_utils::checks::{check_sphere_name, check_string_length},
};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategory {
    pub category_id: i64,
    pub sphere_id: i64,
    pub category_name: String,
    pub category_color: Color,
    pub description: String,
    pub is_active: bool,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategoryHeader {
    pub category_name: String,
    pub category_color: Color,
}

impl From<SphereCategory> for SphereCategoryHeader {
    fn from(sphere_category: SphereCategory) -> Self {
        SphereCategoryHeader {
            category_name: sphere_category.category_name,
            category_color: sphere_category.category_color,
        }
    }
}

impl From<&SphereCategory> for SphereCategoryHeader {
    fn from(sphere_category: &SphereCategory) -> Self {
        SphereCategoryHeader {
            category_name: sphere_category.category_name.clone(),
            category_color: sphere_category.category_color,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::user::User;
    use sharesphere_utils::colors::Color;
    use sharesphere_utils::errors::AppError;
    use crate::sphere_category::SphereCategory;

    pub const CATEGORY_NOT_DELETED_STR: &str = "Category was not deleted, it either doesn't exist or is used.";

    pub async fn get_sphere_category_vec(
        sphere_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereCategory>, AppError> {
        let sphere_category_vec = sqlx::query_as!(
            SphereCategory,
            "SELECT sc.* FROM sphere_categories sc
            JOIN spheres s ON s.sphere_id = sc.sphere_id
            WHERE s.sphere_name = $1
            ORDER BY sc.is_active DESC, sc.category_name",
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
        user.check_sphere_permissions_by_name(sphere_name, PermissionLevel::Manage)?;

        let category = sqlx::query_as!(
            SphereCategory,
            "INSERT INTO sphere_categories
            (sphere_id, category_name, category_color, description, is_active, creator_id)
            VALUES (
                (SELECT sphere_id FROM spheres WHERE sphere_name = $1),
                $2, $3, $4, $5, $6
            ) ON CONFLICT (sphere_id, category_name) DO UPDATE
                SET description = EXCLUDED.description,
                    category_color = EXCLUDED.category_color,
                    is_active = EXCLUDED.is_active,
                    timestamp = NOW()
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
        user.check_sphere_permissions_by_name(sphere_name, PermissionLevel::Manage)?;

        let result = sqlx::query!(
            "DELETE FROM sphere_categories c
             WHERE sphere_id = (
                    SELECT sphere_id FROM spheres WHERE sphere_name = $1
                ) AND category_name = $2 AND NOT EXISTS (
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
) -> Result<Vec<SphereCategory>, AppError> {
    check_sphere_name(&sphere_name)?;
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
) -> Result<SphereCategory, AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&category_name, "Category name", MAX_CATEGORY_NAME_LENGTH, false)?;
    check_string_length(&description, "Category description", MAX_CATEGORY_DESCRIPTION_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let sphere_category = ssr::set_sphere_category(&sphere_name, &category_name, category_color, &description, is_active, &user, &db_pool).await?;
    Ok(sphere_category)
}

#[server]
pub async fn delete_sphere_category(
    sphere_name: String,
    category_name: String,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&category_name, "Category name", MAX_CATEGORY_NAME_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::delete_sphere_category(&sphere_name, &category_name, &user, &db_pool).await?;
    Ok(())
}

/// Component to display a badge with sphere category's name
#[component]
pub fn SphereCategoryBadge(
    #[prop(into)]
    category_header: SphereCategoryHeader,
) -> impl IntoView {
    let class = format!(
        "flex items-center {} px-2 pt-1 pb-1.5 rounded-full text-xs lg:text-sm leading-none",
        category_header.category_color.to_bg_class()
    );
    view! {
        <div class=class>{category_header.category_name}</div>
    }
}

/// Dialog to select a sphere category
#[component]
pub fn SphereCategoryDropdown(
    category_vec_resource: Resource<Result<Vec<SphereCategory>, AppError>>,
    #[prop(default = None)]
    init_category_id: Option<i64>,
    #[prop(default = true)]
    show_inactive: bool,
    #[prop(default = "")]
    name: &'static str,
) -> impl IntoView {
    let selected_category: RwSignal<Option<SphereCategory>> = RwSignal::new(None);
    let show_dropdown = RwSignal::new(false);

    view! {
        <TransitionUnpack resource=category_vec_resource let:sphere_category_vec>
        {
            if sphere_category_vec.is_empty() || (!show_inactive && !sphere_category_vec.iter().any(|sphere_category| sphere_category.is_active)) {
                log::debug!("No category to display.");
                return ().into_any()
            }
            if let Some(init_category_id) = init_category_id &&
                let Some(category) = sphere_category_vec.iter().find(|category| category.category_id == init_category_id) {
                selected_category.set(Some(category.clone()));
            }
            let sphere_category_vec = StoredValue::new(sphere_category_vec.clone());
            view! {
                <input
                    name=name
                    value=move || match &*selected_category.read() {
                        Some(category) => Some(category.category_id),
                        None => None,
                    }
                    class="hidden"
                />
                <div class="flex justify-between">
                    <span class="label text-white">{move_tr!("category")}</span>
                    <DropdownButton
                        button_class="input_primary flex justify-between items-center w-fit gap-2"
                        activated_button_class="input_primary flex justify-between items-center w-fit gap-2"
                        button_content=move || view! {
                            { move || match &*selected_category.read() {
                                Some(category) => Either::Left(view! {
                                    <SphereCategoryBadge category_header=category.clone()/>
                                }),
                                None => Either::Right(view! {
                                    <NoSphereCategory/>
                                })
                            }}
                            <RotatingArrow point_up=show_dropdown/>
                        }
                        align_right=true
                        open_down=false
                        show_dropdown
                    >
                        <ul class="mb-2 p-2 shadow-sm bg-base-200 rounded-sm flex flex-col gap-1">
                            <li>
                                <button
                                    type="button"
                                    class="button-ghost w-full"
                                    on:click=move |_| {
                                        selected_category.set(None);
                                        show_dropdown.set(false);
                                    }
                                >
                                    <NoSphereCategory/>
                                </button>
                            </li>
                            {
                                sphere_category_vec.read_value().iter().map(|sphere_category| {
                                    let category = StoredValue::new(sphere_category.clone());
                                    match show_inactive || sphere_category.is_active {
                                        true => Some(view! {
                                            <li>
                                                <button
                                                    type="button"
                                                    class="button-ghost"
                                                    on:click=move |_| {
                                                        selected_category.set(Some(category.get_value()));
                                                        show_dropdown.set(false);
                                                    }
                                                >
                                                    <SphereCategoryBadge category_header=sphere_category/>
                                                </button>
                                            </li>
                                        }),
                                        false => None,
                                    }
                                }).collect_view()
                            }
                        </ul>
                    </DropdownButton>
                </div>
            }.into_any()
        }
        </TransitionUnpack>
    }
}

/// Dialog to select a sphere category
#[component]
fn NoSphereCategory() -> impl IntoView {
    view! {
        <span class="text-gray-400">{move_tr!("category-none")}</span>
    }
}

/// Collapse with a sphere category name as title, showing its description when opened and with an additional toggle to filter on this category
#[component]
pub fn SphereCategoryCollapseWithFilter(sphere_category: SphereCategory) -> impl IntoView {
    let category_id = sphere_category.category_id;
    let description = sphere_category.description.clone();
    let show_description = RwSignal::new(false);
    let collapse_class = move || match show_description.get() {
        true => "transition-all duration-500 overflow-hidden",
        false => "transition-all duration-500 overflow-hidden h-0",
    };
    let collapse_class_inner = move || match show_description.get() {
        true => "transition-all duration-500 opacity-100 visible",
        false => "transition-all duration-500 opacity-0 invisible",
    };

    view! {
        <div class="flex flex-col gap-1">
            <div class="flex justify-between items-center gap-2">
                <button
                    class="grow p-1 rounded-md flex justify-between items-center hover:bg-base-content/20"
                    on:click=move |_| show_description.update(|value| *value = !*value)
                >
                    <SphereCategoryBadge category_header=sphere_category/>
                    <RotatingArrow point_up=show_description/>
                </button>
                <SphereCategoryToggle category_id/>
            </div>
            <div class=collapse_class>
                <div class=collapse_class_inner>
                    <div class="pl-2 pb-2 text-sm">{description}</div>
                </div>
            </div>
        </div>
    }
}