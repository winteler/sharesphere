use cfg_if::cfg_if;
use leptos::*;
use leptos_router::ActionForm;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

pub const CREATE_FORUM_ROUTE : &str = "/forum";

#[server(CreateForum, "/api")]
pub async fn create_forum(cx: Scope, name: String, description: String, is_nsfw: bool) -> Result<(), ServerFnError> {
    let user = get_user(cx).await?;

    let db_pool = get_db_pool(cx)?;
    let result = match sqlx::query(
        "INSERT INTO forums (name, description, nsfw, user_id) VALUES (?, ?, ?, ?)",
    )
        .bind(name)
        .bind(description)
        .bind (is_nsfw)
        .bind(user.id)
        .execute(&db_pool)
        .await
    {
        Ok(_row) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    };

    // TODO: on success redirect to new forum
    return result;
}

#[component]
pub fn CreateForum(cx: Scope) -> impl IntoView {
    let create_forum = create_server_action::<CreateForum>(cx);

    view! { cx,
        <h2 class="p-6 text-4xl">"Create forum"</h2>
        <ActionForm action=create_forum>
            <div class="flex flex-col gap-1">
                <input type="text" name="name" placeholder="Forum name" class="input input-bordered input-primary w-full max-w-xs"/>
                //<input type="text" name="description" placeholder="Description" class="input input-bordered input-secondary w-full max-w-xs"/>
                <textarea class="textarea textarea-primary" name="description" placeholder="Description"/>
                <div class="form-control">
                    <label class="cursor-pointer label">
                        <span class="label-text label-primary">"NSFW content"</span>
                        <input type="checkbox" name="is_nsfw" class="checkbox checkbox-primary" />
                    </label>
                </div>
            </div>
        </ActionForm>
    }
}