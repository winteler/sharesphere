use leptos::prelude::*;
use leptos::{component, html, server, view, IntoView};
use leptos_fluent::move_tr;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use serde::{Deserialize, Serialize};
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::{
            get_user,
            ssr::{check_user, reload_user}
        },
        session::ssr::get_db_pool,
    },
    sharesphere_utils::checks::{check_sphere_name, check_string_length},
    sharesphere_utils::constants::MAX_SPHERE_DESCRIPTION_LENGTH,
};
use sharesphere_utils::icons::SphereIcon;
use sharesphere_utils::node_utils::has_reached_scroll_load_threshold;
use sharesphere_utils::routes::get_sphere_path;
use sharesphere_utils::widget::{LoadIndicators, Badge};
use crate::state::GlobalState;

pub const SPHERE_FETCH_LIMIT: usize = 100;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Sphere {
    pub sphere_id: i64,
    pub sphere_name: String,
    pub normalized_sphere_name: String,
    pub description: String,
    pub is_nsfw: bool,
    pub is_banned: bool,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub num_members: i32,
    pub creator_id: i64,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereSubscription {
    pub subscription_id: i64,
    pub user_id: i64,
    pub sphere_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SphereWithUserInfo {
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub sphere: Sphere,
    pub subscription_id: Option<i64>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SphereHeader {
    pub sphere_name: String,
    pub icon_url: Option<String>,
    pub is_nsfw: bool,
}

impl From<&Sphere> for SphereHeader {
    fn from(sphere: &Sphere) -> Self {
        Self::new(sphere.sphere_name.clone(), sphere.icon_url.clone(), sphere.is_nsfw)
    }
}

impl SphereHeader {
    pub fn new(sphere_name: String, icon_url: Option<String>, is_nsfw: bool) -> Self {
        Self {
            sphere_name,
            icon_url,
            is_nsfw,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::role::ssr::init_sphere_leader;
    use sharesphere_auth::user::User;
    use sharesphere_utils::errors::AppError;
    use sharesphere_utils::errors::AppError::InternalServerError;
    use sharesphere_utils::checks::check_sphere_name;
    use crate::sphere::{Sphere, SphereHeader, SphereWithUserInfo};

    pub async fn get_sphere_by_name(sphere_name: &str, db_pool: &PgPool) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as::<_, Sphere>(
            "SELECT * FROM spheres WHERE sphere_name = $1"
        )
            .bind(sphere_name)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn get_sphere_with_user_info(
        sphere_name: &str,
        user_id: Option<i64>,
        db_pool: &PgPool,
    ) -> Result<SphereWithUserInfo, AppError> {
        let sphere = sqlx::query_as::<_, SphereWithUserInfo>(
            "SELECT s.*, sub.subscription_id
            FROM spheres s
            LEFT JOIN sphere_subscriptions sub ON
                sub.sphere_id = s.sphere_id AND
                sub.user_id = $1
            WHERE s.sphere_name = $2",
        )
            .bind(user_id)
            .bind(sphere_name)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn is_sphere_available(sphere_name: &str, db_pool: &PgPool) -> Result<bool, AppError> {
        let sphere_exist = sqlx::query!(
            "SELECT sphere_id FROM spheres WHERE normalized_sphere_name = normalize_sphere_name($1)",
            sphere_name,
        )
            .fetch_one(db_pool)
            .await;

        match sphere_exist {
            Ok(_) => Ok(false),
            Err(sqlx::error::Error::RowNotFound) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_popular_sphere_headers(
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT sphere_name, icon_url, is_nsfw
            FROM spheres
            where NOT is_nsfw
            ORDER BY num_members DESC, sphere_name LIMIT $1",
            limit
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_header_vec)
    }

    pub async fn get_subscribed_sphere_headers(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT s.sphere_name, s.icon_url, s.is_nsfw
            FROM spheres s
            JOIN sphere_subscriptions sub ON
                s.sphere_id = sub.sphere_id AND
                sub.user_id = $1
            ORDER BY sphere_name",
            user_id,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_header_vec)
    }

    pub async fn create_sphere(
        name: &str,
        description: &str,
        is_nsfw: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        user.check_can_publish()?;
        check_sphere_name(name)?;

        let sphere = sqlx::query_as::<_, Sphere>(
            "INSERT INTO spheres (sphere_name, description, is_nsfw, creator_id) VALUES ($1, $2, $3, $4) RETURNING *"
        )
            .bind(name)
            .bind(description)
            .bind(is_nsfw)
            .bind(user.user_id)
            .fetch_one(db_pool)
            .await?;

        init_sphere_leader(user.user_id, &sphere.sphere_name, &db_pool).await?;

        Ok(sphere)
    }

    pub async fn update_sphere_description(
        sphere_name: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        let sphere = sqlx::query_as::<_, Sphere>(
            "UPDATE spheres SET description = $1, timestamp = NOW() WHERE sphere_name = $2 RETURNING *"
        )
            .bind(description)
            .bind(sphere_name)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn subscribe(sphere_id: i64, user_id: i64, db_pool: &PgPool) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO sphere_subscriptions (user_id, sphere_id) VALUES ($1, $2)",
            user_id,
            sphere_id
        )
            .execute(db_pool)
            .await?;

        sqlx::query!(
            "UPDATE spheres SET num_members = num_members + 1 WHERE sphere_id = $1",
            sphere_id
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn unsubscribe(sphere_id: i64, user_id: i64, db_pool: &PgPool) -> Result<(), AppError> {
        let deleted_rows = sqlx::query!(
            "DELETE FROM sphere_subscriptions WHERE user_id = $1 AND sphere_id = $2",
            user_id,
            sphere_id,
        )
            .execute(db_pool)
            .await?
            .rows_affected();

        if deleted_rows != 1 {
            return Err(InternalServerError(format!("Expected one subscription deleted, got {deleted_rows} instead.")))
        }

        sqlx::query!(
            "UPDATE spheres SET num_members = num_members - 1 WHERE sphere_id = $1",
            sphere_id
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn is_sphere_available(sphere_name: String) -> Result<bool, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let sphere_existence = ssr::is_sphere_available(&sphere_name, &db_pool).await?;
    Ok(sphere_existence)
}

#[server]
pub async fn get_sphere_by_name(sphere_name: String) -> Result<Sphere, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let sphere = ssr::get_sphere_by_name(&sphere_name, &db_pool).await?;
    Ok(sphere)
}

#[server]
pub async fn get_subscribed_sphere_headers() -> Result<Vec<SphereHeader>, AppError> {
    let db_pool = get_db_pool()?;
    match get_user().await {
        Ok(Some(user)) => {
            let sphere_header_vec = ssr::get_subscribed_sphere_headers(user.user_id, &db_pool).await?;
            Ok(sphere_header_vec)
        }
        _ => Ok(Vec::new()),
    }
}

#[server]
pub async fn get_popular_sphere_headers() -> Result<Vec<SphereHeader>, AppError> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_popular_sphere_headers(20, &db_pool).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn get_sphere_with_user_info(
    sphere_name: String,
) -> Result<SphereWithUserInfo, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };

    let sphere_content = ssr::get_sphere_with_user_info(sphere_name.as_str(), user_id, &db_pool).await?;

    Ok(sphere_content)
}

#[server]
pub async fn create_sphere(
    sphere_name: String,
    description: String,
    is_nsfw: bool,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&description, "Sphere description", MAX_SPHERE_DESCRIPTION_LENGTH, false)?;
    log::trace!("Create Sphere '{sphere_name}', {description}, {is_nsfw}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let new_sphere_path = get_sphere_path(&sphere_name);

    let sphere = ssr::create_sphere(
        sphere_name.as_str(),
        description.as_str(),
        is_nsfw,
        &user,
        &db_pool,
    ).await?;

    ssr::subscribe(sphere.sphere_id, user.user_id, &db_pool).await?;

    reload_user(user.user_id)?;

    // Redirect to the new sphere
    leptos_axum::redirect(&new_sphere_path);
    Ok(())
}

#[server]
pub async fn update_sphere_description(
    sphere_name: String,
    description: String,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&description, "Sphere description", MAX_SPHERE_DESCRIPTION_LENGTH, false)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::update_sphere_description(&sphere_name, &description, &user, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn subscribe(sphere_id: i64) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::subscribe(sphere_id, user.user_id, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn unsubscribe(sphere_id: i64) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::unsubscribe(sphere_id, user.user_id, &db_pool).await?;
    Ok(())
}

/// Component to display a sphere's header
#[component]
pub fn SphereHeader(
    sphere_header: SphereHeader
) -> impl IntoView {
    let default_icon_index = sphere_header.sphere_name.as_bytes().first().cloned().unwrap_or_default();
    view! {
        <Badge text=sphere_header.sphere_name>
            <SphereIcon
                icon_url=sphere_header.icon_url
                default_icon_index
                class="content-toolbar-icon-size"
            />
        </Badge>
    }
}

/// Component to display a sphere's header that navigates to it upon clicking
#[component]
pub fn SphereHeaderLink(
    sphere_header: SphereHeader
) -> impl IntoView {
    // use navigate and prevent default to handle case where sphere header is in another <a>
    let navigate = use_navigate();
    let sphere_name = StoredValue::new(sphere_header.sphere_name.clone());
    let sphere_path = get_sphere_path(&sphere_header.sphere_name);
    let aria_label = move_tr!("navigate-sphere", {"sphere_name" => sphere_name.get_value()});
    view! {
        <button
            class="button-rounded-neutral px-2 py-1"
            on:click=move |ev| {
                ev.prevent_default();
                navigate(sphere_path.as_str(), NavigateOptions::default());
            }
            aria-label=aria_label
        >
            <SphereHeader sphere_header/>
        </button>
    }
}

/// Component to display a collapsable list of sphere links
#[component]
pub fn SphereLinkItems(
    sphere_header_vec: Vec<SphereHeader>,
    #[prop(default = true)]
    is_dropdown_style: bool,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let item_class = match is_dropdown_style {
        true => "px-2 py-1 my-1 rounded-sm hover:bg-base-content/20",
        false => "px-2 py-1 my-1 rounded-sm hover:bg-base-200"
    };
    view! {
        <For
            each= move || sphere_header_vec.clone().into_iter()
            key=|sphere_header| sphere_header.sphere_name.clone()
            children=move |sphere_header| {
                let sphere_path = get_sphere_path(&sphere_header.sphere_name);
                view! {
                    <li>
                        <a
                            href=sphere_path
                            on:click=move |_| state.show_left_sidebar.set(false)
                        >
                            <div class=item_class>
                                <SphereHeader sphere_header=sphere_header/>
                            </div>
                        </a>
                    </li>
                }
            }
        />
    }
}

/// Component to display a list of sphere links
#[component]
pub fn SphereLinkList(
    sphere_header_vec: Vec<SphereHeader>
) -> impl IntoView {
    if sphere_header_vec.is_empty() {
        return ().into_any()
    }
    view! {
        <ul class="flex flex-col p-1">
            <SphereLinkItems sphere_header_vec/>
        </ul>
    }.into_any()
}

/// Component to display a collapsable list of sphere links
#[component]
pub fn InfiniteSphereLinkList(
    /// signal containing the sphere headers to display
    #[prop(into)]
    sphere_header_vec: Signal<Vec<SphereHeader>>,
    #[prop(into)]
    is_loading: Signal<bool>,
    /// signal containing an eventual loading error in order to display it
    #[prop(into)]
    load_error: Signal<Option<AppError>>,
    /// signal to request loading additional sphere headers
    additional_load_count: RwSignal<i32>,
    /// boolean to style the links for a dropdown
    #[prop(optional)]
    is_dropdown_style: bool,
    /// reference to the container of the sphere headers in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    let (list_class, load_div_class) = match is_dropdown_style {
        true => (
            "flex flex-col overflow-y-auto max-h-124 w-full p-1",
            "w-full min-h-0",
        ),
        false => (
            "flex flex-col overflow-y-auto max-h-full w-full p-1",
            "w-full min-h-9 lg:min-h-17",
        ),
    };
    view! {
        <Show when=move || !sphere_header_vec.read().is_empty()>
            <ul class=list_class
                on:scroll=move |_| if has_reached_scroll_load_threshold(list_ref) && !is_loading.get_untracked() {
                    additional_load_count.update(|value| *value += 1);
                }
                node_ref=list_ref
            >
                <SphereLinkItems sphere_header_vec=sphere_header_vec.get() is_dropdown_style/>
                <li><LoadIndicators load_error is_loading load_div_class/></li>
            </ul>
        </Show>
    }.into_any()
}