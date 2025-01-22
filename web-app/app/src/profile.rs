use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map};
use leptos_router::params::ParamsMap;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::comment::{Comment, CommentBody, CommentSortType};
use crate::content::{CommentSortWidget, PostSortWidget};
use crate::errors::AppError;
use crate::post::{PostSortType, PostWithSphereInfo};
use crate::ranking::SortType;
use crate::sidebar::HomeSidebar;
use crate::sphere::SpherePostMiniatures;
use crate::unpack::{handle_additional_load, handle_initial_load};
use crate::widget::{EnumQueryTabs, ToView};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    comment::COMMENT_BATCH_SIZE,
    post::{POST_BATCH_SIZE},
};

pub const USER_ROUTE_PREFIX: &str = "/users";
pub const USER_ROUTE_PARAM_NAME: &str = "username";
pub const PROFILE_TAB_QUERY_PARAM: &str = "tab";

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ProfileTabs {
    Posts,
    Comments,
    Settings,
}

impl ToView for ProfileTabs {
    fn to_view(self) -> AnyView {
        match self {
            ProfileTabs::Posts => view! { <UserPosts/> }.into_any(),
            ProfileTabs::Comments => view! { <UserComments/> }.into_any(),
            ProfileTabs::Settings => view! { <UserSettings/> }.into_any(),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use crate::comment::Comment;
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
                    p.creator_name = $1
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
    ) -> leptos::error::Result<Vec<Comment>, AppError> {
        let comment_vec = sqlx::query_as::<_, Comment>(
            format!(
                "SELECT * FROM comments
                WHERE creator_name = $1
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
) -> Result<Vec<Comment>, ServerFnError<AppError>> {
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
    view! {
        <EnumQueryTabs query_param=PROFILE_TAB_QUERY_PARAM query_enum_iter=ProfileTabs::iter() default_view=move || view! { <UserPosts/> }/>
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
        <SpherePostMiniatures
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
                let num_post = comment_vec.read_untracked().len();
                let additional_load = get_user_comment_vec(username.get_untracked(), sort_signal.get_untracked(), num_post).await;
                handle_additional_load(additional_load, comment_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <CommentSortWidget sort_signal/>
        <For
            each= move || comment_vec.get().into_iter().enumerate()
            key=|(_index, comment)| comment.comment_id
            children=move |(_, comment)| {
                view! {
                    <CommentBody comment/>
                }.into_any()
            }
        />
    }
}

/// Displays a user's settings
#[component]
pub fn UserSettings() -> impl IntoView {
    let params = use_params_map();
    let username = get_username_memo(params);
    view! {
        <div>"Settings"</div>
        <div>{move || username.get()}</div>
    }
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