use const_format::concatcp;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::{Form, Outlet, A};
use leptos_router::hooks::use_params_map;
use leptos_router::params::ParamsMap;
use leptos_use::{signal_debounced, use_textarea_autosize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::app::{GlobalState, PUBLISH_ROUTE};

use crate::auth::LoginGuardButton;
use crate::constants::PATH_SEPARATOR;
use crate::content::PostSortWidget;
use crate::editor::{FormTextEditor, TextareaData};
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::form::LabeledFormCheckbox;
use crate::forum_category::{get_forum_category_vec, DeleteForumCategory, ForumCategory, ForumCategoryHeader, SetForumCategory};
use crate::forum_management::MANAGE_FORUM_ROUTE;
use crate::icons::{ForumIcon, InternalErrorIcon, LoadingIcon, PlusIcon, SettingsIcon, SubscribedIcon};
use crate::moderation::ModeratePost;
use crate::navigation_bar::{get_create_post_path, get_current_path};
use crate::post::{get_post_vec_by_forum_name, PostBadgeList, PostWithForumInfo};
use crate::post::{
    CREATE_POST_FORUM_QUERY_PARAM, CREATE_POST_ROUTE, POST_ROUTE_PREFIX,
};
use crate::ranking::ScoreIndicator;
use crate::role::{get_forum_role_vec, AuthorizedShow, PermissionLevel, SetUserForumRole, UserForumRole};
use crate::rule::{get_forum_rule_vec, AddRule, RemoveRule, Rule, UpdateRule};
use crate::sidebar::ForumSidebar;
use crate::unpack::{ActionError, ArcSuspenseUnpack, ArcTransitionUnpack, TransitionUnpack};
use crate::widget::{AuthorWidget, CommentCountWidget, TimeSinceWidget};
#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::reload_user,
    auth::{get_user, ssr::check_user},
};

pub const CREATE_FORUM_SUFFIX: &str = "/forum";
pub const CREATE_FORUM_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_FORUM_SUFFIX);
pub const FORUM_ROUTE_PREFIX: &str = "/forums";
pub const FORUM_ROUTE_PARAM_NAME: &str = "forum_name";

pub const FORUM_FETCH_LIMIT: i64 = 20;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Forum {
    pub forum_id: i64,
    pub forum_name: String,
    pub normalized_forum_name: String,
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
pub struct ForumSubscription {
    pub subscription_id: i64,
    pub user_id: i64,
    pub forum_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ForumWithUserInfo {
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub forum: Forum,
    pub subscription_id: Option<i64>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ForumHeader {
    pub forum_name: String,
    pub icon_url: Option<String>,
}

#[derive(Copy, Clone)]
pub struct ForumState {
    pub forum_name: Memo<String>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub permission_level: Signal<PermissionLevel>,
    pub forum_resource: Resource<Result<Forum, ServerFnError<AppError>>>,
    pub forum_categories_resource: Resource<Result<Vec<ForumCategory>, ServerFnError<AppError>>>,
    pub forum_roles_resource: Resource<Result<Vec<UserForumRole>, ServerFnError<AppError>>>,
    pub forum_rules_resource: Resource<Result<Vec<Rule>, ServerFnError<AppError>>>,
    pub moderate_post_action: ServerAction<ModeratePost>,
    pub update_forum_desc_action: ServerAction<UpdateForumDescription>,
    pub set_forum_category_action: ServerAction<SetForumCategory>,
    pub delete_forum_category_action: ServerAction<DeleteForumCategory>,
    pub set_forum_role_action: ServerAction<SetUserForumRole>,
    pub add_rule_action: ServerAction<AddRule>,
    pub update_rule_action: ServerAction<UpdateRule>,
    pub remove_rule_action: ServerAction<RemoveRule>,
}

impl ForumHeader {
    pub fn new(forum_name: String, icon_url: Option<String>) -> Self {
        Self {
            forum_name,
            icon_url,
        }
    }
}

/// # Normalizes a forum's name by making it lowercase and replacing '-' by '_'.
/// # Normalization is used to ensure forum names are sufficiently different.
///
/// ```
/// use app::forum::normalize_forum_name;
///
/// assert_eq!(normalize_forum_name("Test 123-"), "test 123_");
/// ```
pub fn normalize_forum_name(name: &str) -> String {
    name.to_lowercase().replace("-", "_")
}

/// # Returns whether a forum name is valid. Valid characters are ascii alphanumeric, '-' and '_'
///
/// ```
/// use app::forum::is_valid_forum_name;
///
/// assert_eq!(is_valid_forum_name("-Abc123_"), true);
/// assert_eq!(is_valid_forum_name(" name"), false);
/// assert_eq!(is_valid_forum_name("name%"), false);
/// ```
pub fn is_valid_forum_name(name: &str) -> bool {
    name.chars().all(move |c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::errors::AppError;
    use crate::errors::AppError::InternalServerError;
    use crate::forum::{is_valid_forum_name, normalize_forum_name, Forum, ForumHeader, ForumWithUserInfo};
    use crate::role::ssr::set_user_forum_role;
    use crate::role::PermissionLevel;
    use crate::user::User;

    pub async fn get_forum_by_name(forum_name: &str, db_pool: &PgPool) -> Result<Forum, AppError> {
        let forum = sqlx::query_as!(
            Forum,
            "SELECT * FROM forums WHERE forum_name = $1",
            forum_name
        )
        .fetch_one(db_pool)
        .await?;

        Ok(forum)
    }

    pub async fn get_forum_with_user_info(
        forum_name: &str,
        user_id: Option<i64>,
        db_pool: &PgPool,
    ) -> Result<ForumWithUserInfo, AppError> {
        let forum = sqlx::query_as::<_, ForumWithUserInfo>(
            "SELECT f.*, s.subscription_id
            FROM forums f
            LEFT JOIN forum_subscriptions s ON
                s.forum_id = f.forum_id AND
                s.user_id = $1
            WHERE f.forum_name = $2",
        )
            .bind(user_id)
            .bind(forum_name)
            .fetch_one(db_pool)
            .await?;

        Ok(forum)
    }

    pub async fn is_forum_available(forum_name: &str, db_pool: &PgPool) -> Result<bool, AppError> {
        let forum_exist = sqlx::query!(
            "SELECT forum_id FROM forums WHERE normalized_forum_name = $1",
            normalize_forum_name(forum_name),
        )
        .fetch_one(db_pool)
        .await;

        match forum_exist {
            Ok(_) => Ok(false),
            Err(sqlx::error::Error::RowNotFound) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_matching_forum_header_vec(
        forum_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<ForumHeader>, AppError> {
        let forum_header_vec = sqlx::query_as!(
            ForumHeader,
            "SELECT forum_name, icon_url FROM forums WHERE forum_name LIKE $1 ORDER BY forum_name LIMIT $2",
            format!("{forum_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_header_vec)
    }

    pub async fn get_popular_forum_headers(
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<ForumHeader>, AppError> {
        let forum_header_vec = sqlx::query_as!(
            ForumHeader,
            "SELECT forum_name, icon_url FROM forums ORDER BY num_members DESC, forum_name LIMIT $1",
            limit
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_header_vec)
    }

    pub async fn get_subscribed_forum_headers(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<ForumHeader>, AppError> {
        let forum_header_vec = sqlx::query_as!(
            ForumHeader,
            "SELECT f.forum_name, f.icon_url FROM forums f
            JOIN forum_subscriptions s ON
                f.forum_id = s.forum_id AND
                s.user_id = $1
            ORDER BY forum_name",
            user_id,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_header_vec)
    }

    pub async fn create_forum(
        name: &str,
        description: &str,
        is_nsfw: bool,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Forum, AppError> {
        user.check_can_publish()?;
        if name.is_empty() {
            return Err(AppError::new("Cannot create Sphere with empty name."));
        }

        if !is_valid_forum_name(&name)
        {
            return Err(AppError::new(
                "Sphere name can only contain alphanumeric lowercase characters.",
            ));
        }

        let forum = sqlx::query_as!(
            Forum,
            "INSERT INTO forums (forum_name, description, is_nsfw, creator_id) VALUES ($1, $2, $3, $4) RETURNING *",
            name,
            description,
            is_nsfw,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        set_user_forum_role(user.user_id, &forum.forum_name, PermissionLevel::Lead, user, &db_pool).await?;

        Ok(forum)
    }

    pub async fn update_forum_description(
        forum_name: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Forum, AppError> {
        user.check_permissions(forum_name, PermissionLevel::Manage)?;
        let forum = sqlx::query_as!(
            Forum,
            "UPDATE forums SET description = $1, timestamp = CURRENT_TIMESTAMP WHERE forum_name = $2 RETURNING *",
            description,
            forum_name,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(forum)
    }

    pub async fn subscribe(forum_id: i64, user_id: i64, db_pool: &PgPool) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO forum_subscriptions (user_id, forum_id) VALUES ($1, $2)",
            user_id,
            forum_id
        )
            .execute(db_pool)
            .await?;

        sqlx::query!(
            "UPDATE forums SET num_members = num_members + 1 WHERE forum_id = $1",
            forum_id
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn unsubscribe(forum_id: i64, user_id: i64, db_pool: &PgPool) -> Result<(), AppError> {
        let deleted_rows = sqlx::query!(
            "DELETE FROM forum_subscriptions WHERE user_id = $1 AND forum_id = $2",
            user_id,
            forum_id,
        )
            .execute(db_pool)
            .await?
            .rows_affected();

        if deleted_rows != 1 {
            return Err(InternalServerError(format!("Expected one subscription deleted, got {deleted_rows} instead.")))
        }

        sqlx::query!(
            "UPDATE forums SET num_members = num_members - 1 WHERE forum_id = $1",
            forum_id
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn is_forum_available(forum_name: String) -> Result<bool, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let forum_existence = ssr::is_forum_available(&forum_name, &db_pool).await?;
    Ok(forum_existence)
}

#[server]
pub async fn get_forum_by_name(forum_name: String) -> Result<Forum, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let forum = ssr::get_forum_by_name(&forum_name, &db_pool).await?;
    Ok(forum)
}

#[server]
pub async fn get_matching_forum_header_vec(
    forum_prefix: String,
) -> Result<Vec<ForumHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let forum_header_vec = ssr::get_matching_forum_header_vec(
        &forum_prefix, 
        FORUM_FETCH_LIMIT, 
        &db_pool
    ).await?;
    Ok(forum_header_vec)
}

#[server]
pub async fn get_subscribed_forum_headers() -> Result<Vec<ForumHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    match get_user().await {
        Ok(Some(user)) => {
            let forum_name_vec = ssr::get_subscribed_forum_headers(user.user_id, &db_pool).await?;
            Ok(forum_name_vec)
        }
        _ => Ok(Vec::new()),
    }
}

#[server]
pub async fn get_popular_forum_headers() -> Result<Vec<ForumHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let forum_header_vec = ssr::get_popular_forum_headers(FORUM_FETCH_LIMIT, &db_pool).await?;
    Ok(forum_header_vec)
}

#[server]
pub async fn get_forum_with_user_info(
    forum_name: String,
) -> Result<ForumWithUserInfo, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };

    let forum_content = ssr::get_forum_with_user_info(forum_name.as_str(), user_id, &db_pool).await?;

    Ok(forum_content)
}

#[server]
pub async fn create_forum(
    forum_name: String,
    description: String,
    is_nsfw: bool,
) -> Result<(), ServerFnError<AppError>> {
    log::trace!("Create Sphere '{forum_name}', {description}, {is_nsfw}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let new_forum_path: &str = &(FORUM_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + forum_name.as_str());

    let forum = ssr::create_forum(
        forum_name.as_str(),
        description.as_str(),
        is_nsfw,
        &user,
        &db_pool,
    ).await?;
    
    ssr::subscribe(forum.forum_id, user.user_id, &db_pool).await?;

    reload_user(user.user_id)?;

    // Redirect to the new forum
    leptos_axum::redirect(new_forum_path);
    Ok(())
}

#[server]
pub async fn update_forum_description(
    forum_name: String,
    description: String,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::update_forum_description(&forum_name, &description, &user, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn subscribe(forum_id: i64) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::subscribe(forum_id, user.user_id, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn unsubscribe(forum_id: i64) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::unsubscribe(forum_id, user.user_id, &db_pool).await?;
    Ok(())
}

/// Get the current forum name from the path. When the current path does not contain a forum, returns the last valid forum. Used to avoid sending a request when leaving a page
fn get_forum_name_memo(params: Memo<ParamsMap>) -> Memo<String> {
    Memo::new(move |current_forum_name: Option<&String>| {
        if let Some(new_forum_name) = params.read().get_str(FORUM_ROUTE_PARAM_NAME) {
            log::trace!("Current forum name {current_forum_name:?}, new forum name: {new_forum_name}");
            new_forum_name.to_string()
        } else {
            log::trace!("No valid forum name, keep current value: {current_forum_name:?}");
            current_forum_name.cloned().unwrap_or_default()
        }
    })
}

/// Component to display a forum's banner
#[component]
pub fn ForumHeader(
    forum_header: ForumHeader
) -> impl IntoView {
    view! {
        <div class="flex gap-2 items-center">
            <ForumIcon icon_url=forum_header.icon_url class="h-5 w-5"/>
            <span class="pt-1 pb-1.5">{forum_header.forum_name}</span>
        </div>
    }
}

/// Component to display a forum's banner
#[component]
pub fn ForumBanner() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_name = get_forum_name_memo(use_params_map());
    let update_forum_desc_action = ServerAction::<UpdateForumDescription>::new();
    let set_forum_category_action = ServerAction::<SetForumCategory>::new();
    let delete_forum_category_action = ServerAction::<DeleteForumCategory>::new();
    let set_forum_role_action = ServerAction::<SetUserForumRole>::new();
    let add_rule_action = ServerAction::<AddRule>::new();
    let update_rule_action = ServerAction::<UpdateRule>::new();
    let remove_rule_action = ServerAction::<RemoveRule>::new();
    let forum_state = ForumState {
        forum_name,
        category_id_filter: RwSignal::new(None),
        permission_level: Signal::derive(
            move || match &(*state.user.read()) {
                Some(Ok(Some(user))) => user.get_forum_permission_level(&*forum_name.read()),
                _ => PermissionLevel::None,
            }
        ),
        forum_resource: Resource::new(
            move || (
                forum_name.get(),
                update_forum_desc_action.version().get(),
                state.forum_reload_signal.get(),
            ),
            move |(forum_name, _, _)| get_forum_by_name(forum_name)
        ),
        forum_categories_resource: Resource::new(
            move || (
                forum_name.get(), 
                set_forum_category_action.version().get(), 
                delete_forum_category_action.version().get()
            ),
            move |(forum_name, _, _)| get_forum_category_vec(forum_name)
        ),
        forum_roles_resource: Resource::new(
            move || (forum_name.get(), set_forum_role_action.version().get()),
            move |(forum_name, _)| get_forum_role_vec(forum_name),
        ),
        forum_rules_resource: Resource::new(
            move || (
                forum_name.get(),
                add_rule_action.version().get(),
                update_rule_action.version().get(),
                remove_rule_action.version().get()
            ),
            move |(forum_name, _, _, _)| get_forum_rule_vec(forum_name),
        ),
        moderate_post_action: ServerAction::<ModeratePost>::new(),
        update_forum_desc_action,
        set_forum_category_action,
        delete_forum_category_action,
        set_forum_role_action,
        add_rule_action,
        update_rule_action,
        remove_rule_action,
    };
    provide_context(forum_state);

    let forum_path = move || FORUM_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + &forum_name.get();

    view! {
        <div class="flex flex-col gap-2 pt-2 px-2 w-full">
            <ArcTransitionUnpack resource=forum_state.forum_resource let:forum>
            {
                let forum_banner_image = format!("url({})", forum.banner_url.clone().unwrap_or(String::from("/banner.jpg")));
                view! {
                    <a
                        href=forum_path()
                        class="flex-none bg-cover bg-center bg-no-repeat rounded w-full h-40 flex items-center justify-center"
                        style:background-image=forum_banner_image
                        style:background-position="center"
                        style:background-repeat="no-repeat"
                        style:background-size="cover"
                    >
                        <div class="p-3 backdrop-blur bg-black/50 rounded-sm flex justify-center gap-3">
                            <ForumIcon icon_url=forum.icon_url.clone() class="h-12 w-12"/>
                            <span class="text-4xl">{forum_state.forum_name.get()}</span>
                        </div>
                    </a>
                }.into_any()
            }
            </ArcTransitionUnpack>
            <Outlet/>
        </div>
        <div class="max-2xl:hidden">
            <ForumSidebar/>
        </div>
    }.into_any()
}

/// Component to display a forum's contents
#[component]
pub fn ForumContents() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_state = expect_context::<ForumState>();
    let forum_name = expect_context::<ForumState>().forum_name;
    let additional_load_count = RwSignal::new(0);
    let post_vec = RwSignal::new(Vec::<PostWithForumInfo>::new());
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();
    let forum_with_sub_resource = Resource::new(
        move || (forum_name(),),
        move |(forum_name,)| get_forum_with_user_info(forum_name),
    );

    let _initial_post_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            // TODO return map in resource directly?
            let mut forum_category_map = HashMap::<i64, ForumCategoryHeader>::new();
            if let Ok(forum_category_vec) = forum_state.forum_categories_resource.await {
                for forum_category in forum_category_vec {
                    forum_category_map.insert(forum_category.category_id, forum_category.clone().into());
                }
            }

            match get_post_vec_by_forum_name(
                forum_name.get(),
                forum_state.category_id_filter.get(),
                state.post_sort_type.get(),
                0
            ).await {
                Ok(init_post_vec) => {
                    post_vec.set(
                        init_post_vec.into_iter().map(|post| {
                            let category_id = match post.category_id {
                                Some(category_id) => forum_category_map.get(&category_id).cloned(),
                                None => None,
                            };
                            PostWithForumInfo::from_post(post, category_id, None)
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
                let mut forum_category_map = HashMap::<i64, ForumCategoryHeader>::new();
                if let Ok(forum_category_vec) = forum_state.forum_categories_resource.await {
                    for forum_category in forum_category_vec {
                        forum_category_map.insert(forum_category.category_id, forum_category.clone().into());
                    }
                }
                let num_post = post_vec.read_untracked().len();
                match get_post_vec_by_forum_name(
                    forum_name.get_untracked(),
                    forum_state.category_id_filter.get_untracked(),
                    state.post_sort_type.get_untracked(),
                    num_post
                ).await {
                    Ok(add_post_vec) => post_vec.update(|post_vec| {
                        post_vec.extend(
                            add_post_vec.into_iter().map(|post| {
                                let category_id = match post.category_id {
                                    Some(category_id) => forum_category_map.get(&category_id).cloned(),
                                    None => None,
                                };
                                PostWithForumInfo::from_post(post, category_id, None)
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
        <ArcSuspenseUnpack resource=forum_with_sub_resource let:forum>
            <ForumToolbar forum/>
        </ArcSuspenseUnpack>
        <ForumPostMiniatures
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
            show_forum_header=false
        />
    }.into_any()
}

/// Component to display the forum toolbar
#[component]
pub fn ForumToolbar(forum: Arc<ForumWithUserInfo>) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_state = expect_context::<ForumState>();
    let forum_categories_resource = forum_state.forum_categories_resource;
    let forum_id = forum.forum.forum_id;
    let forum_name = RwSignal::new(forum.forum.forum_name.clone());
    let is_subscribed = RwSignal::new(forum.subscription_id.is_some());
    let manage_path = move || FORUM_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + forum_name.get().as_str() + MANAGE_FORUM_ROUTE;

    view! {
        <div class="flex w-full justify-between content-center">
            <div class="flex w-full gap-2">
                <PostSortWidget/>
                <ForumCategoryDropdown forum_categories_resource/>
            </div>
            <div class="flex gap-1">
                <AuthorizedShow forum_name permission_level=PermissionLevel::Moderate>
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
                                        state.subscribe_action.dispatch(Subscribe { forum_id });
                                    } else {
                                        state.unsubscribe_action.dispatch(Unsubscribe { forum_id });
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
                        <Form method="GET" action=CREATE_POST_ROUTE attr:class="flex">
                            <input type="text" name=CREATE_POST_FORUM_QUERY_PARAM class="hidden" value=forum_name/>
                            <button type="submit" class="btn btn-circle btn-ghost">
                                <PlusIcon class="h-6 w-6"/>
                            </button>
                        </Form>
                    </LoginGuardButton>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Dialog to select a forum category
#[component]
pub fn ForumCategoryDropdown(
    forum_categories_resource: Resource<Result<Vec<ForumCategory>, ServerFnError<AppError>>>,
    #[prop(default = true)]
    show_inactive: bool,
    #[prop(default = "")]
    name: &'static str,
) -> impl IntoView {
    let forum_state = use_context::<ForumState>();
    let is_selected = RwSignal::new(false);
    let select_class = move || match is_selected.get() {
        true => "select select-bordered w-fit",
        false => "select select-bordered w-fit text-gray-400",
    };
    
    view! {
        <TransitionUnpack resource=forum_categories_resource let:forum_category_vec>
        {
            if forum_category_vec.is_empty() || (!show_inactive && !forum_category_vec.iter().any(|forum_category| forum_category.is_active)) {
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
                        if let Some(forum_state) = forum_state {
                            match selected.parse::<i64>() {
                                Ok(category_id) => forum_state.category_id_filter.set(Some(category_id)),
                                _ => forum_state.category_id_filter.set(None),
                            };
                        };
                    }
                >
                    <option selected value="" class="text-gray-400">"Category"</option>
                    {
                        forum_category_vec.iter().map(|forum_category| {
                            match show_inactive || forum_category.is_active {
                                true => view! {
                                    <option class="text-white" value=forum_category.category_id>{forum_category.category_name.clone()}</option>
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

/// Component to display a vector of forum posts and indicate when more need to be loaded
#[component]
pub fn ForumPostMiniatures(
    /// signal containing the posts to display
    #[prop(into)]
    post_vec: Signal<Vec<PostWithForumInfo>>,
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
    show_forum_header: bool,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col overflow-y-auto w-full pr-2 divide-y divide-base-content/20"
            on:scroll=move |_| match list_ref.get() {
                Some(node_ref) => {
                    if node_ref.scroll_top() + node_ref.offset_height() >= node_ref.scroll_height() && !is_loading.get_untracked() {
                        additional_load_count.update(|value| *value += 1);
                    }
                },
                None => log::error!("Forum container 'ul' node failed to load."),
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
                    let forum_header = match show_forum_header {
                        true => Some(ForumHeader::new(post.forum_name.clone(), post_info.forum_icon_url)),
                        false => None,
                    };
                    let post_path = FORUM_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + post.forum_name.as_str() + POST_ROUTE_PREFIX + PATH_SEPARATOR + &post.post_id.to_string();
                    view! {
                        <li>
                            <a href=post_path>
                                <div class="flex flex-col gap-1 pt-1 pb-2 my-1 rounded hover:bg-base-content/20">
                                    <h2 class="card-title pl-1">{post.title.clone()}</h2>
                                    <PostBadgeList
                                        forum_header
                                        forum_category=post_info.forum_category
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

/// Component to create new forums
#[component]
pub fn CreateForum() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let forum_name = RwSignal::new(String::new());
    let forum_name_debounced: Signal<String> = signal_debounced(forum_name, 250.0);
    let is_forum_available = Resource::new(
        move || forum_name_debounced.get(),
        move |forum_name| async {
            if forum_name.is_empty() {
                None
            } else {
                Some(is_forum_available(forum_name).await)
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
    let is_name_empty = move || forum_name.read().is_empty();
    let is_name_alphanumeric =
        move || is_valid_forum_name(&forum_name.read());
    let are_inputs_invalid = Memo::new(move |_| {
        is_name_empty()
            || is_name_taken.get()
            || !is_name_alphanumeric()
            || description_data.content.read().is_empty()
    });

    view! {
        <div class="w-4/5 2xl:w-1/3 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=state.create_forum_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Settle a Sphere!"</h2>
                    <div class="h-full flex gap-2">
                        <input
                            type="text"
                            name="forum_name"
                            placeholder="Name"
                            autocomplete="off"
                            class="input input-bordered input-primary h-input_l flex-none w-3/5"
                            autofocus
                            on:input=move |ev| {
                                forum_name.set(event_target_value(&ev));
                            }
                            prop:value=forum_name
                        />
                        <Suspense fallback=move || view! { <LoadingIcon/> }>
                        {
                            move || is_forum_available.map(|result| match result {
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
                                    log::error!("Error while checking forum existence: {e}");
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
            <ActionError action=state.create_forum_action.into()/>
        </div>
    }
}