use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;

use crate::app::GlobalState;
use crate::common_components::FormTextEditor;
use crate::icons::ErrorIcon;
use crate::post::Post;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}


#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
) -> Result<(), ServerFnError> {
    log::info!("Create comment for post {post_id}");
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if comment.is_empty() {
        return Err(ServerFnError::ServerError(String::from("Cannot create empty comment.")));
    }

    sqlx::query!(
        "INSERT INTO comments (body, parent_id, post_id, creator_id) VALUES ($1, $2, $3, $4)",
        comment,
        parent_comment_id,
        post_id,
        user.id,
    )
        .execute(&db_pool)
        .await?;

    Ok(())
}

/// Component to display a post's author
#[component]
pub fn PublishComment<'a>(post: &'a Post) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let create_comment_result = state.create_comment_action.value();

    let has_error = move || create_comment_result.with(|val| matches!(val, Some(Err(_))));
    let is_empty = create_rw_signal(true);

    let post_id = post.id;

    view! {
        <div class="flex flex-col gap-2 w-1/2 2xl:w-1/3">
            <ActionForm action=state.create_comment_action>
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
                        on:input=move |ev| {
                            is_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                        }
                        unfold_on_focus=true
                    />
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