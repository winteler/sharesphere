use std::ops::{Deref, DerefMut};
use leptos::prelude::*;
use leptos_fluent::move_tr;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::{check_user},
        session::ssr::get_db_pool,
    },
    sharesphere_utils::constants::{MAX_CATEGORY_DESCRIPTION_LENGTH, MAX_CATEGORY_NAME_LENGTH},
    sharesphere_utils::checks::{check_sphere_name, check_string_length},
};
use sharesphere_utils::colors::Color;
use sharesphere_utils::errors::AppError;
use sharesphere_utils::unpack::TransitionUnpack;

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

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow, sqlx::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategoryHeader {
    pub category_name: String,
    pub category_color: Color,
}

#[repr(transparent)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OptionalSphereCategoryHeader(pub Option<SphereCategoryHeader>);

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

impl Deref for OptionalSphereCategoryHeader {
    type Target = Option<SphereCategoryHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OptionalSphereCategoryHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::{FromRow, PgPool, Row};
    use sqlx::postgres::PgRow;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::user::User;
    use sharesphere_utils::colors::Color;
    use sharesphere_utils::errors::AppError;
    use crate::sphere_category::{OptionalSphereCategoryHeader, SphereCategory, SphereCategoryHeader};

    pub const CATEGORY_NOT_DELETED_STR: &str = "Category was not deleted, it either doesn't exist or is used.";

    impl FromRow<'_, PgRow> for OptionalSphereCategoryHeader {
        fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
            match (row.try_get("category_name"), row.try_get("category_color")) {
                (Ok(category_name), Ok(category_color)) => Ok(OptionalSphereCategoryHeader(Some(SphereCategoryHeader {
                    category_name,
                    category_color,
                }))),
                _ => Ok(OptionalSphereCategoryHeader(None)),
            }
        }
    }

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
    let is_selected = RwSignal::new(init_category_id.is_some());
    let select_class = move || match is_selected.get() {
        true => "select w-fit",
        false => "select w-fit text-gray-400",
    };

    view! {
        <TransitionUnpack resource=category_vec_resource let:sphere_category_vec>
        {
            if sphere_category_vec.is_empty() || (!show_inactive && !sphere_category_vec.iter().any(|sphere_category| sphere_category.is_active)) {
                log::debug!("No category to display.");
                return ().into_any()
            }
            view! {
                <select
                    name=name
                    class=select_class
                    on:input=move |ev| {
                        let selected = event_target_value(&ev);
                        is_selected.set(!selected.is_empty());
                    }
                >
                    <option selected=init_category_id.is_none() value="" class="text-gray-400">{move_tr!("category")}</option>
                    {
                        sphere_category_vec.iter().map(|sphere_category| {
                            let is_selected = init_category_id.is_some_and(|category_id| category_id == sphere_category.category_id);
                            match show_inactive || sphere_category.is_active {
                                true => Some(view! {
                                    <option
                                        class="text-white"
                                        selected=is_selected
                                        value=sphere_category.category_id
                                    >
                                        {sphere_category.category_name.clone()}
                                    </option>
                                }),
                                false => None,
                            }
                        }).collect_view()
                    }
                </select>
            }.into_any()
        }
        </TransitionUnpack>
    }
}