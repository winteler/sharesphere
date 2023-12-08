use leptos::*;
use leptos_router::*;

use crate::common_components::FormTextEditor;
use crate::icons::ErrorIcon;
use crate::post::Post;

#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
) -> Result<(), ServerFnError> {
    Ok(())
}

/// Component to display a post's author
#[component]
pub fn CreateComment<'a>(post: &'a Post) -> impl IntoView {
    let create_comment_action = create_server_action::<CreateComment>();
    let create_comment_result = create_comment_action.value();

    let has_error = move || create_comment_result.with(|val| matches!(val, Some(Err(_))));
    let is_empty = create_rw_signal(true);

    let post_id = post.id;

    view! {
        <div class="flex flex-col gap-2 w-1/2 2xl:w-1/3">
            <ActionForm action=create_comment_action>
                <div class="flex flex-col gap-2 w-full">
                    <input
                        type="text"
                        name="post_id"
                        class="hidden"
                        value=post_id
                    />
                    <input
                        type="text"
                        name="parent_comment_id"
                        class="hidden"
                    />
                    <FormTextEditor
                        name="comment"
                        placeholder="Comment"
                        on_input=move |ev| {
                            is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                        }
                        unfold_on_focus=true/>
                    /*<textarea
                        name="comment"
                        placeholder="Comment"
                        class="textarea textarea-primary h-textarea_s w-full transition-all ease-in-out focus:h-textarea_m"
                        on:input=move |ev| {
                            is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                        }
                    />*/
                    <button type="submit" class="btn btn-active btn-secondary" disabled=is_empty>"Publish"</button>

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