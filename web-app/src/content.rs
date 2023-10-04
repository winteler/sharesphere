use cfg_if::cfg_if;
use leptos::*;
use leptos_router::{ActionForm};

use crate::icons::{ErrorIcon};


cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

pub const CREATE_CONTENT_ROUTE : &str = "/content";

#[server(CreateContent, "/api")]
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

    let is_title_empty = create_rw_signal( true);
    let is_body_empty = create_rw_signal( true);
    let is_content_invalid = create_memo(move |_| { is_title_empty.get() || is_body_empty.get() });

    view! {
        <div class="flex flex-col gap-2 w-3/5 max-w-md 2xl:max-w-lg max-2xl:mx-auto">
            <ActionForm action=create_content>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl max-2xl:text-center">"Create [[content]]"</h2>
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