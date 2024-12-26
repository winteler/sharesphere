use crate::app::{GlobalState, PUBLISH_ROUTE};
use const_format::concatcp;
use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::{Form, Outlet, A};
use leptos_router::hooks::use_params_map;
use leptos_router::params::ParamsMap;
use leptos_use::{signal_debounced, use_textarea_autosize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::LoginGuardButton;
use crate::constants::PATH_SEPARATOR;
use crate::content::PostSortWidget;
use crate::editor::{FormTextEditor, TextareaData};
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::form::LabeledFormCheckbox;
use crate::icons::{InternalErrorIcon, LoadingIcon, PlusIcon, SettingsIcon, SphereIcon, SubscribedIcon};
use crate::moderation::ModeratePost;
use crate::navigation_bar::{get_create_post_path, get_current_path};
use crate::post::{get_post_vec_by_sphere_name, PostBadgeList, PostWithSphereInfo, CREATE_POST_SUFFIX};
use crate::post::{
    CREATE_POST_ROUTE, CREATE_POST_SPHERE_QUERY_PARAM, POST_ROUTE_PREFIX,
};
use crate::ranking::{ScoreIndicator, SortType};
use crate::role::{get_sphere_role_vec, AuthorizedShow, PermissionLevel, SetUserSphereRole, UserSphereRole};
use crate::rule::{get_sphere_rule_vec, AddRule, RemoveRule, Rule, UpdateRule};
use crate::satellite::{get_satellite_path, get_satellite_vec_by_sphere_name, ActiveSatelliteList, CreateSatellite, DisableSatellite, Satellite, SatelliteState, UpdateSatellite};
use crate::sidebar::SphereSidebar;
use crate::sphere_category::{get_sphere_category_vec, DeleteSphereCategory, SetSphereCategory, SphereCategory, SphereCategoryHeader};
use crate::sphere_management::MANAGE_SPHERE_ROUTE;
use crate::unpack::{ActionError, ArcSuspenseUnpack, ArcTransitionUnpack, TransitionUnpack};
use crate::widget::{AuthorWidget, CommentCountWidget, TimeSinceWidget};
#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::reload_user,
    auth::{get_user, ssr::check_user},
};

pub const CREATE_SPHERE_SUFFIX: &str = "/sphere";
pub const CREATE_SPHERE_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_SPHERE_SUFFIX);
pub const SPHERE_ROUTE_PREFIX: &str = "/spheres";
pub const SPHERE_ROUTE_PARAM_NAME: &str = "sphere_name";

pub const SPHERE_FETCH_LIMIT: i64 = 20;

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

#[derive(Copy, Clone)]
pub struct SphereState {
    pub sphere_name: Memo<String>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub permission_level: Signal<PermissionLevel>,
    pub sphere_resource: Resource<Result<Sphere, ServerFnError<AppError>>>,
    pub satellite_vec_resource: Resource<Result<Vec<Satellite>, ServerFnError<AppError>>>,
    pub sphere_categories_resource: Resource<Result<Vec<SphereCategory>, ServerFnError<AppError>>>,
    pub sphere_roles_resource: Resource<Result<Vec<UserSphereRole>, ServerFnError<AppError>>>,
    pub sphere_rules_resource: Resource<Result<Vec<Rule>, ServerFnError<AppError>>>,
    pub create_satellite_action: ServerAction<CreateSatellite>,
    pub update_satellite_action: ServerAction<UpdateSatellite>,
    pub disable_satellite_action: ServerAction<DisableSatellite>,
    pub moderate_post_action: ServerAction<ModeratePost>,
    pub update_sphere_desc_action: ServerAction<UpdateSphereDescription>,
    pub set_sphere_category_action: ServerAction<SetSphereCategory>,
    pub delete_sphere_category_action: ServerAction<DeleteSphereCategory>,
    pub set_sphere_role_action: ServerAction<SetUserSphereRole>,
    pub add_rule_action: ServerAction<AddRule>,
    pub update_rule_action: ServerAction<UpdateRule>,
    pub remove_rule_action: ServerAction<RemoveRule>,
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

/// # Normalizes a sphere's name by making it lowercase and replacing '-' by '_'.
/// # Normalization is used to ensure sphere names are sufficiently different.
///
/// ```
/// use app::sphere::normalize_sphere_name;
///
/// assert_eq!(normalize_sphere_name("Test 123-"), "test 123_");
/// ```
pub fn normalize_sphere_name(name: &str) -> String {
    name.to_lowercase().replace("-", "_")
}

/// # Returns whether a sphere name is valid. Valid characters are ascii alphanumeric, '-' and '_'
///
/// ```
/// use app::sphere::is_valid_sphere_name;
///
/// assert_eq!(is_valid_sphere_name("-Abc123_"), true);
/// assert_eq!(is_valid_sphere_name(" name"), false);
/// assert_eq!(is_valid_sphere_name("name%"), false);
/// ```
pub fn is_valid_sphere_name(name: &str) -> bool {
    name.chars().all(move |c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::errors::AppError;
    use crate::errors::AppError::InternalServerError;
    use crate::role::ssr::set_user_sphere_role;
    use crate::role::PermissionLevel;
    use crate::sphere::{is_valid_sphere_name, normalize_sphere_name, Sphere, SphereHeader, SphereWithUserInfo};
    use crate::user::User;

    pub async fn get_sphere_by_name(sphere_name: &str, db_pool: &PgPool) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as!(
            Sphere,
            "SELECT * FROM spheres WHERE sphere_name = $1",
            sphere_name
        )
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
            "SELECT sphere_id FROM spheres WHERE normalized_sphere_name = $1",
            normalize_sphere_name(sphere_name),
        )
        .fetch_one(db_pool)
        .await;

        match sphere_exist {
            Ok(_) => Ok(false),
            Err(sqlx::error::Error::RowNotFound) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_matching_sphere_header_vec(
        sphere_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT sphere_name, icon_url, is_nsfw
            FROM spheres
            WHERE sphere_name LIKE $1
            ORDER BY sphere_name LIMIT $2",
            format!("{sphere_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_header_vec)
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

        let sphere = sqlx::query_as!(
            Sphere,
            "INSERT INTO spheres (sphere_name, description, is_nsfw, creator_id) VALUES ($1, $2, $3, $4) RETURNING *",
            name,
            description,
            is_nsfw,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        set_user_sphere_role(user.user_id, &sphere.sphere_name, PermissionLevel::Lead, user, &db_pool).await?;

        Ok(sphere)
    }

    pub async fn update_sphere_description(
        sphere_name: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        let sphere = sqlx::query_as!(
            Sphere,
            "UPDATE spheres SET description = $1, timestamp = CURRENT_TIMESTAMP WHERE sphere_name = $2 RETURNING *",
            description,
            sphere_name,
        )
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
pub async fn is_sphere_available(sphere_name: String) -> Result<bool, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_existence = ssr::is_sphere_available(&sphere_name, &db_pool).await?;
    Ok(sphere_existence)
}

#[server]
pub async fn get_sphere_by_name(sphere_name: String) -> Result<Sphere, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere = ssr::get_sphere_by_name(&sphere_name, &db_pool).await?;
    Ok(sphere)
}

#[server]
pub async fn get_matching_sphere_header_vec(
    sphere_prefix: String,
) -> Result<Vec<SphereHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_matching_sphere_header_vec(
        &sphere_prefix,
        SPHERE_FETCH_LIMIT,
        &db_pool
    ).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn get_subscribed_sphere_headers() -> Result<Vec<SphereHeader>, ServerFnError<AppError>> {
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
pub async fn get_popular_sphere_headers() -> Result<Vec<SphereHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_popular_sphere_headers(SPHERE_FETCH_LIMIT, &db_pool).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn get_sphere_with_user_info(
    sphere_name: String,
) -> Result<SphereWithUserInfo, ServerFnError<AppError>> {
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
) -> Result<(), ServerFnError<AppError>> {
    log::trace!("Create Sphere '{sphere_name}', {description}, {is_nsfw}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let new_sphere_path: &str = &(SPHERE_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + sphere_name.as_str());

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
    leptos_axum::redirect(new_sphere_path);
    Ok(())
}

#[server]
pub async fn update_sphere_description(
    sphere_name: String,
    description: String,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::update_sphere_description(&sphere_name, &description, &user, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn subscribe(sphere_id: i64) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::subscribe(sphere_id, user.user_id, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn unsubscribe(sphere_id: i64) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::unsubscribe(sphere_id, user.user_id, &db_pool).await?;
    Ok(())
}

/// Get the current sphere name from the path. When the current path does not contain a sphere, returns the last valid sphere. Used to avoid sending a request when leaving a page
fn get_sphere_name_memo(params: Memo<ParamsMap>) -> Memo<String> {
    Memo::new(move |current_sphere_name: Option<&String>| {
        if let Some(new_sphere_name) = params.read().get_str(SPHERE_ROUTE_PARAM_NAME) {
            log::trace!("Current sphere name {current_sphere_name:?}, new sphere name: {new_sphere_name}");
            new_sphere_name.to_string()
        } else {
            log::trace!("No valid sphere name, keep current value: {current_sphere_name:?}");
            current_sphere_name.cloned().unwrap_or_default()
        }
    })
}

/// Component to display a sphere's banner
#[component]
pub fn SphereHeader(
    sphere_header: SphereHeader
) -> impl IntoView {
    view! {
        <div class="flex gap-2 items-center">
            <SphereIcon icon_url=sphere_header.icon_url class="h-5 w-5"/>
            <span class="pt-1 pb-1.5">{sphere_header.sphere_name}</span>
        </div>
    }
}

/// Component to display a sphere's banner
#[component]
pub fn SphereBanner() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_name = get_sphere_name_memo(use_params_map());
    let create_satellite_action = ServerAction::<CreateSatellite>::new();
    let update_satellite_action = ServerAction::<UpdateSatellite>::new();
    let disable_satellite_action = ServerAction::<DisableSatellite>::new();
    let update_sphere_desc_action = ServerAction::<UpdateSphereDescription>::new();
    let set_sphere_category_action = ServerAction::<SetSphereCategory>::new();
    let delete_sphere_category_action = ServerAction::<DeleteSphereCategory>::new();
    let set_sphere_role_action = ServerAction::<SetUserSphereRole>::new();
    let add_rule_action = ServerAction::<AddRule>::new();
    let update_rule_action = ServerAction::<UpdateRule>::new();
    let remove_rule_action = ServerAction::<RemoveRule>::new();
    let sphere_state = SphereState {
        sphere_name,
        category_id_filter: RwSignal::new(None),
        permission_level: Signal::derive(
            move || match &(*state.user.read()) {
                Some(Ok(Some(user))) => user.get_sphere_permission_level(&*sphere_name.read()),
                _ => PermissionLevel::None,
            }
        ),
        sphere_resource: Resource::new(
            move || (
                sphere_name.get(),
                update_sphere_desc_action.version().get(),
                state.sphere_reload_signal.get(),
            ),
            move |(sphere_name, _, _)| get_sphere_by_name(sphere_name)
        ),
        satellite_vec_resource: Resource::new(
            move || (
                sphere_name.get(),
                create_satellite_action.version().get(),
                update_satellite_action.version().get(),
                disable_satellite_action.version().get(),
            ),
            move |(sphere_name, _, _, _)| get_satellite_vec_by_sphere_name(sphere_name, true)
        ),
        sphere_categories_resource: Resource::new(
            move || (
                sphere_name.get(),
                set_sphere_category_action.version().get(),
                delete_sphere_category_action.version().get()
            ),
            move |(sphere_name, _, _)| get_sphere_category_vec(sphere_name)
        ),
        sphere_roles_resource: Resource::new(
            move || (sphere_name.get(), set_sphere_role_action.version().get()),
            move |(sphere_name, _)| get_sphere_role_vec(sphere_name),
        ),
        sphere_rules_resource: Resource::new(
            move || (
                sphere_name.get(),
                add_rule_action.version().get(),
                update_rule_action.version().get(),
                remove_rule_action.version().get()
            ),
            move |(sphere_name, _, _, _)| get_sphere_rule_vec(sphere_name),
        ),
        create_satellite_action,
        update_satellite_action,
        disable_satellite_action,
        moderate_post_action: ServerAction::<ModeratePost>::new(),
        update_sphere_desc_action,
        set_sphere_category_action,
        delete_sphere_category_action,
        set_sphere_role_action,
        add_rule_action,
        update_rule_action,
        remove_rule_action,
    };
    provide_context(sphere_state);

    let sphere_path = move || SPHERE_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + &sphere_name.get();

    view! {
        <div class="flex flex-col gap-2 pt-2 px-2 w-full">
            <ArcTransitionUnpack resource=sphere_state.sphere_resource let:sphere>
            {
                let sphere_banner_image = format!("url({})", sphere.banner_url.clone().unwrap_or(String::from("/banner.jpg")));
                view! {
                    <a
                        href=sphere_path()
                        class="flex-none bg-cover bg-center bg-no-repeat rounded w-full h-40 flex items-center justify-center"
                        style:background-image=sphere_banner_image
                        style:background-position="center"
                        style:background-repeat="no-repeat"
                        style:background-size="cover"
                    >
                        <div class="p-3 backdrop-blur bg-black/50 rounded-sm flex justify-center gap-3">
                            <SphereIcon icon_url=sphere.icon_url.clone() class="h-12 w-12"/>
                            <span class="text-4xl">{sphere_state.sphere_name.get()}</span>
                        </div>
                    </a>
                }.into_any()
            }
            </ArcTransitionUnpack>
            <Outlet/>
        </div>
        <div class="max-2xl:hidden">
            <SphereSidebar/>
        </div>
    }.into_any()
}

/// Component to display a sphere's contents
#[component]
pub fn SphereContents() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = expect_context::<SphereState>().sphere_name;
    let additional_load_count = RwSignal::new(0);
    let post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();
    let sphere_with_sub_resource = Resource::new(
        move || (sphere_name(),),
        move |(sphere_name,)| get_sphere_with_user_info(sphere_name),
    );

    let _initial_post_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            // TODO return map in resource directly?
            let mut sphere_category_map = HashMap::<i64, SphereCategoryHeader>::new();
            if let Ok(sphere_category_vec) = sphere_state.sphere_categories_resource.await {
                for sphere_category in sphere_category_vec {
                    sphere_category_map.insert(sphere_category.category_id, sphere_category.clone().into());
                }
            }
            log::info!("Load posts by sphere");

            match get_post_vec_by_sphere_name(
                sphere_name.get(),
                sphere_state.category_id_filter.get(),
                state.post_sort_type.get(),
                0
            ).await {
                Ok(init_post_vec) => {
                    post_vec.set(
                        init_post_vec.into_iter().map(|post| {
                            let category_id = match post.category_id {
                                Some(category_id) => sphere_category_map.get(&category_id).cloned(),
                                None => None,
                            };
                            PostWithSphereInfo::from_post(post, category_id, None)
                        }).collect(),
                    );
                    if let Some(list_ref) = list_ref.get_untracked() {
                        list_ref.set_scroll_top(0);
                    }
                },
                Err(ref e) => {
                    post_vec.update(|post_vec| post_vec.clear());
                    load_error.set(Some(AppError::from(e)))
                },
            };
            is_loading.set(false);
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let mut sphere_category_map = HashMap::<i64, SphereCategoryHeader>::new();
                if let Ok(sphere_category_vec) = sphere_state.sphere_categories_resource.await {
                    for sphere_category in sphere_category_vec {
                        sphere_category_map.insert(sphere_category.category_id, sphere_category.clone().into());
                    }
                }
                let num_post = post_vec.read_untracked().len();
                match get_post_vec_by_sphere_name(
                    sphere_name.get_untracked(),
                    sphere_state.category_id_filter.get_untracked(),
                    state.post_sort_type.get_untracked(),
                    num_post
                ).await {
                    Ok(add_post_vec) => post_vec.update(|post_vec| {
                        post_vec.extend(
                            add_post_vec.into_iter().map(|post| {
                                let category_id = match post.category_id {
                                    Some(category_id) => sphere_category_map.get(&category_id).cloned(),
                                    None => None,
                                };
                                PostWithSphereInfo::from_post(post, category_id, None)
                            })
                        )
                    }),
                    Err(e) => load_error.set(Some(AppError::from(e))),
                }
                is_loading.set(false);
            }
        }
    );

    view! {
        <ActiveSatelliteList/>
        <ArcSuspenseUnpack resource=sphere_with_sub_resource let:sphere>
            <SphereToolbar
                sphere
                sort_signal=state.post_sort_type
                category_id_signal=sphere_state.category_id_filter
            />
        </ArcSuspenseUnpack>
        <SpherePostMiniatures
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
            show_sphere_header=false
        />
    }.into_any()
}

/// Component to display the sphere toolbar
#[component]
pub fn SphereToolbar(
    sphere: Arc<SphereWithUserInfo>,
    sort_signal: RwSignal<SortType>,
    category_id_signal: RwSignal<Option<i64>>
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let satellite_state = use_context::<SatelliteState>();
    let category_vec_resource = sphere_state.sphere_categories_resource;
    let sphere_id = sphere.sphere.sphere_id;
    let sphere_name = RwSignal::new(sphere.sphere.sphere_name.clone());
    let is_subscribed = RwSignal::new(sphere.subscription_id.is_some());
    let manage_path = move || SPHERE_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + sphere_name.get().as_str() + MANAGE_SPHERE_ROUTE;

    view! {
        <div class="flex w-full justify-between content-center">
            <div class="flex w-full gap-2">
                <PostSortWidget sort_signal/>
                <SphereCategoryDropdown category_vec_resource category_id_signal=Some(category_id_signal)/>
            </div>
            <div class="flex gap-1">
                <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
                    <A href=manage_path.clone() attr:class="btn btn-circle btn-ghost">
                        <SettingsIcon class="h-5 w-5"/>
                    </A>
                </AuthorizedShow>
                <div class="tooltip" data-tip="Join">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <SubscribedIcon class="h-6 w-6" show_color=is_subscribed/> }.into_any()
                        redirect_path_fn=&get_current_path
                        let:_user
                    >
                        <button type="submit" class="btn btn-circle btn-ghost" on:click=move |_| {
                                is_subscribed.update(|value| {
                                    *value = !*value;
                                    if *value {
                                        state.subscribe_action.dispatch(Subscribe { sphere_id });
                                    } else {
                                        state.unsubscribe_action.dispatch(Unsubscribe { sphere_id });
                                    }
                                })
                            }
                        >
                            <SubscribedIcon class="h-6 w-6" show_color=is_subscribed/>
                        </button>
                    </LoginGuardButton>
                </div>
                <div class="tooltip" data-tip="New">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <PlusIcon class="h-6 w-6"/> }.into_any()
                        redirect_path_fn=&get_create_post_path
                        let:_user
                    >
                    { move || match satellite_state {
                        Some(satellite_state) => {
                            let create_post_link = get_satellite_path(
                                &*sphere_state.sphere_name.read(),
                                satellite_state.satellite_id.get()
                            ) + PUBLISH_ROUTE + CREATE_POST_SUFFIX;
                            Either::Left(view! {
                                <a href=create_post_link class="btn btn-circle btn-ghost">
                                    <PlusIcon class="h-6 w-6"/>
                                </a>
                            })
                        }
                        None => Either::Right(view! {
                            <Form method="GET" action=CREATE_POST_ROUTE attr:class="flex">
                                <input type="text" name=CREATE_POST_SPHERE_QUERY_PARAM class="hidden" value=sphere_name/>
                                <button type="submit" class="btn btn-circle btn-ghost">
                                    <PlusIcon class="h-6 w-6"/>
                                </button>
                            </Form>
                        }),
                    }}
                    </LoginGuardButton>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Dialog to select a sphere category
#[component]
pub fn SphereCategoryDropdown(
    category_vec_resource: Resource<Result<Vec<SphereCategory>, ServerFnError<AppError>>>,
    #[prop(default = None)]
    category_id_signal: Option<RwSignal<Option<i64>>>,
    #[prop(default = true)]
    show_inactive: bool,
    #[prop(default = "")]
    name: &'static str,
) -> impl IntoView {
    let is_selected = RwSignal::new(false);
    let select_class = move || match is_selected.get() {
        true => "select select-bordered w-fit",
        false => "select select-bordered w-fit text-gray-400",
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
                    on:click=move |ev| {
                        let selected = event_target_value(&ev);
                        is_selected.set(!selected.is_empty());
                        if let Some(category_id_signal) = category_id_signal {
                            match selected.parse::<i64>() {
                                Ok(category_id) => category_id_signal.set(Some(category_id)),
                                _ => category_id_signal.set(None),
                            };
                        };
                    }
                >
                    <option selected value="" class="text-gray-400">"Category"</option>
                    {
                        sphere_category_vec.iter().map(|sphere_category| {
                            match show_inactive || sphere_category.is_active {
                                true => view! {
                                    <option class="text-white" value=sphere_category.category_id>{sphere_category.category_name.clone()}</option>
                                }.into_any(),
                                false => ().into_any(),
                            }
                        }).collect_view()
                    }
                </select>
            }.into_any()
        }
        </TransitionUnpack>
    }
}

/// Component to display a vector of sphere posts and indicate when more need to be loaded
#[component]
pub fn SpherePostMiniatures(
    /// signal containing the posts to display
    #[prop(into)]
    post_vec: Signal<Vec<PostWithSphereInfo>>,
    /// signal indicating new posts are being loaded
    #[prop(into)]
    is_loading: Signal<bool>,
    /// signal containing an eventual loading error in order to display it
    #[prop(into)]
    load_error: Signal<Option<AppError>>,
    /// signal to request loading additional posts
    additional_load_count: RwSignal<i64>,
    /// reference to the container of the posts in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
    #[prop(default = true)]
    show_sphere_header: bool,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col overflow-y-auto w-full pr-2 divide-y divide-base-content/20"
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
            <For
                // a function that returns the items we're iterating over; a signal is fine
                each= move || post_vec.get().into_iter().enumerate()
                // a unique key for each item as a reference
                key=|(_index, post)| post.post.post_id
                // renders each item to a view
                children=move |(_key, post_info)| {
                    let post = post_info.post;
                    let sphere_header = match show_sphere_header {
                        true => Some(SphereHeader::new(post.sphere_name.clone(), post_info.sphere_icon_url, false)),
                        false => None,
                    };
                    let post_path = SPHERE_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + post.sphere_name.as_str() + POST_ROUTE_PREFIX + PATH_SEPARATOR + &post.post_id.to_string();
                    view! {
                        <li>
                            <a href=post_path>
                                <div class="flex flex-col gap-1 pl-1 pt-1 pb-2 my-1 rounded hover:bg-base-content/20">
                                    <h2 class="card-title pl-1">{post.title.clone()}</h2>
                                    <PostBadgeList
                                        sphere_header
                                        sphere_category=post_info.sphere_category
                                        is_spoiler=post.is_spoiler
                                        is_nsfw=post.is_nsfw
                                    />
                                    <div class="flex gap-1">
                                        <ScoreIndicator score=post.score/>
                                        <CommentCountWidget count=post.num_comments/>
                                        <AuthorWidget author=post.creator_name.clone() is_moderator=post.is_creator_moderator/>
                                        <TimeSinceWidget timestamp=post.create_timestamp/>
                                    </div>
                                </div>
                            </a>
                        </li>
                    }
                }
            />
            <Show when=move || load_error.read().is_some()>
            {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(load_error.get().unwrap());
                view! {
                    <li><div class="flex justify-start py-4"><ErrorTemplate outside_errors/></div></li>
                }
            }
            </Show>
            <Show when=is_loading>
                <li><LoadingIcon/></li>
            </Show>
        </ul>
    }
}

/// Component to create new spheres
#[component]
pub fn CreateSphere() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let sphere_name = RwSignal::new(String::new());
    let sphere_name_debounced: Signal<String> = signal_debounced(sphere_name, 250.0);
    let is_sphere_available = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_name| async {
            if sphere_name.is_empty() {
                None
            } else {
                Some(is_sphere_available(sphere_name).await)
            }
        },
    );

    let is_name_taken = RwSignal::new(false);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let textarea_autosize = use_textarea_autosize(textarea_ref);
    let description_data = TextareaData {
        content: textarea_autosize.content,
        set_content: textarea_autosize.set_content,
        textarea_ref,
    };
    let is_name_empty = move || sphere_name.read().is_empty();
    let is_name_alphanumeric =
        move || is_valid_sphere_name(&sphere_name.read());
    let are_inputs_invalid = Memo::new(move |_| {
        is_name_empty()
            || is_name_taken.get()
            || !is_name_alphanumeric()
            || description_data.content.read().is_empty()
    });

    view! {
        <div class="w-4/5 2xl:w-1/3 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=state.create_sphere_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Settle a Sphere!"</h2>
                    <div class="h-full flex gap-2">
                        <input
                            type="text"
                            name="sphere_name"
                            placeholder="Name"
                            autocomplete="off"
                            class="input input-bordered input-primary h-input_l flex-none w-3/5"
                            autofocus
                            on:input=move |ev| {
                                sphere_name.set(event_target_value(&ev));
                            }
                            prop:value=sphere_name
                        />
                        <Suspense fallback=move || view! { <LoadingIcon/> }>
                        {
                            move || is_sphere_available.map(|result| match result {
                                None | Some(Ok(true)) => {
                                    is_name_taken.set(false);
                                    view! {}.into_any()
                                },
                                Some(Ok(false)) => {
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-input_l flex items-center justify-center">
                                            <span class="font-semibold">"Unavailable"</span>
                                        </div>
                                    }.into_any()
                                },
                                Some(Err(e)) => {
                                    log::error!("Error while checking sphere existence: {e}");
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-input_l flex items-center justify-center">
                                            <InternalErrorIcon class="h-16 w-16"/>
                                            <span class="font-semibold">"Server error"</span>
                                        </div>
                                    }.into_any()
                                },
                            })

                        }
                        </Suspense>
                        <div class="alert alert-error h-input_l flex content-center" class:hidden=move || is_name_empty() || is_name_alphanumeric()>
                            <InternalErrorIcon class="h-16 w-16"/>
                            <span>"Only alphanumeric characters."</span>
                        </div>
                    </div>
                    <FormTextEditor
                        name="description"
                        placeholder="Description"
                        data=description_data
                    />
                    <LabeledFormCheckbox name="is_nsfw" label="NSFW content"/>
                    <Suspense fallback=move || view! { <LoadingIcon/> }>
                        <button type="submit" class="btn btn-active btn-secondary" disabled=are_inputs_invalid>"Create"</button>
                    </Suspense>
                </div>
            </ActionForm>
            <ActionError action=state.create_sphere_action.into()/>
        </div>
    }
}