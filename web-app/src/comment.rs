use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
use crate::auth::LoginGuardButton;
use crate::icons::{CommentIcon, ErrorIcon, LoadingIcon};
use crate::score::{VotePanel};
use crate::widget::{AuthorWidget, TimeSinceWidget, FormTextEditor};

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
    pub creator_name: String,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: i32,
    pub trending_score: i32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
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
pub fn CommentSection(post_id: i64) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let comment_vec = create_resource(move || state.create_comment_action.version().get(), move |_| get_post_comments(post_id));

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
                                view! { <LoadingIcon/> }.into_view()
                            },
                        })
                    }
                }
            </Transition>
        </div>
    }
}

/// Comment section component
#[component]
pub fn CommentButton(post_id: i64) -> impl IntoView {
    let hide_comment_form = create_rw_signal(true);

    view! {
        <div>
            <LoginGuardButton
                login_button_class="btn btn-ghost"
                login_button_content=move || view! { <CommentIcon/> }
            >
                <button
                    class="btn m-1" id="menu-button" aria-expanded="true" aria-haspopup="true"
                    class=("btn-accent", move || !hide_comment_form())
                    class=("btn-ghost", move || hide_comment_form())
                    on:click=move |_| hide_comment_form.update(|hide: &mut bool| *hide = !*hide)
                >
                    <CommentIcon/>
                </button>
                <div
                    class="absolute float-left z-10 w-1/2 2xl:w-1/3 origin-top-right" role="menu" aria-orientation="vertical" aria-labelledby="menu-button" tabindex="-1"
                    class:hidden=hide_comment_form
                >
                    <CommentForm post_id=post_id on:submit=move |_| hide_comment_form.update(|hide: &mut bool| *hide = true)/>
                </div>
            </LoginGuardButton>
        </div>
    }
}

/// Component to publish a comment
#[component]
pub fn CommentForm(post_id: i64) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let create_comment_result = state.create_comment_action.value();
    let has_error = move || create_comment_result.with(|val| matches!(val, Some(Err(_))));

    view! {
        <div class="flex flex-col gap-2">
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
                        placeholder="Your comment..."
                        with_publish_button=true
                    />
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

/// Comment box component
#[component]
pub fn CommentBox<'a>(comment: &'a Comment) -> impl IntoView {
    view! {
        <div>
            {comment.body.clone()}
        </div>
        <div class="card">
            <div class="card-body">
                <div class="flex flex-col gap-4">
                    {comment.body.clone()}
                    <div class="flex gap-2">
                        <VotePanel score=comment.score/>
                        // TODO, pass comment id as optional prop
                        <CommentButton post_id=comment.post_id/>
                        <AuthorWidget author=&comment.creator_name/>
                        <TimeSinceWidget timestamp=&comment.create_timestamp/>
                    </div>
                </div>
            </div>
        </div>
    }
}