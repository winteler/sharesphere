use cfg_if::cfg_if;
use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use std::collections::{BTreeSet};

use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::icons::{ErrorIcon, LoadingIcon};

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

    match sqlx::query(
        "INSERT INTO forums (name, description, nsfw, creator_id) VALUES ($1, $2, $3, $4)",
    )
        .bind(name.clone())
        .bind(description)
        .bind (is_nsfw.is_some())
        .bind(user.id)
        .execute(&db_pool)
        .await
    {
        Ok(_row) => {
            // Redirect to the new forum
            let new_forum_path : &str = &(FORUM_ROUTE_PREFIX.to_owned() + "/" + name.as_str());
            leptos_axum::redirect(new_forum_path);
            Ok(())
        },
        Err(e) => {
            log::error!("Error while creating new [[forum]] {e}");
            Err(ServerFnError::ServerError(e.to_string()))
        },
    }
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
                    existing_forums.get().map(|result| {
                        match result {
                            Ok(forum_set) => {
                                log::info!("Forum name set: {:?}", forum_set);
                                view! {
                                    <div class="flex flex-col gap-2 w-3/5 max-w-md 2xl:max-w-lg max-2xl:mx-auto">
                                        <ActionForm action=state.create_forum_action>
                                            <div class="flex flex-col gap-2 w-full">
                                                <h2 class="py-4 text-4xl max-2xl:text-center">"Create [[forum]]"</h2>
                                                <div class="flex gap-2 items-center">
                                                    <input
                                                        type="text"
                                                        name="name"
                                                        placeholder="[[Forum]] name"
                                                        class="input input-bordered input-primary h-16"
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
                            Err(e) => {
                                log::info!("Error while getting forum names: {}", e);
                                view! { <div>"Error"</div>}.into_view()
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
    let forum_id = move || {
        params.with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned()).unwrap_or_default()
    };
    // TODO: add forum banner
    view! {
        <div class="flex flex-col gap-1 w-full">
            <h2 class="text-4xl max-2xl:text-center">{forum_id}</h2>
            <Outlet/>
        </div>
    }
}

/// Component to display a forum's contents
#[component]
pub fn ForumContents() -> impl IntoView {
    // TODO: add list of forum contents
    view! {
        <h2 class="p-6 text-4xl max-2xl:text-center">"[[forum contents]]"</h2>
    }
}