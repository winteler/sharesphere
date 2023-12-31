use cfg_if::cfg_if;
use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::auth::{LoginButton};
use crate::comment::{CommentButton, CommentSection};
use crate::common_components::FormTextEditor;
use crate::constants::{SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR};
use crate::forum::{get_all_forum_names};
use crate::icons::{AuthorIcon, ClockIcon, ErrorIcon, LoadingIcon, ScoreIcon};

pub const CREATE_POST_SUFFIX : &str = "/content";
pub const CREATE_POST_ROUTE : &str = concatcp!(PUBLISH_ROUTE, CREATE_POST_SUFFIX);
pub const CREATE_POST_FORUM_QUERY_PARAM : &str = "forum";
pub const POST_ROUTE_PREFIX : &str = "/posts";
pub const POST_ROUTE_PARAM_NAME : &str = "post_name";
pub const POST_ROUTE : &str = concatcp!(POST_ROUTE_PREFIX, PARAM_ROUTE_PREFIX, POST_ROUTE_PARAM_NAME);

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
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
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
        use crate::forum::FORUM_ROUTE_PREFIX;
    }
}

#[server]
pub async fn get_post_by_id(id: i64) -> Result<Post, ServerFnError> {
    let db_pool = get_db_pool()?;
    Ok(sqlx::query_as!(
        Post,
        "SELECT * FROM posts WHERE id = $1",
        id
    )
        .fetch_one(&db_pool)
        .await?)
}

#[server]
pub async fn get_post_vec_by_forum_name(forum_name: String) -> Result<Vec<Post>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let post_vec = sqlx::query_as!(
        Post,
        "SELECT posts.* FROM posts \
        join forums on forums.id = posts.forum_id \
        WHERE forums.name = $1",
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
        return Err(ServerFnError::ServerError(String::from("Cannot create content without a valid forum and title.")));
    }

    let new_post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (title, body, is_nsfw, tags, forum_id, creator_id, creator_name)
         VALUES (
            $1, $2, $3, $4,
            (SELECT id FROM forums WHERE name = $5),
            $6, $7
        ) RETURNING *",
        title.clone(),
        body,
        is_nsfw.is_some(),
        tag.unwrap_or_default(),
        forum.clone(),
        user.id,
        user.username,
    )
        .fetch_one(&db_pool)
        .await?;

    log::info!("New post id: {}", new_post.id);
    let new_post_path : &str = &(FORUM_ROUTE_PREFIX.to_owned() + "/" + forum.as_str() + POST_ROUTE_PREFIX + "/" + new_post.id.to_string().as_ref());
    leptos_axum::redirect(new_post_path);
    Ok(())
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
    let state = expect_context::<GlobalState>();
    let params = use_params_map();
    let post_id = create_rw_signal(0i64);
    let get_post_id = move || {
        let post_id_parse_result = params.with(|params| params.get(POST_ROUTE_PARAM_NAME).cloned()).unwrap_or_default().parse::<i64>();
        if post_id_parse_result.is_ok() {
            post_id.update(|value: &mut i64| *value = post_id_parse_result.unwrap());
        }
        post_id.get()
    };

    let is_user_logged_in = move || state.user.with(|user| match user {
        Some(Ok(user)) => !user.anonymous,
        _ => false,
    });

    // TODO: create PostDetail struct with additional info, like vote of user. Load this here instead of normal post
    let post = create_resource(
        move || get_post_id(),
        move |post_id| get_post_by_id(post_id));

    view! {
        <Suspense fallback=move || (view! { <LoadingIcon/> })>
            {
                post.with(|result| {
                    match result {
                        Some(Ok(post)) => {
                            view! {
                                <div class="flex flex-col content-start gap-1">
                                    <div class="card">
                                        <div class="card-body">
                                            <div class="flex flex-col gap-4">
                                                <h2 class="card-title">{post.title.clone()}</h2>
                                                {post.body.clone()}
                                                <div class="flex gap-2">
                                                    <VotePanel is_logged_in=false score=post.score/>
                                                    <PostAuthor post=&post/>
                                                    <PostTime post=&post/>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    <CommentButton post=&post/>
                                    <CommentSection post=&post/>
                                </div>
                            }.into_view()
                        },
                        Some(Err(e)) => {
                            log::info!("Error while getting forum names: {}", e);
                            view! { <ErrorIcon/> }.into_view()
                        },
                        None => {
                            log::trace!("Resource not loaded yet.");
                            view! { <Outlet/> }.into_view()
                        }
                    }
                })
            }
        </Suspense>
    }
}

/// Component to display a post's score
#[component]
pub fn ScoreIndicator(score: i32) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <ScoreIcon/>
            {score}
        </div>
    }
}

/// Component to display and modify post's score
#[component]
pub fn VotePanel(score: i32, is_logged_in: bool) -> impl IntoView {
    view! {
        <div class="flex items-center gap-1">
            <Show
                when=move || { is_logged_in }
                fallback=|| view! { <LoginButton><div class="btn btn-circle btn-sm hover:btn-success">"+"</div></LoginButton> }
            >
                <button class="btn btn-circle btn-sm hover:btn-success">
                    "+"
                </button>
            </Show>
            <ScoreIndicator score=score/>
            <Show
                when=move || { is_logged_in }
                fallback=|| view! { <LoginButton><div class="btn btn-circle btn-sm hover:btn-error">"-"</div></LoginButton> }
            >
                <button class="btn btn-circle btn-sm hover:btn-error">
                    "-"
                </button>
            </Show>
        </div>
    }
}

/// Component to display a post's author
#[component]
pub fn PostAuthor<'a>(post: &'a Post) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <AuthorIcon/>
            {post.creator_name.clone()}
        </div>
    }
}

/// Component to display the creation time of a post
#[component]
pub fn PostTime<'a>(post: &'a Post) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <ClockIcon/>
            {
                let post_age = chrono::Utc::now().signed_duration_since(post.create_timestamp);
                let seconds = post_age.num_seconds();

                match seconds {
                    seconds if seconds < SECONDS_IN_MINUTE => {
                        format!("{} {}", seconds, if seconds == 1 { "second" } else { "seconds" })
                    },
                    seconds if seconds < SECONDS_IN_HOUR => {
                        let minutes = seconds/SECONDS_IN_MINUTE;
                        format!("{} {}", minutes, if minutes == 1 { "minute" } else { "minutes" })
                    },
                    seconds if seconds < SECONDS_IN_DAY => {
                        let hours = seconds/SECONDS_IN_HOUR;
                        format!("{} {}", hours, if hours == 1 { "hour" } else { "hours" })
                    },
                    seconds if seconds < SECONDS_IN_MONTH => {
                        let days = seconds/SECONDS_IN_DAY;
                        format!("{} {}", days, if days == 1 { "day" } else { "days" })
                    },
                    seconds if seconds < SECONDS_IN_YEAR => {
                        let months = seconds/SECONDS_IN_MONTH;
                        format!("{} {}", months, if months == 1 { "month" } else { "months" })
                    },
                    _ => {
                        let years = seconds/SECONDS_IN_YEAR;
                        format!("{} {}", years, if years == 1 { "year" } else { "years" })
                    },
                }
            }
        </div>
    }
}