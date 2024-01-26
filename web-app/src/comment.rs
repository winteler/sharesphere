use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
use crate::auth::LoginGuardButton;
use crate::icons::{CommentIcon, ErrorIcon, LoadingIcon, MaximizeIcon, MinimizeIcon};
use crate::post::{get_post_id_memo};
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

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentWithChildren {
    pub comment: Comment,
    pub child_comments: Vec<CommentWithChildren>,
}

const DEPTH_TO_COLOR_MAPPING_SIZE: usize = 6;
const DEPTH_TO_COLOR_MAPPING: [&str; DEPTH_TO_COLOR_MAPPING_SIZE] = [
    "bg-blue-500",
    "bg-green-500",
    "bg-yellow-500",
    "bg-orange-500",
    "bg-red-500",
    "bg-violet-500",
];

#[server]
pub async fn get_post_comments(
    post_id: i64,
) -> Result<Vec<Comment>, ServerFnError> {
    let db_pool = get_db_pool()?;

    if post_id < 1 {
        return Err(ServerFnError::ServerError(String::from("Invalid post id.")));
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
pub async fn get_post_comment_tree(
    post_id: i64,
) -> Result<Vec<CommentWithChildren>, ServerFnError> {
    let db_pool = get_db_pool()?;

    if post_id < 1 {
        return Err(ServerFnError::ServerError(String::from("Invalid post id.")));
    }

    let _comment_vec = sqlx::query_as!(
        Comment,
        "WITH RECURSIVE comment_tree AS (
            SELECT 1 AS depth,
                   id,
                   ARRAY[(create_timestamp, id)] AS path
            FROM comments
            WHERE
                post_id = $1 AND
                parent_id IS NULL
            UNION ALL
            SELECT r.depth + 1,
                   n.id,
                   r.path || (n.create_timestamp, n.id)
            FROM comment_tree r
            JOIN comments n ON n.parent_id = r.id
        )
        SELECT c.*
        FROM comments c
        INNER JOIN comment_tree r ON c.id = r.id
        ORDER BY r.path",
        post_id,
    )
        .fetch_all(&db_pool)
        .await?;

    let mut comment_tree = Vec::<CommentWithChildren>::new();
    let mut stack = Vec::<CommentWithChildren>::new();
    for comment in _comment_vec {
        let current = CommentWithChildren {
            comment: comment,
            child_comments: Vec::<CommentWithChildren>::default(),
        };


        while let Some(top) = stack.last() {
            if current.comment.parent_id.is_some() && current.comment.parent_id.unwrap() == top.comment.id {
                // current comment is child of the previous one, keep building stack
                break;
            } else {
                // current comment is not a child of the previous one, previous comment is complete. Add it in its parent.
                let complete_comment = stack.pop().unwrap();
                if let Some(new_top) = stack.last_mut() {
                    // add the complete comment in its parent
                    new_top.child_comments.push(complete_comment);
                }
                else {
                    // root comment, add it in the comment_tree
                    comment_tree.push(complete_comment);
                }
            }
        }

        stack.push(current);
    }

    // add last comments in comment_tree
    while !stack.is_empty() {
        // previous comment is complete. Add it in its parent.
        let complete_comment = stack.pop().unwrap();
        if let Some(new_top) = stack.last_mut() {
            // add the complete comment in its parent
            new_top.child_comments.push(complete_comment);
        }
        else {
            // root comment, add it in the comment_tree
            comment_tree.push(complete_comment);
        }
    }

    Ok(comment_tree)
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
        "INSERT INTO comments (body, parent_id, post_id, creator_id, creator_name) VALUES ($1, $2, $3, $4, $5)",
        comment,
        parent_comment_id,
        post_id,
        user.id,
        user.username,
    )
        .execute(&db_pool)
        .await?;

    Ok(())
}

/// Component to open the comment form
#[component]
pub fn CommentButton(
    post_id: i64,
    #[prop(default = None)]
    parent_comment_id: Option<i64>,
) -> impl IntoView {
    let hide_comment_form = create_rw_signal(true);

    view! {
        <div>
            <LoginGuardButton
                login_button_class="btn btn-circle btn-ghost"
                login_button_content=move || view! { <CommentIcon/> }
            >
                <button
                    class="btn btn-circle m-1" id="menu-button" aria-expanded="true" aria-haspopup="true"
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
                    <CommentForm
                        post_id=post_id
                        parent_comment_id=parent_comment_id
                        on:submit=move |_| hide_comment_form.update(|hide: &mut bool| *hide = true)
                    />
                </div>
            </LoginGuardButton>
        </div>
    }
}

/// Component to publish a comment
#[component]
pub fn CommentForm(
    post_id: i64,
    #[prop(default = None)]
    parent_comment_id: Option<i64>,
) -> impl IntoView {
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
                        value=parent_comment_id
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

/// Comment section component
#[component]
pub fn CommentSection() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let params = use_params_map();
    let post_id = get_post_id_memo(params);
    let comment_vec = create_resource(
        move || (post_id(), state.create_comment_action.version().get()),
        move |(post_id, _)| {
            log::trace!("Load comments for post: {post_id}");
            get_post_comment_tree(post_id)
        });

    view! {
        <div class="flex flex-col h-full gap-4">
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                {
                    move || {
                        comment_vec.with(|result| match result {
                            Some(Ok(comment_vec)) => {
                                let mut comment_ranking = 0;
                                comment_vec.iter().map(|comment| {
                                    comment_ranking = comment_ranking + 1;
                                    view! {
                                        <CommentBox
                                            comment=&comment
                                            ranking=comment_ranking - 1
                                        />
                                    }.into_view()
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


/// Comment box component
#[component]
pub fn CommentBox<'a>(
    comment: &'a CommentWithChildren,
    #[prop(default = 0)]
    depth: usize,
    #[prop(default = 0)]
    ranking: usize,
) -> impl IntoView {

    let maximize = create_rw_signal(true);
    let sidebar_css = move || {
        if maximize() {
            "flex flex-col justify-start items-center"
        } else {
            "flex flex-col justify-center items-center"
        }
    };
    let color_bar_css = format!("{} rounded-full h-full w-1 ", DEPTH_TO_COLOR_MAPPING[(depth + ranking) % DEPTH_TO_COLOR_MAPPING.len()]);

    view! {
        <div class="flex">
            <div class=sidebar_css>
                <label class="btn btn-ghost btn-sm swap swap-rotate w-5">
                    <input
                        type="checkbox"
                        value=maximize
                        on:click=move |_| maximize.update(|value: &mut bool| *value = !*value)
                    />
                    <MinimizeIcon class="swap-on" size=5/>
                    <MaximizeIcon class="swap-off" size=5/>
                </label>
                <div
                    class=color_bar_css
                    class:hidden=move || !maximize()
                />
            </div>
            <div class="flex flex-col py-1">
                <div
                    class="text-white pl-2"
                    class:hidden=move || !maximize()
                >
                    {comment.comment.body.clone()}
                </div>
                <CommentWidgetBar comment=&comment.comment/>
                <div
                    class="flex flex-col"
                    class:hidden=move || !maximize()
                >
                {
                    let mut child_ranking = 0;
                    comment.child_comments.iter().map(move |child_comment| {
                        child_ranking = child_ranking + 1;
                        view! {
                            <CommentBox
                                comment=child_comment
                                depth=depth + 1
                                ranking=ranking + child_ranking - 1
                            />
                        }
                    }).collect_view()
                }
                </div>
            </div>
        </div>
    }
}


/// Component to encapsulate the widgets associated with each comment
#[component]
fn CommentWidgetBar<'a>(comment: &'a Comment) -> impl IntoView {
    view! {
        <div class="flex gap-2">
            <VotePanel score=comment.score/>
            <CommentButton post_id=comment.post_id parent_comment_id=Some(comment.id)/>
            <AuthorWidget author=&comment.creator_name/>
            <TimeSinceWidget timestamp=&comment.create_timestamp/>
        </div>
    }
}