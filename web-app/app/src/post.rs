use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::auth::LoginGuardButton;
use crate::comment::{CommentButton, CommentSection};
use crate::forum::{get_all_forum_names};
use crate::icons::{ErrorIcon, LoadingIcon, MinusIcon, PlusIcon};
use crate::score::{get_vote_button_css, update_vote_value, DynScoreIndicator, PostVote, VoteOnPost};
use crate::widget::{AuthorWidget, FormTextEditor, TimeSinceWidget};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};
#[cfg(feature = "ssr")]
use crate::forum::FORUM_ROUTE_PREFIX;

pub const CREATE_POST_SUFFIX : &str = "/content";
pub const CREATE_POST_ROUTE : &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);
pub const CREATE_POST_FORUM_QUERY_PARAM : &str = "forum";
pub const POST_ROUTE_PREFIX : &str = "/posts";
pub const POST_ROUTE_PARAM_NAME : &str = "post_name";
pub const POST_ROUTE : &str = concatcp!(POST_ROUTE_PREFIX, PARAM_ROUTE_PREFIX, POST_ROUTE_PARAM_NAME);

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Post {
    pub post_id: i64,
    pub title: String,
    pub body: String,
    pub is_meta_post: bool,
    pub is_nsfw: bool,
    pub spoiler_level: i32,
    pub tags: Option<String>,
    pub is_edited: bool,
    pub moderated_body: Option<String>,
    pub meta_post_id: Option<i64>,
    pub forum_id: i64,
    pub creator_id: i64,
    pub creator_name: String,
    pub num_comments: i32,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: i32,
    pub trending_score: i32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub scoring_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PostWithVote {
    pub post: Post,
    pub vote: Option<PostVote>
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow, Ord, PartialOrd, Serialize, Deserialize)]
    pub struct PostJoinVote {
        #[sqlx(flatten)]
        pub post: super::Post,
        pub vote_id: Option<i64>,
        pub vote_creator_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl PostJoinVote {
        pub fn into_post_with_vote(self) -> PostWithVote {
            let post_vote = if self.vote_id.is_some() {
                Some(PostVote {
                    vote_id: self.vote_id.unwrap(),
                    creator_id: self.vote_creator_id.unwrap(),
                    post_id: self.vote_post_id.unwrap(),
                    value: self.value.unwrap(),
                    timestamp: self.vote_timestamp.unwrap(),
                })
            } else {
                None
            };

            PostWithVote {
                post: self.post,
                vote: post_vote,
            }
        }
    }

    pub async fn get_post_to_rank_vec() -> Result<Vec<Post>, ServerFnError> {
        let db_pool = get_db_pool()?;
        Ok(sqlx::query_as!(
            Post,
            "SELECT * FROM posts \
            WHERE create_timestamp > (CURRENT_TIMESTAMP - INTERVAL '2 days')",
        )
            .fetch_all(&db_pool)
            .await?)
    }

    pub async fn update_post_scores() -> Result<(), ServerFnError> {
        let db_pool = get_db_pool()?;
        sqlx::query!(
            "UPDATE posts \
            SET scoring_timestamp = CURRENT_TIMESTAMP \
            WHERE create_timestamp > (CURRENT_TIMESTAMP - INTERVAL '2 days')",
        )
            .execute(&db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn get_post_with_vote_by_id(post_id: i64) -> Result<PostWithVote, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user_id = match get_user().await {
        Ok(user) => Some(user.user_id),
        Err(_) => None
    };

    let post_join_vote = sqlx::query_as::<_, ssr::PostJoinVote>(
        "SELECT p.*,
                v.vote_id,
                v.creator_id as vote_creator_id,
                v.post_id as vote_post_id,
                v.value,
                v.timestamp as vote_timestamp
        FROM posts p
        LEFT JOIN post_votes v
        ON v.post_id = p.post_id AND
           v.creator_id = $1
        WHERE p.post_id = $2",
    )
        .bind(user_id)
        .bind(post_id)
        .fetch_one(&db_pool)
        .await?;

    Ok(post_join_vote.into_post_with_vote())
}

#[server]
pub async fn get_post_vec_by_forum_name(forum_name: String) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let post_vec = sqlx::query_as!(
        Post,
        "SELECT posts.* FROM posts \
        join forums on forums.forum_id = posts.forum_id \
        WHERE forums.forum_name = $1",
        forum_name
    )
        .fetch_all(&db_pool)
        .await?;

    Ok(post_vec)
}

#[server]
pub async fn create_post(forum: String, title: String, body: String, is_nsfw: Option<String>, tag: Option<String>) -> Result<(), ServerFnError> {
    log::info!("Create [[content]] '{title}'");
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if forum.is_empty() || title.is_empty() {
        return Err(ServerFnError::new("Cannot create content without a valid forum and title."));
    }

    let new_post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (title, body, is_nsfw, tags, forum_id, creator_id, creator_name)
         VALUES (
            $1, $2, $3, $4,
            (SELECT forum_id FROM forums WHERE forum_name = $5),
            $6, $7
        ) RETURNING *",
        title.clone(),
        body,
        is_nsfw.is_some(),
        tag.unwrap_or_default(),
        forum.clone(),
        user.user_id,
        user.username,
    )
        .fetch_one(&db_pool)
        .await?;

    log::info!("New post id: {}", new_post.post_id);
    let new_post_path : &str = &(FORUM_ROUTE_PREFIX.to_owned() + "/" + forum.as_str() + POST_ROUTE_PREFIX + "/" + new_post.post_id.to_string().as_ref());
    leptos_axum::redirect(new_post_path);
    Ok(())
}

/// Get a memo returning the last valid post id from the url. Used to avoid triggering resources when leaving pages
pub fn get_post_id_memo(params: Memo<ParamsMap>) -> Memo<i64> {
    create_memo(move |current_post_id: Option<&i64>| {
        if let Some(new_post_id_string) = params.with(|params| params.get(POST_ROUTE_PARAM_NAME).cloned()) {
            if let Ok(new_post_id) = new_post_id_string.parse::<i64>() {
                log::trace!("Current post id: {:?}, new post id: {new_post_id}", current_post_id);
                new_post_id
            }
            else {
                log::trace!("Could not parse new post id: {new_post_id_string}, reuse current post id: {:?}", current_post_id);
                current_post_id.cloned().unwrap_or_default()
            }
        }
        else {
            log::trace!("Could not find new post id, reuse current post id: {:?}", current_post_id);
            current_post_id.cloned().unwrap_or_default()
        }
    })
}

/// Function to react to an post's upvote or downvote button being clicked.
fn get_on_post_vote_closure(
    vote: RwSignal<i16>,
    score: RwSignal<i32>,
    post_id: i64,
    initial_score: i32,
    post_vote_id: Option<i64>,
    post_vote_value: Option<i16>,
    vote_action: Action<VoteOnPost, Result<Option<PostVote>, ServerFnError>>,
    is_upvote: bool,
) -> impl Fn(ev::MouseEvent) {

    move |_| {
        vote.update(|vote| update_vote_value(vote, is_upvote));

        log::info!("Post vote value {}", vote.get_untracked());

        let (current_vote_id, current_vote_value) = match vote_action.value().get_untracked() {
            Some(Ok(Some(vote))) => (Some(vote.vote_id), Some(vote.value)),
            _ => {
                if post_vote_id.is_some() {
                    (post_vote_id, post_vote_value)
                } else {
                    (None, None)
                }
            }
        };

        vote_action.dispatch(VoteOnPost {
            post_id,
            vote: vote.get_untracked(),
            previous_vote_id: current_vote_id,
            previous_vote: current_vote_value,
        });
        score.update(|score| *score = initial_score + i32::from(vote.get_untracked()));
    }
}

/// Component to create a new content
#[component]
pub fn CreatePost() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let create_post_result = state.create_post_action.value();
    // check if the server has returned an error
    let has_error = move || create_post_result.with(|val| matches!(val, Some(Err(_))));

    let query = use_query_map();
    let forum_query = move || query.with_untracked(|query| query.get(CREATE_POST_FORUM_QUERY_PARAM).unwrap_or(&String::default()).to_string());

    let forum_name_input = create_rw_signal(forum_query());
    let is_title_empty = create_rw_signal(true);
    let is_body_empty = create_rw_signal(true);
    let is_content_invalid = create_memo(move |_| { is_title_empty.get() || is_body_empty.get() });

    let existing_forums = create_resource(
        move || state.create_forum_action.version().get(),
        move |_| get_all_forum_names());

    view! {
        <Transition fallback=move || (view! { <LoadingIcon/> })>
            {
                move || {
                    existing_forums.with(|result| {
                        match result {
                            Some(Ok(forum_set)) => {
                                log::info!("Forum name set: {:?}", forum_set);

                                let matching_forum_list = forum_set
                                    .iter()
                                    .filter(|forum| forum.starts_with(forum_name_input.get().as_str())).map(|forum_name| {
                                        view! {
                                            <li>
                                                <button value=forum_name on:click=move |ev| forum_name_input.update(|name| *name = event_target_value(&ev))>
                                                    {forum_name}
                                                </button>
                                            </li>
                                        }
                                    }).collect_view();

                                view! {
                                    <div class="flex flex-col gap-2 mx-auto w-1/2 2xl:w-1/3">
                                        <ActionForm action=state.create_post_action>
                                            <div class="flex flex-col gap-2 w-full">
                                                <h2 class="py-4 text-4xl text-center">"Create [[content]]"</h2>
                                                <div class="dropdown dropdown-end">
                                                    <input
                                                        tabindex="0"
                                                        type="text"
                                                        name="forum"
                                                        placeholder="[[Forum]]"
                                                        autocomplete="off"
                                                        class="input input-bordered input-primary w-full h-input_m"
                                                        on:input=move |ev| {
                                                            forum_name_input.update(|name: &mut String| *name = event_target_value(&ev));
                                                        }
                                                        prop:value=forum_name_input
                                                    />
                                                    <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full">
                                                        {matching_forum_list}
                                                    </ul>
                                                </div>
                                                <input
                                                    type="text"
                                                    name="title"
                                                    placeholder="Title"
                                                    class="input input-bordered input-primary h-input_m"
                                                    on:input=move |ev| {
                                                        is_title_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                                                    }
                                                />
                                                <FormTextEditor
                                                    name="body"
                                                    placeholder="Content"
                                                    on:input=move |ev| {
                                                        is_body_empty.update(|is_empty: &mut bool| *is_empty = event_target_value(&ev).is_empty());
                                                    }
                                                />

                                                <select name="tag" class="select select-bordered w-full max-w-xs">
                                                    <option disabled selected>"Tag"</option>
                                                    <option>"This should be"</option>
                                                    <option>"Customized"</option>
                                                </select>
                                                <button type="submit" class="btn btn-active btn-secondary" disabled=is_content_invalid>"Create"</button>
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
                                }.into_view()
                            }
                            Some(Err(e)) => {
                                log::info!("Error while getting forum names: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            },
                            None => {
                                log::trace!("Resource not loaded yet.");
                                view! { <LoadingIcon/> }.into_view()
                            }
                        }
                    })
                }
            }
        </Transition>
    }
}

/// Component to display a content
#[component]
pub fn Post() -> impl IntoView {
    let params = use_params_map();
    let post_id = get_post_id_memo(params);

    // TODO: create PostDetail struct with additional info, like vote of user. Load this here instead of normal post
    let post = create_resource(
        move || post_id(),
        move |post_id| {
            log::trace!("Load data for post: {post_id}");
            get_post_with_vote_by_id(post_id)
        });

    view! {
        <div class="flex flex-col content-start gap-1">
            <Suspense fallback=move || view! { <LoadingIcon/> }>
                {
                    post.with(|result| {
                        match result {
                            Some(Ok(post)) => {
                                view! {
                                    <div class="card">
                                        <div class="card-body">
                                            <div class="flex flex-col gap-4">
                                                <h2 class="card-title text-white">{post.post.title.clone()}</h2>
                                                <div class="text-white">{post.post.body.clone()}</div>
                                                <PostWidgetBar post=post/>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            },
                            Some(Err(e)) => {
                                log::info!("Error while getting forum names: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            },
                            None => {
                                log::trace!("Resource not loaded yet.");
                                view! { <LoadingIcon/> }.into_view()
                            }
                        }
                    })
                }
            </Suspense>
            <CommentSection/>
        </div>
    }
}

/// Component to encapsulate the widgets associated with each post
#[component]
fn PostWidgetBar<'a>(post: &'a PostWithVote) -> impl IntoView {
    view! {
        <div class="flex gap-2">
            <PostVotePanel
                post=post
            />
            <CommentButton post_id=post.post.post_id/>
            <AuthorWidget author=&post.post.creator_name/>
            <TimeSinceWidget timestamp=&post.post.create_timestamp/>
        </div>
    }
}

/// Component to display and modify a post's score
#[component]
pub fn PostVotePanel<'a>(
    post: &'a PostWithVote,
) -> impl IntoView {

    let post_id = post.post.post_id;
    let (vote, initial_score ) = match &post.vote {
        Some(vote) => (vote.value, post.post.score - i32::from(vote.value)),
        None => (0, post.post.score),
    };

    let score = create_rw_signal(post.post.score);
    let vote = create_rw_signal(vote);

    let comment_vote_id = match &post.vote {
        Some(vote) => Some(vote.vote_id),
        None => None,
    };
    let comment_vote_value = match &post.vote {
        Some(vote) => Some(vote.value),
        None => None,
    };

    let vote_action = create_server_action::<VoteOnPost>();

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
                    on:click=get_on_post_vote_closure(
                        vote,
                        score,
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
                    on:click=get_on_post_vote_closure(
                        vote,
                        score,
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