use cfg_if::cfg_if;
use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::constants::{SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR};
use crate::fl;
use crate::icons::{AuthorIcon, ClockIcon, ErrorIcon, LoadingIcon, ScoreIcon, StacksIcon};
use crate::localization;
use crate::post::{get_posts_by_forum_name, Post, POST_ROUTE_PREFIX};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Forum {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_nsfw: bool,
    pub is_banned: bool,
    pub tags: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

pub const CREATE_FORUM_SUFFIX : &str = "/forum";
pub const CREATE_FORUM_ROUTE : &str = concatcp!(PUBLISH_ROUTE, CREATE_FORUM_SUFFIX);
pub const FORUM_ROUTE_PREFIX : &str = "/forums";
pub const FORUM_ROUTE_PARAM_NAME : &str = "forum_name";
pub const FORUM_ROUTE : &str = concatcp!(FORUM_ROUTE_PREFIX, PARAM_ROUTE_PREFIX, FORUM_ROUTE_PARAM_NAME);

#[server]
pub async fn create_forum( name: String, description: String, is_nsfw: Option<String>) -> Result<(), ServerFnError> {
    log::info!("Create [[forum]] '{name}', {description}, {:?}", is_nsfw);
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if name.is_empty() {
        return Err(ServerFnError::ServerError(String::from("Cannot create forum with empty name.")));
    }

    sqlx::query!(
        "INSERT INTO forums (name, description, is_nsfw, creator_id) VALUES ($1, $2, $3, $4)",
        name.clone(),
        description,
        is_nsfw.is_some(),
        user.id
    )
        .execute(&db_pool)
        .await?;

    // Redirect to the new forum
    let new_forum_path : &str = &(FORUM_ROUTE_PREFIX.to_owned() + "/" + name.as_str());
    leptos_axum::redirect(new_forum_path);
    Ok(())
}

#[server]
pub async fn get_forum_by_name(forum_name: String) -> Result<Forum, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum = sqlx::query_as!(
        Forum,
        "SELECT * FROM forums where name = $1",
        forum_name
    )
        .fetch_one(&db_pool)
        .await?;

    Ok(forum)
}

#[server]
pub async fn get_forum_by_name_map() -> Result<BTreeMap<String, Forum>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_vec = sqlx::query_as!(
        Forum,
        "SELECT * FROM forums"
    )
        .fetch_all(&db_pool)
        .await?;

    let mut forum_by_name_map = BTreeMap::<String, Forum>::new();

    for forum in forum_vec {
        forum_by_name_map.insert(forum.name.clone(), forum);
    }

    Ok(forum_by_name_map)
}

#[server]
pub async fn get_all_forum_names() -> Result<BTreeSet<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_name_vec = sqlx::query!("SELECT name FROM forums").fetch_all(&db_pool).await?;

    let mut forum_name_set = BTreeSet::<String>::new();

    for forum_name in forum_name_vec {
        forum_name_set.insert(forum_name.name);
    }

    Ok(forum_name_set)
}

#[server]
pub async fn get_subscribed_forums() -> Result<BTreeSet<String>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let forum_name_vec = sqlx::query!("SELECT name FROM forums").fetch_all(&db_pool).await?;

    let mut forum_name_set = BTreeSet::<String>::new();

    for forum_name in forum_name_vec {
        forum_name_set.insert(forum_name.name);
    }

    Ok(forum_name_set)

    // TODO: get subscribed forum
    /*let user = get_user().await;

    match user {
        Ok(user) => {
        }
        Err(_) => {
            let forum_name_vec = sqlx::query!("SELECT name FROM forums").fetch_all(&db_pool).await?;

            let mut forum_name_set: BTreeSet<String> = BTreeSet::with_capacity(forum_name_vec.len());
            for forum_name in forum_name_vec {
                forum_name_set.insert(forum_name.name);
            }

            Ok(forum_name_set)
        }
    }*/
}

/// Component to create new forums
#[component]
pub fn CreateForum() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let create_forum_result = state.create_forum_action.value();
    // check if the server has returned an error
    let has_error = move || create_forum_result.with(|val| matches!(val, Some(Err(_))));

    let existing_forums = create_blocking_resource( move || (state.create_forum_action.version().get()) , move |_| get_all_forum_names());

    let is_name_empty = create_rw_signal(true);
    let is_name_taken = create_rw_signal(false);
    let is_name_invalid = create_memo(move |_| { is_name_empty.get() || is_name_taken.get() });

    view! {
        <Transition fallback=move || (view! { <LoadingIcon/> })>
            {
                move || {
                    existing_forums.with(|result| {
                        match result {
                            Some(Ok(forum_set)) => {
                                let forum_set = forum_set.clone();
                                log::info!("Forum name set: {:?}", forum_set);
                                view! {
                                    <div class="flex flex-col gap-2 mx-auto w-1/2 2xl:w-1/3">
                                        <ActionForm action=state.create_forum_action>
                                            <div class="flex flex-col gap-2 w-full">
                                                <h2 class="py-4 text-4xl text-center">"Create [[forum]]"</h2>
                                                <div class="flex gap-2">
                                                    <input
                                                        type="text"
                                                        name="name"
                                                        placeholder="[[Forum]] name"
                                                        autocomplete="off"
                                                        class="input input-bordered input-primary h-16 flex-none w-1/2"
                                                        on:input=move |ev| {
                                                            let input = event_target_value(&ev);
                                                            is_name_empty.update(|is_empty: &mut bool| *is_empty = input.is_empty());
                                                            is_name_taken.update(|is_taken: &mut bool| *is_taken = forum_set.contains(&input));
                                                        }
                                                    />
                                                    <div class="alert alert-error" class:hidden=move || !is_name_taken.get()>
                                                        <ErrorIcon/>
                                                        <span>"Unavailable."</span>
                                                    </div>
                                                </div>
                                                <textarea name="description" placeholder="Description" class="textarea textarea-primary h-40 w-full"/>
                                                <div class="form-control">
                                                    <label class="cursor-pointer label p-0">
                                                        <span class="label-text">"NSFW content"</span>
                                                        <input type="checkbox" name="is_nsfw" class="checkbox checkbox-primary"/>
                                                    </label>
                                                </div>
                                                <button type="submit" class="btn btn-active btn-secondary" disabled=is_name_invalid>"Create"</button>
                                            </div>
                                        </ActionForm>
                                        <Show
                                            when=has_error
                                            fallback=move || ()
                                        >
                                            <div class="alert alert-error flex justify-center">
                                                <ErrorIcon/>
                                                <span>"Server error. Please reload the page and retry."</span>
                                            </div>
                                        </Show>
                                    </div>
                                }.into_view()
                            }
                            Some(Err(e)) => {
                                log::info!("Error while getting forum names: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            },
                            None => {
                                log::trace!("Resource not loaded yet.");
                                view! { <ErrorIcon/> }.into_view()
                            }
                        }
                    })
                }
            }
        </Transition>
    }
}

/// Component to display a forum's banner
#[component]
pub fn ForumBanner() -> impl IntoView {

    let params = use_params_map();
    let forum_name = move || {
        params.with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned()).unwrap_or_default()
    };
    let forum_path = move || FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name().as_str();

    let forum = create_resource(move || (), move |_| get_forum_by_name(forum_name()));
    // TODO: add forum banner
    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            {
                move || {
                     forum.with(|result| match result {
                        Some(Ok(forum)) => {
                            let forum_banner_image = format!("url({})", forum.banner_url.clone().unwrap_or(String::from("https://daisyui.com/images/stock/photo-1507358522600-9f71e620c44e.jpg")));
                            view! {
                                <div class="flex flex-col w-full">
                                    <div
                                        class="hero bg-blue-500"
                                        style:background-image=forum_banner_image
                                    >
                                        <div class="hero-overlay bg-opacity-0"></div>
                                        <div class="hero-content text-neutral-content text-left">
                                            <a href=forum_path class="btn btn-ghost normal-case text-l">
                                                <StacksIcon/>
                                                <h2 class="text-4xl">{forum_name}</h2>
                                            </a>
                                        </div>
                                    </div>
                                    <Outlet/>
                                </div>
                            }.into_view()
                        },
                        Some(Err(e)) => {
                            log::info!("Error: {}", e);
                            view! { <ErrorIcon/> }.into_view()
                        },
                        None => {
                            log::trace!("Resource not loaded yet.");
                            view! { <Outlet/> }.into_view()
                        },
                    })
                }
            }
        </Transition>
    }
}

/// Component to display a forum's contents
#[component]
pub fn ForumContents() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let params = use_params_map();
    let forum_name = move || {
        params.with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned()).unwrap_or_default()
    };

    let post_vec = create_resource(move || (state.create_post_action.version().get()), move |_| get_posts_by_forum_name(forum_name()));

    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            {
                move || {
                     post_vec.with(|result| match result {
                        Some(Ok(post_vec)) => {
                            view! { <ForumPostMiniatures post_vec=post_vec forum_name=forum_name()/> }.into_view()
                        },
                        Some(Err(e)) => {
                            log::info!("Error: {}", e);
                            view! { <ErrorIcon/> }.into_view()
                        },
                        None => {
                            log::trace!("Resource not loaded yet.");
                            view! { <ErrorIcon/> }.into_view()
                        },
                    })
                }
            }
        </Transition>
    }
}

/// Component to display a given set of forum posts
#[component]
pub fn ForumPostMiniatures<'a>(post_vec: &'a Vec<Post>, forum_name: String) -> impl IntoView {
    view! {
        <ul class="menu w-full text-lg">
            {
                post_vec.iter().map(move |post| {
                    let post_path = FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name.as_str() + POST_ROUTE_PREFIX + "/" + &post.id.to_string();
                    view! {
                        <li>
                            <a href=post_path>
                                <div class="flex flex-col gap-1">
                                    <h2 class="card-title">{post.title.clone()}</h2>
                                    <div class="flex gap-1">
                                        <div class="flex rounded-btn px-1 gap-1 items-center">
                                            <ScoreIcon/>
                                            {post.score.clone()}
                                        </div>
                                        <div class="flex rounded-btn px-1 gap-1 items-center">
                                            <AuthorIcon/>
                                            {post.creator_name.clone()}
                                        </div>
                                        <div class="flex rounded-btn px-1 gap-1 items-center">
                                            <ClockIcon/>
                                            {
                                                let post_age = chrono::Utc::now().signed_duration_since(post.create_timestamp);
                                                let seconds = post_age.num_seconds();

                                                match seconds {
                                                    seconds if seconds < SECONDS_IN_MINUTE => {
                                                        format!("{} {}", seconds, if seconds == 1 { "second" } else { "seconds" })
                                                    },
                                                    seconds if seconds < SECONDS_IN_HOUR => {
                                                        let minutes = seconds/SECONDS_IN_MINUTE;
                                                        format!("{} {}", minutes, if minutes == 1 { "minute" } else { "minutes" })
                                                        //fl!("time_unit_count", count=minutes, unit="minute")
                                                    },
                                                    seconds if seconds < SECONDS_IN_DAY => {
                                                        let hours = seconds/SECONDS_IN_HOUR;
                                                        format!("{} {}", hours, if hours == 1 { "hour" } else { "hours" })
                                                    },
                                                    seconds if seconds < SECONDS_IN_MONTH => {
                                                        let days = seconds/SECONDS_IN_DAY;
                                                        format!("{} {}", days, if days == 1 { "day" } else { "days" })
                                                    },
                                                    seconds if seconds < SECONDS_IN_YEAR => {
                                                        let months = seconds/SECONDS_IN_MONTH;
                                                        format!("{} {}", months, if months == 1 { "month" } else { "months" })
                                                    },
                                                    _ => {
                                                        let years = seconds/SECONDS_IN_YEAR;
                                                        format!("{} {}", years, if years == 1 { "year" } else { "years" })
                                                    },
                                                }
                                            }
                                        </div>
                                    </div>
                                </div>
                            </a>
                        </li>
                    }
                }).collect_view()
            }
        </ul>
    }
}
