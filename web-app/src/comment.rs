use leptos::*;
use leptos_router::*;

use crate::post::Post;

#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
    is_spoiler: Option<String>,
    is_nsfw: Option<String>
) -> Result<(), ServerFnError> {
    Ok(())
}

/// Component to display a post's author
#[component]
pub fn CreateComment<'a>(_post: &'a Post) -> impl IntoView {
    let create_comment_action = create_server_action::<CreateComment>();
    let create_comment_result = create_comment_action.value();

    let has_error = move || create_comment_result.with(|val| matches!(val, Some(Err(_))));


    view! {
        <div class="flex flex-col gap-2 mx-auto w-1/2 2xl:w-1/3">
            <ActionForm action=create_comment_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Create [[content]]"</h2>
                    <div class="dropdown dropdown-end">
                        <input
                            tabindex="0"
                            type="text"
                            name="forum"
                            placeholder="[[Forum]]"
                            autocomplete="off"
                            class="input input-bordered input-primary w-full h-16"
                            on:input=move |ev| {
                                forum_name_input.update(|name: &mut String| *name = event_target_value(&ev));
                            }
                            prop:value=forum_name_input
                        />
                        <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full">
                            {matching_forum_list}
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
    }
}