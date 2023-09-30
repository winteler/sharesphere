use cfg_if::cfg_if;
use std::collections::{HashSet};
use leptos::*;
use leptos_router::{ActionForm, Outlet};

use crate::icons::{ErrorIcon, LoadingIcon};



cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

pub const CREATE_FORUM_ROUTE : &str = "/forum";

#[server(CreateForum, "/api")]
pub async fn create_forum(cx: Scope, name: String, description: String, is_nsfw: Option<String>) -> Result<(), ServerFnError> {
    log!("Create [[forum]] '{name}', {description}, {:?}", is_nsfw);
    let user = get_user(cx).await?;
    log!("Could get user: {:?}", user);

    let db_pool = get_db_pool(cx)?;
    log!("Got db pool");

    if name.is_empty() {
        return Err(ServerFnError::ServerError(String::from("Cannot create forum with empty name.")));
    }

    let result = match sqlx::query(
        "INSERT INTO forums (name, description, nsfw, creator_id) VALUES ($1, $2, $3, $4)",
    )
        .bind(name)
        .bind(description)
        .bind (is_nsfw.is_some())
        .bind(user.id)
        .execute(&db_pool)
        .await
    {
        Ok(_row) => Ok(()),
        Err(e) => {
            error!("Error while creating new [[forum]] {e}");
            Err(ServerFnError::ServerError(e.to_string()))
        },
    };

    // TODO: on success redirect to new forum
    return result;
}

#[server(GetAllForumNames, "/api")]
pub async fn get_all_forum_names(cx: Scope) -> Result<HashSet<String>, ServerFnError> {
    let db_pool = get_db_pool(cx)?;
    log!("Got db pool");

    let forum_name_vec = sqlx::query!("SELECT name FROM forums").fetch_all(&db_pool).await?;

    let mut forum_name_set: HashSet<String> = HashSet::with_capacity(forum_name_vec.len());
    for forum_name in forum_name_vec {
        forum_name_set.insert(forum_name.name);
    }

    Ok(forum_name_set)
}

#[component]
pub fn CreateForum(cx: Scope) -> impl IntoView {
    use leptos_router::FromFormData;

    let create_forum = create_server_action::<CreateForum>(cx);
    let create_forum_result = create_forum.value();
    // check if the server has returned an error
    let has_error = move || create_forum_result.with(|val| matches!(val, Some(Err(_))));

    let existing_forums = create_resource(cx, move || (create_forum.version()) , move |_| get_all_forum_names(cx));

    let is_name_empty = create_rw_signal(cx, true);
    let is_name_taken = create_rw_signal(cx, false);
    let is_name_invalid = create_memo(cx, move |_| { is_name_empty.get() || is_name_taken.get() });

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        let data = CreateForum::from_event(&event);
        if data.is_err() || data.unwrap().name == "nope!" {
            // ev.prevent_default() will prevent form submission
            event.prevent_default();
        }
    };

    view! { cx,
        <Suspense fallback=move || (view! {cx, <LoadingIcon/>})>
                {
                    move || {
                        existing_forums.read(cx).map(|result| {
                            match result {
                                Ok(forum_set) => {
                                    log!("Forum name set: {:?}", forum_set);
                                    view! {cx,
                                        <div class="flex flex-col gap-1 max-w-md 2xl:max-w-lg max-2xl:mx-auto">
                                            <ActionForm action=create_forum on:submit=on_submit>
                                                <div class="flex flex-col gap-1 w-full">
                                                    <h2 class="p-6 text-4xl max-2xl:text-center">"Create [[forum]]"</h2>
                                                    <div class="flex gap-1 items-center">
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
                                                    <textarea name="description" placeholder="Description" class="textarea textarea-primary h-40"/>
                                                    <div class="form-control">
                                                        <label class="cursor-pointer label">
                                                            <span class="label-text">"NSFW content"</span>
                                                            <input type="checkbox" name="is_nsfw" class="checkbox checkbox-primary"/>
                                                        </label>
                                                    </div>
                                                    <button type="submit" class="btn btn-active btn-secondary" disabled=is_name_invalid>"Create"</button>
                                                </div>
                                            </ActionForm>
                                            <Show
                                                when=has_error
                                                fallback=move |_| ()
                                            >
                                                <div class="alert alert-error flex justify-center">
                                                    <ErrorIcon/>
                                                    <span>"Server error. Please reload the page and retry."</span>
                                                </div>
                                            </Show>
                                        </div>
                                    }.into_view(cx)
                                }
                                Err(e) => {
                                    log!("Error while getting forum names: {}", e);
                                    view! {cx, <div>"Error"</div>}.into_view(cx)
                                }
                            }
                        })
                    }
                }
        </Suspense>
    }
}

#[component]
pub fn ForumBanner(cx: Scope) -> impl IntoView {

    // TODO: add forum banner
    view! { cx,
        <div class="flex flex-col gap-1 w-full">
            <h2 class="p-6 text-4xl max-2xl:text-center">"[[forum banner]]"</h2>
            <Outlet/>
        </div>

    }
}

#[component]
pub fn ForumContents(cx: Scope) -> impl IntoView {
    // TODO: add list of forum contents
    view! { cx,
        <h2 class="p-6 text-4xl max-2xl:text-center">"[[forum contents]]"</h2>
    }
}

#[component]
pub fn Content(cx: Scope) -> impl IntoView {

    // TODO: add content and its comments
    view! { cx,
        <h2 class="p-6 text-4xl max-2xl:text-center">"[[content]]"</h2>
    }
}