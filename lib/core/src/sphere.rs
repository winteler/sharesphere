use leptos::prelude::*;
use leptos::{component, html, server, view, IntoView};
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
};
use sharesphere_utils::icons::SphereIcon;
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
    use crate::sphere::{is_valid_sphere_name, Sphere, SphereHeader, SphereWithUserInfo};

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
        if name.is_empty() {
            return Err(AppError::new("Cannot create Sphere with empty name."));
        }

        if !is_valid_sphere_name(&name)
        {
            return Err(AppError::new(
                "Sphere name can only contain alphanumeric lowercase characters.",
            ));
        }

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
            "UPDATE spheres SET description = $1, timestamp = CURRENT_TIMESTAMP WHERE sphere_name = $2 RETURNING *"
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
    let db_pool = get_db_pool()?;
    let sphere_existence = ssr::is_sphere_available(&sphere_name, &db_pool).await?;
    Ok(sphere_existence)
}

#[server]
pub async fn get_sphere_by_name(sphere_name: String) -> Result<Sphere, AppError> {
    let db_pool = get_db_pool()?;
    let sphere = ssr::get_sphere_by_name(&sphere_name, &db_pool).await?;
    Ok(sphere)
}

#[server]
pub async fn get_subscribed_sphere_headers() -> Result<Vec<SphereHeader>, AppError> {
    let db_pool = get_db_pool()?;
    match get_user().await {
        Ok(Some(user)) => {
            let sphere_name_vec = ssr::get_subscribed_sphere_headers(user.user_id, &db_pool).await?;
            Ok(sphere_name_vec)
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
    view! {
        <Badge text=sphere_header.sphere_name>
            <SphereIcon icon_url=sphere_header.icon_url class="content-toolbar-icon-size"/>
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
    let sphere_path = get_sphere_path(&sphere_header.sphere_name);
    let aria_label = format!("Navigate to sphere {} with path {}", sphere_header.sphere_name, sphere_path);
    view! {
        <button
            class="button-rounded-neutral p-0 px-2"
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
    sphere_header_vec: Vec<SphereHeader>
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <For
            each= move || sphere_header_vec.clone().into_iter()
            key=|sphere_header| sphere_header.sphere_name.clone()
            children=move |sphere_header| {
                let sphere_path = get_sphere_path(&sphere_header.sphere_name);
                view! {
                    <li class="px-2 rounded-sm hover:bg-base-content/20">
                        <a
                            href=sphere_path
                            on:click=move |_| state.show_left_sidebar.set(false)
                        >
                            <SphereHeader sphere_header=sphere_header/>
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
    additional_load_count: RwSignal<i64>,
    /// reference to the container of the sphere headers in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    view! {
        <Show when=move || !sphere_header_vec.read().is_empty()>
            <ul class="flex flex-col overflow-y-auto max-h-full w-full p-1"
                on:scroll=move |_| match list_ref.get() {
                    Some(node_ref) => {
                        if node_ref.scroll_top() + node_ref.offset_height() >= node_ref.scroll_height() && !is_loading.get_untracked() {
                            additional_load_count.update(|value| *value += 1);
                        }
                    },
                    None => log::error!("Sphere container 'ul' node failed to load."),
                }
                node_ref=list_ref
            >
                <SphereLinkItems sphere_header_vec=sphere_header_vec.get()/>
                <LoadIndicators load_error is_loading/>
            </ul>
        </Show>
    }.into_any()
}

/// # Returns whether a sphere name is valid. Valid characters are ascii alphanumeric, '-' and '_'
///
/// ```
/// use sharesphere_core::sphere::is_valid_sphere_name;
///
/// assert_eq!(is_valid_sphere_name("-Abc123_"), true);
/// assert_eq!(is_valid_sphere_name(" name"), false);
/// assert_eq!(is_valid_sphere_name("name%"), false);
/// ```
pub fn is_valid_sphere_name(name: &str) -> bool {
    name.chars().all(move |c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}