use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
use crate::common_components::FormTextEditor;
use crate::icons::{ErrorIcon, LoadingIcon};
use crate::post::{Post};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub body: String,
    pub is_edited: bool,
    pub moderated_body: Option<String>,
    pub parent_id: Option<i64>,
    pub post_id: i64,
    pub creator_id: i64,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: i32,
    pub trending_score: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[server]
pub async fn get_post_comments(
    post_id: i64,
) -> Result<Vec<Comment>, ServerFnError> {
    let db_pool = get_db_pool()?;

    if post_id < 1 {
        return Err(ServerFnError::ServerError(String::from("Cannot create empty comment.")));
    }

    let comment_vec = sqlx::query_as!(
        Comment,
        "select * from comments where post_id = $1",
        post_id,
    )
        .fetch_all(&db_pool)
        .await?;

    Ok(comment_vec)
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

/// Comment section component
#[component]
pub fn CommentSection<'a>(post: &'a Post) -> impl IntoView {
    view! {
        <PublishComment post=post/>
        <CommentTree post=post/>
    }
}

/// Component to publish a comment
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

/// Comment tree component
#[component]
pub fn CommentTree<'a>(post: &'a Post) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let post_id = post.id;
    let comment_vec = create_resource(move || (state.create_comment_action.version().get()), move |_| get_post_comments(post_id));

    view! {
        <div class="flex flex-col h-full">
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                {
                    move || {
                         comment_vec.with(|result| match result {
                            Some(Ok(comment_vec)) => {
                                comment_vec.iter().map(|comment| {
                                    view! { <CommentBox comment=comment/> }.into_view()
                                }).collect_view()
                            },
                            Some(Err(e)) => {
                                log::info!("Error: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            },
                            None => {
                                log::trace!("Resource not loaded yet.");
                                view! { <ErrorIcon/> }.into_view()
                            },
                        })
                    }
                }
            </Transition>
        </div>
    }
}

/// Comment box component
#[component]
pub fn CommentBox<'a>(comment: &'a Comment) -> impl IntoView {
    view! {
        <div>
            {comment.body.clone()}
        </div>
    }
}