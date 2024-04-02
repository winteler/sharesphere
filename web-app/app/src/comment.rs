use std::fmt;

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};
use crate::app::GlobalState;
use crate::auth::LoginGuardButton;
use crate::icons::{CommentIcon, ErrorIcon, LoadingIcon, MaximizeIcon, MinimizeIcon};
use crate::post::get_post_id_memo;
use crate::ranking::{ContentWithVote, SortType, Vote, VotePanel};
#[cfg(feature = "ssr")]
use crate::widget::{get_styled_html_from_markdown};
use crate::widget::{AuthorWidget, FormMarkdownEditor, TimeSinceWidget};

const DEPTH_TO_COLOR_MAPPING_SIZE: usize = 6;
const DEPTH_TO_COLOR_MAPPING: [&str; DEPTH_TO_COLOR_MAPPING_SIZE] = [
    "bg-blue-500",
    "bg-green-500",
    "bg-yellow-500",
    "bg-orange-500",
    "bg-red-500",
    "bg-violet-500",
];

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: i64,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_edited: bool,
    pub moderated_body: Option<String>,
    pub parent_id: Option<i64>,
    pub post_id: i64,
    pub creator_id: i64,
    pub creator_name: String,
    pub score: i32,
    pub score_minus: i32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentWithChildren {
    pub comment: Comment,
    pub vote: Option<Vote>,
    pub child_comments: Vec<CommentWithChildren>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CommentSortType {
    Best,
    Recent,
}

impl fmt::Display for CommentSortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sort_type_name = match self {
            CommentSortType::Best => "Best",
            CommentSortType::Recent => "Recent",
        };
        write!(f, "{sort_type_name}")
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use crate::auth::User;
    use crate::ranking::{Vote, VoteValue};
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow, Ord, PartialOrd, Serialize, Deserialize)]
    pub struct CommentWithVote {
        #[sqlx(flatten)]
        pub comment: Comment,
        pub vote_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub vote_comment_id: Option<Option<i64>>,
        pub vote_user_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl CommentWithVote {
        pub fn into_comment_with_children(self) -> CommentWithChildren {
            let comment_vote = if self.vote_id.is_some() {
                Some(Vote {
                    vote_id: self.vote_id.unwrap(),
                    user_id: self.vote_user_id.unwrap(),
                    comment_id: self.vote_comment_id.unwrap(),
                    post_id: self.vote_post_id.unwrap(),
                    value: VoteValue::from(self.value.unwrap()),
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

    impl CommentSortType {
        pub fn to_order_by_code(self) -> &'static str {
            match self {
                CommentSortType::Best => "score",
                CommentSortType::Recent => "create_timestamp",
            }
        }
    }

    pub async fn create_comment(
        post_id: i64,
        parent_comment_id: Option<i64>,
        comment: &str,
        markdown_comment: Option<&str>,
        user: &User,
        db_pool: PgPool,
    ) -> Result<Comment, ServerFnError> {
        if comment.is_empty() {
            return Err(ServerFnError::new("Cannot create empty comment."));
        }

        let comment = sqlx::query_as!(
            Comment,
            "INSERT INTO comments (body, markdown_body, parent_id, post_id, creator_id, creator_name) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            comment,
            markdown_comment,
            parent_comment_id,
            post_id,
            user.user_id,
            user.username,
        )
            .fetch_one(&db_pool)
            .await?;

        Ok(comment)
    }

    pub async fn get_post_comment_tree(
        post_id: i64,
        sort_type: SortType,
        user_id: Option<i64>,
        db_pool: PgPool,
    ) -> Result<Vec<CommentWithChildren>, ServerFnError> {
        if post_id < 1 {
            return Err(ServerFnError::new("Invalid post id."));
        }

        let sort_column = sort_type.to_order_by_code();

        let comment_with_vote_vec = sqlx::query_as::<_, CommentWithVote>(
            format!("WITH RECURSIVE comment_tree AS (
            SELECT c.*,
                   v.vote_id,
                   v.user_id as vote_user_id,
                   v.post_id as vote_post_id,
                   v.comment_id as vote_comment_id,
                   v.value,
                   v.timestamp as vote_timestamp,
                   1 AS depth,
                   ARRAY[(c.{sort_column}, c.comment_id)] AS path
            FROM comments c
            LEFT JOIN votes v
            ON v.comment_id = c.comment_id AND
               v.user_id = $1
            WHERE
                c.post_id = $2 AND
                c.parent_id IS NULL
            UNION ALL
            SELECT n.*,
                   vr.vote_id,
                   vr.user_id as vote_user_id,
                   vr.post_id as vote_post_id,
                   vr.comment_id as vote_comment_id,
                   vr.value,
                   vr.timestamp as vote_timestamp,
                   r.depth + 1,
                   r.path || (n.{sort_column}, n.comment_id)
            FROM comment_tree r
            JOIN comments n ON n.parent_id = r.comment_id
            LEFT JOIN votes vr
            ON vr.comment_id = n.comment_id AND
               vr.user_id = $1
        )
        SELECT * FROM comment_tree
        ORDER BY path DESC").as_str(),
        )
            .bind(user_id)
            .bind(post_id)
            .fetch_all(&db_pool)
            .await?;

        let mut comment_tree = Vec::<CommentWithChildren>::new();
        let mut stack = Vec::<(i64, Vec::<CommentWithChildren>)>::new();
        for comment_with_vote in comment_with_vote_vec {
            let mut current = comment_with_vote.into_comment_with_children();

            if let Some((top_parent_id, child_comments)) = stack.last_mut() {
                if *top_parent_id == current.comment.comment_id {
                    // child comments at the top of the stack belong to the current comment, add them
                    current.child_comments.append(child_comments);
                    stack.pop();
                }
            }

            // if the current element has a parent, add it to the stack. Otherwise, add it to the comment tree as a root element.
            if let Some(parent_id) = current.comment.parent_id {
                if let Some((top_parent_id, top_child_comments)) = stack.last_mut() {
                    if parent_id == *top_parent_id {
                        // same parent id as the top of the stack, add it
                        top_child_comments.push(current);
                    } else {
                        // different parent id as the top of the stack, add it as a new element on the stack
                        stack.push((parent_id, Vec::from([current])));
                    }
                } else {
                    // no element on the stack, add the current comment as a new element
                    stack.push((parent_id, Vec::from([current])));
                }
            } else {
                comment_tree.push(current);
            }
        }

        Ok(comment_tree)
    }
}

#[server]
pub async fn get_post_comment_tree(
    post_id: i64,
    sort_type: SortType,
) -> Result<Vec<CommentWithChildren>, ServerFnError> {
    let user_id = match get_user().await {
        Ok(user) => Some(user.user_id),
        Err(_) => None
    };
    let db_pool = get_db_pool()?;

    let comment_tree = ssr::get_post_comment_tree(post_id, sort_type, user_id, db_pool).await?;

    Ok(comment_tree)
}

#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
    is_markdown: Option<String>,
) -> Result<(), ServerFnError> {
    log::trace!("Create comment for post {post_id}");
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = match is_markdown {
        Some(_) => (get_styled_html_from_markdown(comment.clone()).await?, Some(comment.as_str())),
        None => (comment, None),
    };

    ssr::create_comment(
        post_id,
        parent_comment_id,
        comment.as_str(),
        markdown_comment,
        &user,
        db_pool,
    ).await?;

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
                    class=("btn-primary", move || !hide_comment_form())
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

    let form_ref = create_node_ref::<html::Form>();

    let response_ok = move || state.create_comment_action.value().with(|val| matches!(val, Some(Ok(_))));

    create_effect(move |_| {
        if response_ok() {
            form_ref.get().expect("form node should be loaded").reset()
        }
    });

    view! {
        <div class="flex flex-col gap-2">
            <ActionForm
                action=state.create_comment_action
                node_ref=form_ref
            >
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
                    <FormMarkdownEditor
                        name="comment"
                        is_markdown_name="is_markdown"
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
        move || (post_id(), state.create_comment_action.version().get(), state.comment_sort_type.get()),
        move |(post_id, _, sort_type)| {
            log::debug!("Load comments for post: {post_id} sorting by {sort_type}");
            get_post_comment_tree(post_id, sort_type)
        });

    view! {
        <div class="flex flex-col h-full">
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                {
                    move || {
                        comment_vec.map(|result| match result {
                            Ok(comment_vec) => {
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
                            Err(e) => {
                                log::info!("Error: {}", e);
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

    let is_markdown = comment.comment.markdown_body.is_some();
    let comment_class = move || match (maximize(), is_markdown) {
        (true, true) => "pl-2 pt-1",
        (true, false) => "pl-2 pt-1 whitespace-pre",
        (false, _) => "hidden",
    };

    view! {
        <div class="flex pl-2 py-1">
            <div class=sidebar_css>
                <label class="btn btn-ghost btn-sm swap swap-rotate w-5">
                    <input
                        type="checkbox"
                        value=maximize
                        on:click=move |_| maximize.update(|value: &mut bool| *value = !*value)
                    />
                    <MinimizeIcon class="h-5 w-5 swap-on"/>
                    <MaximizeIcon class="h-5 w-5 swap-off"/>
                </label>
                <div
                    class=color_bar_css
                    class:hidden=move || !maximize()
                />
            </div>
            <div class="flex flex-col">
                <div
                    class=comment_class
                    inner_html={comment.comment.body.clone()}
                />
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

    let content = ContentWithVote::Comment(&comment.comment, &comment.vote);

    view! {
        <div class="flex gap-2">
            <VotePanel content=content/>
            <CommentButton post_id=comment.comment.post_id parent_comment_id=Some(comment.comment.comment_id)/>
            <AuthorWidget author=&comment.comment.creator_name/>
            <TimeSinceWidget timestamp=&comment.comment.create_timestamp/>
        </div>
    }
}