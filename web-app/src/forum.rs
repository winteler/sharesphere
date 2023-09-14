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
pub async fn create_forum(cx: Scope, name: String, description: String, is_nsfw: Option<String>) -> Result<(), ServerFnError> {
    log!("Create [[forum]] '{name}', {description}, {:?}", is_nsfw);
    let user = get_user(cx).await?;
    log!("Could get user: {:?}", user);

    let db_pool = get_db_pool(cx)?;
    log!("Got db pool");
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

#[component]
pub fn CreateForum(cx: Scope) -> impl IntoView {
    let create_forum = create_server_action::<CreateForum>(cx);

    view! { cx,
        <ActionForm action=create_forum>
            <div class="flex flex-col gap-1 w-full max-w-md 2xl:max-w-lg max-2xl:mx-auto">
                <h2 class="p-6 text-4xl max-2xl:text-center">"Create [[forum]]"</h2>
                <input type="text" name="name" placeholder="[[Forum]] name" class="input input-bordered input-primary"/>
                <textarea name="description" placeholder="Description" class="textarea textarea-primary h-40"/>
                <div class="form-control">
                    <label class="cursor-pointer label">
                        <span class="label-text">"NSFW content"</span>
                        <input type="checkbox" name="is_nsfw" class="checkbox checkbox-primary"/>
                    </label>
                </div>
                <button type="submit" class="btn btn-active btn-secondary">"Create"</button>
            </div>
        </ActionForm>
    }
}

#[component]
pub fn ForumBanner(cx: Scope) -> impl IntoView {

    // TODO: add forum banner
    view! { cx,
        <h2 class="p-6 text-4xl max-2xl:text-center">"[[forum banner]]"</h2>
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
        <h2 class="p-6 text-4xl max-2xl:text-center">"Create [[content]]"</h2>
    }
}