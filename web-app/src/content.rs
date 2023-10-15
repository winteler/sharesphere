use cfg_if::cfg_if;
use const_format::concatcp;
use leptos::*;
use leptos_router::{ActionForm};

use crate::app::PUBLISH_ROUTE;
use crate::forum::get_all_forum_names;
use crate::icons::{ErrorIcon, LoadingIcon};


cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

pub const CREATE_CONTENT_SUFFIX : &str = "/content";
pub const CREATE_CONTENT_ROUTE : &str = concatcp!(PUBLISH_ROUTE, CREATE_CONTENT_SUFFIX);

#[server]
pub async fn create_content( title: String, body: String, tags: Option<String>) -> Result<(), ServerFnError> {
    log::info!("Create [[content]] '{title}'");
    let user = get_user().await?;
    log::info!("Could get user: {:?}", user);

    let db_pool = get_db_pool()?;
    log::info!("Got db pool");

    if title.is_empty() || body.is_empty() {
        return Err(ServerFnError::ServerError(String::from("Cannot create content with empty title.")));
    }

    // TODO: on success redirect to new content
    return Ok(());
}

/// Component to create a new content
#[component]
pub fn CreateContent() -> impl IntoView {
    let create_content = create_server_action::<CreateContent>();
    let create_content_result = create_content.value();
    // check if the server has returned an error
    let has_error = move || create_content_result.with(|val| matches!(val, Some(Err(_))));

    let forum_name_input = create_rw_signal(String::default());
    let is_title_empty = create_rw_signal(true);
    let is_body_empty = create_rw_signal(true);
    let is_content_invalid = create_memo(move |_| { is_title_empty.get() || is_body_empty.get() });

    // TODO: refresh when new forum created?
    let existing_forums = create_blocking_resource( move || (), move |_| get_all_forum_names());

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
                                        <ActionForm action=create_content>
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
                                                <select class="select select-bordered w-full max-w-xs">
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
                                view! { <div>"Error"</div>}.into_view()
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