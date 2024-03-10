use std::fmt;

use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};
use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::comment::{CommentButton, CommentSection};
#[cfg(feature = "ssr")]
use crate::forum::FORUM_ROUTE_PREFIX;
use crate::forum::get_all_forum_names;
use crate::icons::{ErrorIcon, LoadingIcon};
use crate::ranking::{ContentWithVote, SortType, Vote, VotePanel};
use crate::widget::{AuthorWidget, CommentSortWidget, FormTextEditor, TimeSinceWidget};

pub const CREATE_POST_SUFFIX : &str = "/content";
pub const CREATE_POST_ROUTE : &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);
pub const CREATE_POST_FORUM_QUERY_PARAM : &str = "forum";
pub const POST_ROUTE_PREFIX : &str = "/posts";
pub const POST_ROUTE_PARAM_NAME : &str = "post_name";
pub const POST_ROUTE : &str = concatcp!(POST_ROUTE_PREFIX, PARAM_ROUTE_PREFIX, POST_ROUTE_PARAM_NAME);

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
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
    pub forum_name: String,
    pub creator_id: i64,
    pub creator_name: String,
    pub num_comments: i32,
    pub score: i32,
    pub score_minus: i32,
    pub recommended_score: f32,
    pub trending_score: f32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub scoring_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PostWithVote {
    pub post: Post,
    pub vote: Option<Vote>
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PostSortType {
    Hot,
    Trending,
    Best,
    Recent,
}

impl fmt::Display for PostSortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sort_type_name = match self {
            PostSortType::Hot => "Hot",
            PostSortType::Trending => "Trending",
            PostSortType::Best => "Best",
            PostSortType::Recent => "Recent",
        };
        write!(f, "{sort_type_name}")
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::ranking::{Vote, VoteValue};
    use super::*;

    #[derive(Clone, Debug, PartialEq, sqlx::FromRow, PartialOrd, Serialize, Deserialize)]
    pub struct PostJoinVote {
        #[sqlx(flatten)]
        pub post: super::Post,
        pub vote_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub vote_comment_id: Option<Option<i64>>,
        pub vote_creator_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl PostJoinVote {
        pub fn into_post_with_vote(self) -> PostWithVote {
            let post_vote = if self.vote_id.is_some() {
                Some(Vote {
                    vote_id: self.vote_id.unwrap(),
                    post_id: self.vote_post_id.unwrap(),
                    comment_id: None,
                    creator_id: self.vote_creator_id.unwrap(),
                    value: VoteValue::from(self.value.unwrap()),
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

    impl PostSortType {
        pub fn to_order_by_code(self) -> &'static str {
            match self {
                PostSortType::Hot => "recommended_score",
                PostSortType::Trending => "trending_score",
                PostSortType::Best => "score",
                PostSortType::Recent => "create_timestamp",
            }
        }
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
pub async fn get_sorted_post_vec(sort_type: SortType) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let post_vec = sqlx::query_as::<_, Post>(
        format!("SELECT * FROM posts \
        ORDER BY {} DESC", sort_type.to_order_by_code()).as_str()
    )
        .fetch_all(&db_pool)
        .await?;

    Ok(post_vec)
}

#[server]
pub async fn get_subscribed_post_vec(user_id: i64, sort_type: SortType) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let post_vec = sqlx::query_as::<_, Post>(
        format!("SELECT p.* FROM posts p \
        JOIN forums f on f.forum_id = p.forum_id \
        WHERE f.forum_id IN ( \
            SELECT forum_id FROM forum_subscriptions WHERE user_id = $1 \
        ) \
        ORDER BY {} DESC", sort_type.to_order_by_code()).as_str()
    )
        .bind(user_id)
        .fetch_all(&db_pool)
        .await?;

    Ok(post_vec)
}

#[server]
pub async fn get_post_vec_by_forum_name(forum_name: String, sort_type: SortType) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;

    let post_vec = sqlx::query_as::<_, Post>(
        format!("SELECT p.* FROM posts p \
        JOIN forums f on f.forum_id = p.forum_id \
        WHERE f.forum_name = $1 \
        ORDER BY {} DESC", sort_type.to_order_by_code()).as_str()
    )
        .bind(forum_name)
        .fetch_all(&db_pool)
        .await?;

    Ok(post_vec)
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
                v.comment_id as vote_comment_id,
                v.value,
                v.timestamp as vote_timestamp
        FROM posts p
        LEFT JOIN votes v
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
pub async fn create_post(forum: String, title: String, body: String, is_nsfw: Option<String>, tag: Option<String>) -> Result<(), ServerFnError> {
    log::info!("Create [[content]] '{title}'");
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if forum.is_empty() || title.is_empty() {
        return Err(ServerFnError::new("Cannot create content without a valid forum and title."));
    }

    let new_post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (title, body, is_nsfw, tags, forum_id, forum_name, creator_id, creator_name)
         VALUES (
            $1, $2, $3, $4,
            (SELECT forum_id FROM forums WHERE forum_name = $5),
            $5, $6, $7
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
                log::trace!("Current post id: {current_post_id:?}, new post id: {new_post_id}");
                new_post_id
            }
            else {
                log::trace!("Could not parse new post id: {new_post_id_string}, reuse current post id: {current_post_id:?}");
                current_post_id.cloned().unwrap_or_default()
            }
        }
        else {
            log::trace!("Could not find new post id, reuse current post id: {current_post_id:?}");
            current_post_id.cloned().unwrap_or_default()
        }
    })
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
                    existing_forums.map(|result| match result {
                        Ok(forum_set) => {
                            log::trace!("Forum name set: {forum_set:?}");

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
                        Err(e) => {
                            log::info!("Error while getting forum names: {}", e);
                            view! { <ErrorIcon/> }.into_view()
                        },
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
            log::debug!("Load data for post: {post_id}");
            get_post_with_vote_by_id(post_id)
        });

    view! {
        <div class="flex flex-col content-start gap-1">
            <Transition fallback=move || view! { <LoadingIcon/> }>
                {
                    post.map(|result| match result {
                        Ok(post) => {
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
                        Err(e) => {
                            log::info!("Error while getting forum names: {}", e);
                            view! { <ErrorIcon/> }.into_view()
                        },
                    })
                }
            </Transition>
            <CommentSortWidget/>
            <CommentSection/>
        </div>
    }
}

/// Component to encapsulate the widgets associated with each post
#[component]
fn PostWidgetBar<'a>(post: &'a PostWithVote) -> impl IntoView {

    let content = ContentWithVote::Post(&post.post, &post.vote);

    view! {
        <div class="flex gap-2">
            <VotePanel
                content=content
            />
            <CommentButton post_id=post.post.post_id/>
            <AuthorWidget author=&post.post.creator_name/>
            <TimeSinceWidget timestamp=&post.post.create_timestamp/>
        </div>
    }
}