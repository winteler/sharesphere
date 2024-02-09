use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
use crate::auth::LoginGuardButton;
use crate::icons::{CommentIcon, ErrorIcon, LoadingIcon, MaximizeIcon, MinimizeIcon, MinusIcon, PlusIcon};
use crate::post::{get_post_id_memo};
use crate::score::{get_vote_button_css, CommentVote, DynScoreIndicator, VoteOnComment};
use crate::widget::{AuthorWidget, FormTextEditor, TimeSinceWidget};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: i64,
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
    pub vote: Option<CommentVote>,
    pub child_comments: Vec<CommentWithChildren>,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        #[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow, Ord, PartialOrd, Serialize, Deserialize)]
        pub struct CommentWithVote {
            #[sqlx(flatten)]
            pub comment: Comment,
            pub vote_id: Option<i64>,
            pub vote_creator_id: Option<i64>,
            pub vote_comment_id: Option<i64>,
            pub vote_post_id: Option<i64>,
            pub value: Option<i16>,
            pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
        }

        impl CommentWithVote {
            pub fn into_comment_with_children(self) -> CommentWithChildren {
                let comment_vote = if self.vote_id.is_some() {
                    Some(CommentVote {
                        vote_id: self.vote_id.unwrap(),
                        creator_id: self.vote_creator_id.unwrap(),
                        comment_id: self.vote_comment_id.unwrap(),
                        post_id: self.vote_post_id.unwrap(),
                        value: self.value.unwrap(),
                        timestamp: self.vote_timestamp.unwrap(),
                    })
                } else {
                    None
                };

                CommentWithChildren {
                    comment: self.comment,
                    vote: comment_vote,
                    child_comments: Vec::<CommentWithChildren>::new(),
                }
            }
        }
    }
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

    let comment_with_vote_vec = sqlx::query_as::<_, CommentWithVote>(
        "WITH RECURSIVE comment_tree AS (
            SELECT 1 AS depth,
                   comment_id,
                   ARRAY[(create_timestamp, comment_id)] AS path
            FROM comments
            WHERE
                post_id = $1 AND
                parent_id IS NULL
            UNION ALL
            SELECT r.depth + 1,
                   n.comment_id,
                   r.path || (n.create_timestamp, n.comment_id)
            FROM comment_tree r
            JOIN comments n ON n.parent_id = r.comment_id
        )
        SELECT
            c.*,
            v.vote_id,
            v.creator_id as vote_creator_id,
            v.post_id as vote_post_id,
            v.comment_id as vote_comment_id,
            v.value,
            v.timestamp as vote_timestamp
        FROM comments c
        INNER JOIN comment_tree r ON c.comment_id = r.comment_id
        LEFT JOIN comment_votes v on v.comment_id = c.comment_id
        ORDER BY r.path",
    )
        .bind(post_id)
        .fetch_all(&db_pool)
        .await?;

    let mut comment_tree = Vec::<CommentWithChildren>::new();
    let mut stack = Vec::<CommentWithChildren>::new();
    for comment_with_vote in comment_with_vote_vec {
        let current = comment_with_vote.into_comment_with_children();

        while let Some(top) = stack.last() {
            if current.comment.parent_id.is_some() && current.comment.parent_id.unwrap() == top.comment.comment_id {
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
        user.user_id,
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
                login_button_class="btn btn-circle btn-ghost btn-sm"
                login_button_content=move || view! { <CommentIcon/> }
            >
                <button
                    class="btn btn-circle btn-sm m-1" id="menu-button" aria-expanded="true" aria-haspopup="true"
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
        <div class="flex flex-col h-full">
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
        <div class="flex py-1">
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
            <div class="flex flex-col">
                <div
                    class="text-white pl-2 pt-1"
                    class:hidden=move || !maximize()
                >
                    {comment.comment.body.clone()}
                </div>
                <CommentWidgetBar comment=&comment/>
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
fn CommentWidgetBar<'a>(comment: &'a CommentWithChildren) -> impl IntoView {

    view! {
        <div class="flex gap-2">
            <CommentVotePanel comment=comment/>
            <CommentButton post_id=comment.comment.post_id parent_comment_id=Some(comment.comment.comment_id)/>
            <AuthorWidget author=&comment.comment.creator_name/>
            <TimeSinceWidget timestamp=&comment.comment.create_timestamp/>
        </div>
    }
}

/// Function to react to an comment's upvote or downvote button being clicked.
fn get_on_comment_vote_closure(
    vote: RwSignal<i16>,
    score: RwSignal<i32>,
    comment_id: i64,
    post_id: i64,
    initial_score: i32,
    comment_vote_id: Option<i64>,
    comment_vote_value: Option<i16>,
    vote_action: Action<VoteOnComment, Result<Option<CommentVote>, ServerFnError>>,
    is_upvote: bool,
) -> impl Fn(ev::MouseEvent) {

    move |_| {
        vote.update(|vote| *vote = match *vote {
            1 => if is_upvote { 0 } else { -1 },
            -1 => if is_upvote { 1 } else { 0 },
            _ => if is_upvote { 1 } else { -1 },
        });

        log::info!("Vote value: {}", vote());

        let (current_vote_id, current_vote_value) = if comment_vote_id.is_some() {
            (comment_vote_id, comment_vote_value)
        } else {
            match vote_action.value().get_untracked() {
                Some(Ok(Some(vote))) => (Some(vote.vote_id), Some(vote.value)),
                _ => (None, None),
            }
        };

        vote_action.dispatch(VoteOnComment {
            comment_id,
            post_id,
            vote: vote.get_untracked(),
            previous_vote_id: current_vote_id,
            previous_vote: current_vote_value,
        });
        score.update(|score| *score = initial_score + i32::from(vote.get_untracked()));
    }
}

/// Component to display and modify a comment's score
#[component]
pub fn CommentVotePanel<'a>(
    comment: &'a CommentWithChildren,
) -> impl IntoView {

    let comment_id = comment.comment.comment_id;
    let post_id = comment.comment.post_id;
    let initial_score = comment.comment.score;

    let score = create_rw_signal(comment.comment.score);
    let vote = create_rw_signal(
        match comment.vote.clone() {
            Some(vote) => vote.value,
            None => 0,
        }
    );

    let comment_vote_id = match &comment.vote {
        Some(vote) => Some(vote.vote_id),
        None => None,
    };
    let comment_vote_value = match &comment.vote {
        Some(vote) => Some(vote.value),
        None => None,
    };

    let vote_action = create_server_action::<VoteOnComment>();

    let upvote_button_css = get_vote_button_css(vote, true);
    let downvote_button_css = get_vote_button_css(vote, false);

    view! {
        <div class="flex items-center gap-1">
            <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle btn-sm hover:btn-success"
                login_button_content=move || view! { <PlusIcon/> }
            >
                <button
                    class=upvote_button_css()
                    on:click=get_on_comment_vote_closure(
                        vote,
                        score,
                        comment_id,
                        post_id,
                        initial_score,
                        comment_vote_id,
                        comment_vote_value,
                        vote_action,
                        true)
                >
                    <PlusIcon/>
                </button>
            </LoginGuardButton>
            <DynScoreIndicator score=score/>
            <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle btn-sm hover:btn-error"
                login_button_content=move || view! { <MinusIcon/> }
            >
                <button
                    class=downvote_button_css()
                    on:click=get_on_comment_vote_closure(
                        vote,
                        score,
                        comment_id,
                        post_id,
                        initial_score,
                        comment_vote_id,
                        comment_vote_value,
                        vote_action,
                        false)
                >
                    <MinusIcon/>
                </button>
            </LoginGuardButton>
        </div>
    }
}