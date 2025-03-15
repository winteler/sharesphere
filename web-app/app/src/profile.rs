use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map};
use leptos_router::params::ParamsMap;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::comment::{CommentMiniatureList, CommentSortType, CommentWithContext};
use crate::content::{CommentSortWidget, PostSortWidget};
use crate::errors::AppError;
use crate::post::{PostMiniatureList, PostSortType, PostWithSphereInfo};
use crate::ranking::SortType;
use crate::sidebar::HomeSidebar;
use crate::unpack::{handle_additional_load, handle_initial_load, ActionError};
use crate::widget::{EnumQueryTabs, ToView};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    comment::COMMENT_BATCH_SIZE,
    post::{POST_BATCH_SIZE},
};
use crate::app::GlobalState;
use crate::auth::NavigateToUserAccount;
use crate::form::LabeledFormCheckbox;
use crate::icons::{LoadingIcon, UserIcon, UserSettingsIcon};

pub const USER_ROUTE_PREFIX: &str = "/users";
pub const USER_ROUTE_PARAM_NAME: &str = "username";
pub const PROFILE_TAB_QUERY_PARAM: &str = "tab";

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ProfileTabs {
    #[default]
    Posts,
    Comments,
}

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SelfProfileTabs {
    #[default]
    Posts,
    Comments,
    Settings,
}

impl ToView for ProfileTabs {
    fn to_view(self) -> AnyView {
        match self {
            ProfileTabs::Posts => view! { <UserPosts/> }.into_any(),
            ProfileTabs::Comments => view! { <UserComments/> }.into_any(),
        }
    }
}

impl ToView for SelfProfileTabs {
    fn to_view(self) -> AnyView {
        match self {
            SelfProfileTabs::Posts => view! { <UserPosts/> }.into_any(),
            SelfProfileTabs::Comments => view! { <UserComments/> }.into_any(),
            SelfProfileTabs::Settings => view! { <UserSettings/> }.into_any(),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use crate::comment::{CommentWithContext};
    use crate::errors::AppError;
    use crate::post::PostWithSphereInfo;
    use crate::post::ssr::PostJoinCategory;
    use crate::ranking::SortType;

    pub async fn get_user_post_vec(
        username: &str,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> leptos::error::Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            format!(
                "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url
                FROM posts p
                JOIN spheres s on s.sphere_id = p.sphere_id
                LEFT JOIN sphere_categories c on c.category_id = p.category_id
                WHERE
                    p.creator_name = $1 AND
                    p.moderator_id IS NULL AND
                    p.delete_timestamp IS NULL
                ORDER BY {} DESC
                LIMIT $2
                OFFSET $3",
                sort_type.to_order_by_code(),
            ).as_str()
        )
            .bind(username)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        let post_vec = post_vec.into_iter().map(PostJoinCategory::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn get_user_comment_vec(
        username: &str,
        sort_type: SortType,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> leptos::error::Result<Vec<CommentWithContext>, AppError> {
        let comment_vec = sqlx::query_as::<_, CommentWithContext>(
            format!(
                "SELECT c.*, s.sphere_name, s.icon_url, s.is_nsfw, p.satellite_id, p.title as post_title FROM comments c
                JOIN posts p ON p.post_id = c.post_id
                JOIN spheres s ON s.sphere_id = p.sphere_id
                WHERE
                    c.creator_name = $1 AND
                    c.moderator_id IS NULL AND
                    c.delete_timestamp IS NULL
                ORDER BY {} DESC
                LIMIT $2
                OFFSET $3",
                sort_type.to_order_by_code(),
            ).as_str()
        )
            .bind(username)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(comment_vec)
    }
}

#[server]
pub async fn get_user_post_vec(
    username: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    
    // TODO check if private profile

    let post_vec = ssr::get_user_post_vec(
        &username,
        sort_type,
        POST_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(post_vec)
}

#[server]
pub async fn get_user_comment_vec(
    username: String,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithContext>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;

    // TODO check if private profile

    let comment_vec = ssr::get_user_comment_vec(
        &username,
        sort_type,
        COMMENT_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    ).await?;

    Ok(comment_vec)
}


/// Displays a user's profile
#[component]
pub fn UserProfile() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let params = use_params_map();
    let query_username = get_username_memo(params);
    view! {
        <div class="w-full flex justify-center">
            <div class="w-full 2xl:w-2/3 flex flex-col max-2xl:items-center">
                <div class="p-2 pt-4 flex items-center gap-1 text-2xl font-bold">
                    <UserIcon/>
                    {move || query_username.get()}
                </div>
                <Transition fallback=move || view! {  <LoadingIcon/> }>
                { 
                    move || Suspend::new(async move { 
                        match state.user.await {
                            Ok(Some(user)) if user.username == query_username.get() => view! { 
                                <EnumQueryTabs 
                                    query_param=PROFILE_TAB_QUERY_PARAM 
                                    query_enum_iter=SelfProfileTabs::iter()
                                /> 
                            }.into_any(),
                            _ => view! { 
                                <EnumQueryTabs 
                                    query_param=PROFILE_TAB_QUERY_PARAM 
                                    query_enum_iter=ProfileTabs::iter()
                                /> 
                            }.into_any(),
                        }
                    })
                }
                </Transition>
            </div>
        </div>
        <div class="max-2xl:hidden">
            <HomeSidebar/>
        </div>
    }
}

/// Displays a user's posts
#[component]
pub fn UserPosts() -> impl IntoView {
    let params = use_params_map();
    let username = get_username_memo(params);
    let sort_signal = RwSignal::new(SortType::Post(PostSortType::Hot));
    let post_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_post_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let initial_load = get_user_post_vec(username.get(), sort_signal.get(), 0).await;
            handle_initial_load(initial_load, post_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = post_vec.read_untracked().len();
                let additional_load = get_user_post_vec(username.get_untracked(), sort_signal.get_untracked(), num_post).await;
                handle_additional_load(additional_load, post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <PostSortWidget sort_signal/>
        <PostMiniatureList
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}

/// Displays a user's comments
#[component]
pub fn UserComments() -> impl IntoView {
    let params = use_params_map();
    let username = get_username_memo(params);
    let sort_signal = RwSignal::new(SortType::Comment(CommentSortType::Recent));
    let comment_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_comment_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let initial_load = get_user_comment_vec(username.get(), sort_signal.get(), 0).await;
            handle_initial_load(initial_load, comment_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_comment_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let additional_load = get_user_comment_vec(
                    username.get_untracked(),
                    sort_signal.get_untracked(),
                    comment_vec.read_untracked().len(),
                ).await;
                handle_additional_load(additional_load, comment_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <CommentSortWidget sort_signal/>
        <CommentMiniatureList
            comment_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}

/// Displays a user's settings
#[component]
pub fn UserSettings() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <div class="self-center flex flex-col gap-3 w-3/4 2xl:w-1/2">
            <Suspense fallback=move || view! {  <LoadingIcon/> }>
            {
                move || Suspend::new(async move {
                    let (is_nsfw, show_nsfw, days_hide_spoiler) = match state.user.await {
                        Ok(Some(user)) => (user.is_nsfw, user.show_nsfw, user.days_hide_spoiler.unwrap_or_default()),
                        _ => (false, false, 0),
                    };
                    view! {
                        <ActionForm action=state.set_settings_action attr:class="flex flex-col gap-3">
                            <LabeledFormCheckbox name="is_nsfw" label="NSFW profile" value=is_nsfw/>
                            <LabeledFormCheckbox name="show_nsfw" label="Show NSFW" value=show_nsfw/>
                            <div class="flex justify-between items-center">
                                "Hide spoilers duration (days)"
                                <input
                                    type="number"
                                    min="0"
                                    max="999"
                                    name="days_hide_spoilers"
                                    class="input input-primary no-spinner text-right w-16"
                                    autocomplete="off"
                                    value=days_hide_spoiler
                                />
                            </div>
                            <button type="submit" class="btn btn-secondary">
                                "Save"
                            </button>
                        </ActionForm>
                        <ActionError action=state.set_settings_action.into()/>
                    }
                })
            }
            </Suspense>
            <UserAccountButton/>
        </div>
    }
}

/// Button to navigate to the user's account on the OIDC provider
#[component]
pub fn UserAccountButton() -> impl IntoView {
    let navigate_to_account_action = ServerAction::<NavigateToUserAccount>::new();
    view! {
        <ActionForm action=navigate_to_account_action attr:class="flex justify-center items-center">
            <button type="submit" class="btn btn-primary flex">
                <UserSettingsIcon/>
                "Account"
            </button>
        </ActionForm>
    }.into_any()
}

pub fn get_profile_path(
    username: &str,
) -> String {
    format!("{USER_ROUTE_PREFIX}/{username}")
}

/// Get a memo returning the last valid user id from the url. Used to avoid triggering resources when leaving pages.
pub fn get_username_memo(params: Memo<ParamsMap>) -> Memo<String> {
    Memo::new(move |current_username: Option<&String>| {
        if let Some(new_username) = params.read().get_str(USER_ROUTE_PARAM_NAME) {
            new_username.to_string()
        } else {
            log::trace!("Could not find new user id, reuse current user id: {current_username:?}");
            current_username.cloned().unwrap_or_default()
        }
    })
}