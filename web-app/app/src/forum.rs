use std::collections::BTreeSet;

use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use leptos_use::signal_debounced;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user, auth::ssr::reload_user};
use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::auth::LoginGuardButton;
#[cfg(feature = "ssr")]
use crate::auth::ssr::check_user;
use crate::editor::FormTextEditor;
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::forum_management::MANAGE_FORUM_SUFFIX;
use crate::icons::{InternalErrorIcon, LoadingIcon, LogoIcon, PlusIcon, SettingsIcon, StarIcon, SubscribedIcon};
use crate::navigation_bar::get_create_post_path;
use crate::post::{get_post_vec_by_forum_name, POST_BATCH_SIZE};
use crate::post::{
    CREATE_POST_FORUM_QUERY_PARAM, CREATE_POST_ROUTE, Post, POST_ROUTE_PREFIX,
};
use crate::ranking::ScoreIndicator;
use crate::unpack::{SuspenseUnpack, TransitionUnpack};
use crate::widget::{AuthorWidget, PostSortWidget, TimeSinceWidget};

pub const CREATE_FORUM_SUFFIX: &str = "/forum";
pub const CREATE_FORUM_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_FORUM_SUFFIX);
pub const FORUM_ROUTE_PREFIX: &str = "/forums";
pub const FORUM_ROUTE_PARAM_NAME: &str = "forum_name";
pub const FORUM_ROUTE: &str = concatcp!(
    FORUM_ROUTE_PREFIX,
    PARAM_ROUTE_PREFIX,
    FORUM_ROUTE_PARAM_NAME
);

pub const FORUM_FETCH_LIMIT: i64 = 20;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Forum {
    pub forum_id: i64,
    pub forum_name: String,
    pub description: String,
    pub is_nsfw: bool,
    pub is_banned: bool,
    pub tags: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub num_members: i32,
    pub creator_id: i64,
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

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::collections::BTreeSet;

    use sqlx::PgPool;

    use crate::auth::User;
    use crate::errors::AppError;
    use crate::errors::AppError::InternalServerError;
    use crate::forum::{Forum, ForumWithUserInfo};
    use crate::role::PermissionLevel;
    use crate::role::ssr::set_user_forum_role;

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
            "SELECT f.*, s.subscription_id \
            FROM forums f \
            LEFT JOIN forum_subscriptions s ON \
                s.forum_id = f.forum_id AND \
                s.user_id = $1 \
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
            "SELECT forum_id FROM forums WHERE forum_name = $1",
            forum_name
        )
        .fetch_one(db_pool)
        .await;

        match forum_exist {
            Ok(_) => Ok(false),
            Err(sqlx::error::Error::RowNotFound) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_matching_forum_names(
        forum_prefix: String,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<BTreeSet<String>, AppError> {
        let forum_name_vec = sqlx::query!(
            "SELECT forum_name FROM forums WHERE forum_name like $1 LIMIT $2",
            forum_prefix + "%",
            limit,
        )
        .fetch_all(db_pool)
        .await?;

        let mut forum_name_set = BTreeSet::<String>::new();

        for row in forum_name_vec {
            forum_name_set.insert(row.forum_name);
        }

        Ok(forum_name_set)
    }

    pub async fn get_popular_forum_names(
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<String>, AppError> {
        let forum_record_vec = sqlx::query!(
            "SELECT * FROM forums ORDER BY num_members DESC, forum_name LIMIT $1",
            limit
        )
        .fetch_all(db_pool)
        .await?;

        let mut forum_name_vec = Vec::<String>::with_capacity(forum_record_vec.len());

        for forum in forum_record_vec {
            forum_name_vec.push(forum.forum_name);
        }

        Ok(forum_name_vec)
    }

    pub async fn get_subscribed_forum_names(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<String>, AppError> {
        let forum_record_vec = sqlx::query!(
            "SELECT f.forum_name FROM forums f \
            JOIN forum_subscriptions s ON \
                f.forum_id = s.forum_id AND \
                s.user_id = $1 \
            ORDER BY forum_name",
            user_id,
        )
        .fetch_all(db_pool)
        .await?;

        let mut forum_name_vec = Vec::<String>::with_capacity(forum_record_vec.len());

        for forum in forum_record_vec {
            forum_name_vec.push(forum.forum_name);
        }

        Ok(forum_name_vec)
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

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() && (c.is_lowercase() || c.is_numeric()))
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

        set_user_forum_role(forum.forum_id, &forum.forum_name, user.user_id, PermissionLevel::Lead, user, &db_pool).await?;

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
pub async fn is_forum_available(forum_name: String) -> Result<bool, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_existence = ssr::is_forum_available(&forum_name, &db_pool).await?;
    Ok(forum_existence)
}

#[server]
pub async fn get_forum_by_name(forum_name: String) -> Result<Forum, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum = ssr::get_forum_by_name(&forum_name, &db_pool).await?;
    Ok(forum)
}

#[server]
pub async fn get_matching_forum_names(
    forum_prefix: String,
) -> Result<BTreeSet<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_name_set =
        ssr::get_matching_forum_names(forum_prefix, FORUM_FETCH_LIMIT, &db_pool).await?;
    Ok(forum_name_set)
}

#[server]
pub async fn get_subscribed_forum_names() -> Result<Vec<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    match get_user().await {
        Ok(Some(user)) => {
            let forum_name_vec = ssr::get_subscribed_forum_names(user.user_id, &db_pool).await?;
            Ok(forum_name_vec)
        }
        _ => Ok(Vec::<String>::new()),
    }
}

#[server]
pub async fn get_popular_forum_names() -> Result<Vec<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_name_vec = ssr::get_popular_forum_names(FORUM_FETCH_LIMIT, &db_pool).await?;
    Ok(forum_name_vec)
}

#[server]
pub async fn get_forum_with_user_info(
    forum_name: String,
) -> Result<ForumWithUserInfo, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };

    let forum_content =
        ssr::get_forum_with_user_info(forum_name.as_str(), user_id, &db_pool).await?;

    Ok(forum_content)
}

#[server]
pub async fn create_forum(
    forum_name: String,
    description: String,
    is_nsfw: bool,
) -> Result<(), ServerFnError> {
    log::info!("Create Sphere '{forum_name}', {description}, {is_nsfw}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let new_forum_path: &str = &(FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name.as_str());

    ssr::create_forum(
        forum_name.as_str(),
        description.as_str(),
        is_nsfw,
        &user,
        &db_pool,
    ).await?;

    reload_user(user.user_id)?;

    // Redirect to the new forum
    leptos_axum::redirect(new_forum_path);
    Ok(())
}

#[server]
pub async fn subscribe(forum_id: i64) -> Result<(), ServerFnError> {
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    ssr::subscribe(forum_id, user.user_id, &db_pool).await?;

    Ok(())
}

#[server]
pub async fn unsubscribe(forum_id: i64) -> Result<(), ServerFnError> {
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    ssr::unsubscribe(forum_id, user.user_id, &db_pool).await?;
    Ok(())
}

/// Get the current forum name from the path. When the current path does not contain a forum, returns the last valid forum. Used to avoid sending a request when leaving a page
pub fn get_forum_name_memo(params: Memo<ParamsMap>) -> Memo<String> {
    create_memo(move |current_forum_name: Option<&String>| {
        if let Some(new_forum_name) =
            params.with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned())
        {
            log::trace!(
                "Current forum name {current_forum_name:?}, new forum name: {new_forum_name}"
            );
            new_forum_name
        } else {
            log::trace!("No valid forum name, keep current value: {current_forum_name:?}");
            current_forum_name.cloned().unwrap_or_default()
        }
    })
}

/// Component to display a forum's banner
#[component]
pub fn ForumBanner() -> impl IntoView {
    let params = use_params_map();
    let forum_name = get_forum_name_memo(params);
    let forum_path = move || {
        FORUM_ROUTE_PREFIX.to_owned()
            + "/"
            + params
                .with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned())
                .unwrap_or_default()
                .as_ref()
    };

    let forum_resource = create_resource(
        move || forum_name(),
        move |forum_name| {
            log::debug!("Load data for forum: {forum_name}");
            get_forum_by_name(forum_name)
        },
    );

    view! {
        <div class="flex flex-col gap-2 pt-2 px-2 w-full">
            <TransitionUnpack resource=forum_resource let:forum>
            {
                let forum_banner_image = format!("url({})", forum.banner_url.clone().unwrap_or(String::from("/banner.jpg")));
                view! {
                    <a
                        href=forum_path()
                        class="flex-none bg-cover bg-center bg-no-repeat rounded w-full h-24 flex items-center justify-center"
                        style:background-image=forum_banner_image
                    >
                        <div class="p-3 backdrop-blur bg-black/50 rounded-sm flex justify-center gap-3">
                            <LogoIcon class="h-12 w-12"/>
                            <span class="text-4xl">{forum_name()}</span>
                        </div>
                    </a>
                }
            }
            </TransitionUnpack>
            <Outlet/>
        </div>
    }
}

/// Component to display a forum's contents
#[component]
pub fn ForumContents() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let params = use_params_map();
    let forum_name = get_forum_name_memo(params);
    let post_vec = create_rw_signal(Vec::<Post>::with_capacity(POST_BATCH_SIZE as usize));
    let additional_load_count = create_rw_signal(0);
    let is_loading = create_rw_signal(false);
    let load_error = create_rw_signal(None);
    let list_ref = create_node_ref::<html::Ul>();
    let forum_with_sub_resource = create_resource(
        move || (forum_name(),),
        move |(forum_name,)| get_forum_with_user_info(forum_name),
    );

    // Effect for initial load, forum and sort changes
    create_effect(move |_| {
        let forum_name = forum_name.get();
        let sort_type = state.post_sort_type.get();
        is_loading.set(true);
        load_error.set(None);
        post_vec.update(|post_vec| post_vec.clear());
        spawn_local(async move {
            match get_post_vec_by_forum_name(forum_name, sort_type, 0).await {
                Ok(new_post_vec) => {
                    post_vec.update(|post_vec| {
                        if let Some(list_ref) = list_ref.get_untracked() {
                            list_ref.set_scroll_top(0);
                        }
                        *post_vec = new_post_vec;
                    });
                },
                Err(e) => load_error.set(Some(AppError::from(&e))),
            }
            is_loading.set(false);
        });
    });

    // Effect for additional load upon reaching end of scroll
    create_effect(move |_| {
        if additional_load_count.get() > 0 {
            is_loading.set(true);
            load_error.set(None);
            let post_count = post_vec.with_untracked(|post_vec| post_vec.len());
            spawn_local(async move {
                match get_post_vec_by_forum_name(forum_name.get_untracked(), state.post_sort_type.get_untracked(), post_count).await {
                    Ok(mut new_post_vec) => post_vec.update(|post_vec| post_vec.append(&mut new_post_vec)),
                    Err(e) => load_error.set(Some(AppError::from(&e)))
                }
                is_loading.set(false);
            });
        }
    });

    view! {
        <SuspenseUnpack
            resource=forum_with_sub_resource let:forum_with_sub
        >
            <ForumToolbar forum=&forum_with_sub/>
        </SuspenseUnpack>
        <ForumPostMiniatures
            post_vec=post_vec
            is_loading=is_loading
            load_error=load_error
            additional_load_count=additional_load_count
            list_ref=list_ref
        />
    }
}

/// Component to display the forum toolbar
#[component]
pub fn ForumToolbar<'a>(forum: &'a ForumWithUserInfo) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_id = forum.forum.forum_id;
    let forum_name = forum.forum.forum_name.clone();
    let can_moderate_forum = Signal::derive(move || state.user.with(|user| match user {
        Some(Ok(Some(user))) => user.check_can_moderate_forum(&forum_name).is_ok(),
        _ => false,
    }));
    let forum_name = create_rw_signal(forum.forum.forum_name.clone());
    let is_subscribed = create_rw_signal(forum.subscription_id.is_some());

    view! {
        <div class="flex w-full justify-between content-center">
            <PostSortWidget/>
            <div class="flex gap-1">
                <Show when=can_moderate_forum>
                    <A href=MANAGE_FORUM_SUFFIX class="btn btn-circle btn-ghost">
                        <SettingsIcon class="h-5 w-5"/>
                    </A>
                </Show>
                <div class="tooltip" data-tip="Join">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <StarIcon class="h-6 w-6" show_colour=is_subscribed/> }
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
                            <StarIcon class="h-6 w-6" show_colour=is_subscribed/>
                        </button>
                    </LoginGuardButton>
                </div>
                <div class="tooltip" data-tip="Join">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <SubscribedIcon class="h-6 w-6" show_colour=is_subscribed/> }
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
                            <SubscribedIcon class="h-6 w-6" show_colour=is_subscribed/>
                        </button>
                    </LoginGuardButton>
                </div>
                <div class="tooltip" data-tip="New">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <PlusIcon class="h-6 w-6"/> }
                        redirect_path_fn=&get_create_post_path
                        let:_user
                    >
                        <Form action=CREATE_POST_ROUTE class="flex">
                            <input type="text" name=CREATE_POST_FORUM_QUERY_PARAM class="hidden" value=forum_name/>
                            <button type="submit" class="btn btn-circle btn-ghost">
                                <PlusIcon class="h-6 w-6"/>
                            </button>
                        </Form>
                    </LoginGuardButton>
                </div>
            </div>
        </div>
    }
}

/// Component to display a vector of forum posts and indicate when more need to be loaded
#[component]
pub fn ForumPostMiniatures(
    /// signal containing the posts to display
    post_vec: RwSignal<Vec<Post>>,
    /// signal indicating new posts are being loaded
    is_loading: RwSignal<bool>,
    /// signal containing an eventual loading error in order to display it
    load_error: RwSignal<Option<AppError>>,
    /// signal to request loading additional posts
    additional_load_count: RwSignal<i64>,
    /// reference to the container of the posts in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col overflow-y-auto w-full pr-2 divide-y divide-base-content/20 "
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
                key=|(_index, post)| post.post_id
                // renders each item to a view
                children=move |(_key, post)| {
                    let post_path = FORUM_ROUTE_PREFIX.to_owned() + "/" + post.forum_name.as_str() + POST_ROUTE_PREFIX + "/" + &post.post_id.to_string();
                    view! {
                        <li>
                            <a href=post_path>
                                <div class="flex flex-col gap-1 pt-1 pb-2 my-1 rounded hover:bg-base-content/20">
                                    <h2 class="card-title pl-1">{post.title.clone()}</h2>
                                    <div class="flex gap-1">
                                        <ScoreIndicator score=post.score/>
                                        <AuthorWidget author=post.creator_name.clone()/>
                                        <TimeSinceWidget timestamp=post.create_timestamp/>
                                    </div>
                                </div>
                            </a>
                        </li>
                    }
                }
            />
            <Show when=move || load_error.with(|error| error.is_some())>
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
    let create_forum_result = state.create_forum_action.value();
    // check if the server has returned an error
    let has_error = move || create_forum_result.with(|val| matches!(val, Some(Err(_))));

    let forum_name = create_rw_signal(String::new());
    let forum_name_debounced: Signal<String> = signal_debounced(forum_name, 250.0);
    let is_forum_available = create_resource(
        move || forum_name_debounced.get(),
        move |forum_name| async {
            if forum_name.is_empty() {
                None
            } else {
                Some(is_forum_available(forum_name).await)
            }
        },
    );

    let is_name_taken = create_rw_signal(false);
    let description = create_rw_signal(String::new());
    let is_nsfw = create_rw_signal(false);
    let is_name_empty = move || forum_name.with(|forum_name| forum_name.is_empty());
    let is_name_alphanumeric =
        move || forum_name.with(|forum_name| forum_name.chars().all(char::is_alphanumeric));
    let are_inputs_invalid = create_memo(move |_| {
        is_name_empty()
            || is_name_taken()
            || !is_name_alphanumeric()
            || description.with(|description| description.is_empty())
    });
    let is_nsfw_string = move || is_nsfw.get().to_string();

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
                                forum_name.update(|value| *value = event_target_value(&ev).to_lowercase());
                            }
                            prop:value=forum_name
                        />
                        <Suspense fallback=move || view! { <LoadingIcon/> }>
                        {
                            move || is_forum_available.map(|result| match result {
                                None | Some(Ok(true)) => {
                                    is_name_taken.set(false);
                                    View::default()
                                },
                                Some(Ok(false)) => {
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-input_l flex items-center justify-center">
                                            <span class="font-semibold">"Unavailable"</span>
                                        </div>
                                    }.into_view()
                                },
                                Some(Err(e)) => {
                                    log::error!("Error while checking forum existence: {e}");
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-input_l flex items-center justify-center">
                                            <InternalErrorIcon class="h-16 w-16"/>
                                            <span class="font-semibold">"Server error"</span>
                                        </div>
                                    }.into_view()
                                },
                            })

                        }
                        </Suspense>
                        <div class="alert alert-error h-input_l flex content-center" class:hidden=move || is_name_empty() || is_name_alphanumeric()>
                            <InternalErrorIcon/>
                            <span>"Only alphanumeric characters."</span>
                        </div>
                    </div>
                    <FormTextEditor
                        name="description"
                        placeholder="Description"
                        content=description
                    />
                    <div class="form-control">
                        <input type="text" name="is_nsfw" value=is_nsfw_string class="hidden"/>
                        <label class="cursor-pointer label p-0">
                            <span class="label-text">"NSFW content"</span>
                            <input type="checkbox" class="checkbox checkbox-primary" checked=is_nsfw on:click=move |_| is_nsfw.update(|value| *value = !*value)/>
                        </label>
                    </div>
                    <Suspense fallback=move || view! { <LoadingIcon/> }>
                        <button type="submit" class="btn btn-active btn-secondary" disabled=are_inputs_invalid>"Create"</button>
                    </Suspense>
                </div>
            </ActionForm>
            <Show
                when=has_error
                fallback=move || ()
            >
                <div class="alert alert-error flex justify-center">
                    <InternalErrorIcon/>
                    <span>"Server error. Please reload the page and retry."</span>
                </div>
            </Show>
        </div>
    }
}