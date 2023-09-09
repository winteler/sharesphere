use leptos::*;
use leptos_router::ActionForm;

pub const CREATE_FORUM_ROUTE : &str = "/forum";

#[server(CreateForum, "/api")]
pub async fn create_forum(cx: Scope, forum_name: String, description: String, is_nsfw: bool) -> Result<(), ServerFnError> {
    Ok(())
}

#[component]
pub fn CreateForum(cx: Scope) -> impl IntoView {
    let create_forum = create_server_action::<CreateForum>(cx);

    view! { cx,
        <h2 class="p-6 text-4xl">"Create forum"</h2>
        <ActionForm action=create_forum>
            <input type="text" name="forum_name" placeholder="Forum name" class="input input-bordered input-primary w-full max-w-xs"/>
            //<input type="text" name="description" placeholder="Description" class="input input-bordered input-secondary w-full max-w-xs"/>
            <textarea class="textarea textarea-secondary" name="description" placeholder="Description"/>
            <div class="form-control">
                <label class="cursor-pointer label">
                    <span class="label-text">"NSFW"</span>
                    <input type="checkbox" name="is_nsfw" class="checkbox checkbox-secondary" />
                </label>
            </div>
        </ActionForm>
    }
}