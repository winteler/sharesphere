use cfg_if::cfg_if;
use const_format::concatcp;
use leptos::*;
use leptos_router::{ActionForm};

use crate::app::{GlobalState, PUBLISH_ROUTE};
use crate::forum::{get_all_forum_names};
use crate::icons::{ErrorIcon, LoadingIcon};


cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

pub const CREATE_POST_SUFFIX : &str = "/content";
pub const CREATE_POST_ROUTE : &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);

#[server]
pub async fn create_post(forum_id: i64, title: String, body: String, is_nsfw: Option<String>, tag: Option<String>) -> Result<(), ServerFnError> {
    log::info!("Create [[content]] '{title}'");
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if title.is_empty() || body.is_empty() {
        return Err(ServerFnError::ServerError(String::from("Cannot create content with empty title.")));
    }

    match sqlx::query(
        "INSERT INTO posts (title, body, nsfw, tag, forum_id, creator_id) VALUES ($1, $2, $3, $4, $5, $6)",
    )
        .bind(title.clone())
        .bind(body)
        .bind(is_nsfw.is_some())
        .bind(tag.unwrap_or_default())
        .bind(forum_id)
        .bind(user.id)
        .execute(&db_pool)
        .await
    {
        Ok(_row) => {
            // Redirect to the new post
            // TODO: redirect to new post
            let new_post_path : &str = "/";
            leptos_axum::redirect(new_post_path);
            Ok(())
        },
        Err(e) => {
            log::error!("Error while creating new [[forum]] {e}");
            Err(ServerFnError::ServerError(e.to_string()))
        },
    }
}

/// Component to create a new content
#[component]
pub fn CreatePost() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let create_post_result = state.create_post_action.value();
    // check if the server has returned an error
    let has_error = move || create_post_result.with(|val| matches!(val, Some(Err(_))));

    let forum_name_input = create_rw_signal(String::default());
    let is_title_empty = create_rw_signal(true);
    let is_body_empty = create_rw_signal(true);
    let is_content_invalid = create_memo(move |_| { is_title_empty.get() || is_body_empty.get() });

    let existing_forums = create_blocking_resource( move || (state.create_forum_action.version().get()), move |_| get_all_forum_names());

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
                                        <ActionForm action=state.create_post_action>
                                            <div class="flex flex-col gap-2 w-full">
                                                <h2 class="py-4 text-4xl max-2xl:text-center">"Create [[content]]"</h2>
                                                <div class="dropdown dropdown-end">
                                                    <input
                                                        tabindex="0"
                                                        type="text"
                                                        name="forum"
                                                        placeholder="[[Forum]]"
                                                        autocomplete="off"
                                                        class="input input-bordered input-primary w-full"
                                                        on:input=move |ev| {
                                                            forum_name_input.update(|name: &mut String| *name = event_target_value(&ev));
                                                        }
                                                        prop:value=forum_name_input
                                                    />
                                                    <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full">
                                                        {
                                                            move || {
                                                                forum_set.iter().filter(|forum| forum.starts_with(forum_name_input.get().as_str())).map(|forum_name| {
                                                                    view! {
                                                                        <li>
                                                                            <button value=forum_name on:click=move |ev| forum_name_input.update(|name| *name = event_target_value(&ev))>
                                                                                {forum_name}
                                                                            </button>
                                                                        </li>
                                                                    }
                                                                }).collect_view()
                                                            }
                                                        }
                                                    </ul>
                                                </div>
                                                <input
                                                    type="text"
                                                    name="title"
                                                    placeholder="Title"
                                                    class="input input-bordered input-primary h-16"
                                                    on:input=move |ev| {
                                                        is_title_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                                                    }
                                                />
                                                <textarea
                                                    name="body"
                                                    placeholder="Text"
                                                    class="textarea textarea-primary h-40 w-full"
                                                    on:input=move |ev| {
                                                        is_body_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                                                    }
                                                />
                                                <select name="tag" class="select select-bordered w-full max-w-xs">
                                                    <option disabled selected>"Tag"</option>
                                                    <option>"This should be"</option>
                                                    <option>"Customized"</option>
                                                </select>
                                                <button type="submit" class="btn btn-active btn-secondary" disabled=is_content_invalid>"Create"</button>
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
                                view! { <ErrorIcon/> }.into_view()
                            }
                        }
                    })
                }
            }
        </Transition>
    }
}

/// Component to display a content
#[component]
pub fn Content() -> impl IntoView {

    // TODO: add content and its comments
    view! {
        <h2 class="p-6 text-4xl max-2xl:text-center">"[[content]]"</h2>
    }
}