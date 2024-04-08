use std::collections::BTreeSet;

use const_format::concatcp;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};
use crate::app::{GlobalState, PARAM_ROUTE_PREFIX, PUBLISH_ROUTE};
use crate::auth::{LoginGuardButton};
use crate::editor::FormTextEditor;
use crate::icons::{ErrorIcon, LoadingIcon, LogoIcon, PlusIcon, StarIcon, SubscribedIcon};
use crate::navigation_bar::get_create_post_path;
use crate::post::{CREATE_POST_FORUM_QUERY_PARAM, CREATE_POST_ROUTE, Post, POST_ROUTE_PREFIX};
use crate::ranking::{ScoreIndicator, SortType};
use crate::unpack::SuspenseUnpack;
use crate::widget::{AuthorWidget, PostSortWidget, TimeSinceWidget};

pub const CREATE_FORUM_SUFFIX: &str = "/forum";
pub const CREATE_FORUM_ROUTE: &str = concatcp!(PUBLISH_ROUTE, CREATE_FORUM_SUFFIX);
pub const FORUM_ROUTE_PREFIX: &str = "/forums";
pub const FORUM_ROUTE_PARAM_NAME: &str = "forum_name";
pub const FORUM_ROUTE: &str = concatcp!(
    FORUM_ROUTE_PREFIX,
    PARAM_ROUTE_PREFIX,
    FORUM_ROUTE_PARAM_NAME
);

pub const FORUM_FETCH_LIMIT: i64 = 20;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Forum {
    pub forum_id: i64,
    pub forum_name: String,
    pub description: String,
    pub is_nsfw: bool,
    pub is_banned: bool,
    pub tags: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub num_members: i32,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ForumSubscription {
    pub subscription_id: i64,
    pub user_id: i64,
    pub forum_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ForumWithSubscription {
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub forum: Forum,
    pub subscription_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ForumContent {
    pub forum: ForumWithSubscription,
    pub post_vec: Vec<Post>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::collections::BTreeSet;

    use leptos::ServerFnError;
    use sqlx::PgPool;

    use crate::forum::{Forum, ForumContent, ForumWithSubscription};
    use crate::post::ssr::get_post_vec_by_forum_name;
    use crate::ranking::SortType;

    pub async fn get_forum_contents(
        forum_name: &str,
        sort_type: SortType,
        user_id: Option<i64>,
        db_pool: PgPool,
    ) -> Result<ForumContent, ServerFnError> {
        let forum = sqlx::query_as::<_, ForumWithSubscription>(
            "SELECT f.*, s.subscription_id \
            FROM forums f \
            LEFT JOIN forum_subscriptions s ON \
                s.forum_id = f.forum_id AND \
                s.user_id = $1 \
            where forum_name = $2",
        )
        .bind(user_id)
        .bind(forum_name)
        .fetch_one(&db_pool)
        .await?;

        let post_vec = get_post_vec_by_forum_name(forum_name, sort_type, db_pool).await?;

        Ok(ForumContent { forum, post_vec })
    }

    pub async fn is_forum_available(
        forum_name: &str,
        db_pool: PgPool,
    ) -> Result<bool, ServerFnError> {
        let forum_exist = sqlx::query!(
            "SELECT forum_id FROM forums WHERE forum_name = $1",
            forum_name
        )
        .fetch_one(&db_pool)
        .await;

        match forum_exist {
            Ok(_) => Ok(false),
            Err(sqlx::error::Error::RowNotFound) => Ok(true),
            Err(e) => Err(ServerFnError::from(e)),
        }
    }

    pub async fn get_forum_by_name(
        forum_name: &str,
        db_pool: PgPool,
    ) -> Result<Forum, ServerFnError> {
        let forum = sqlx::query_as!(
            Forum,
            "SELECT * FROM forums WHERE forum_name = $1",
            forum_name
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(forum)
    }

    pub async fn get_matching_forum_names(
        forum_prefix: String,
        limit: i64,
        db_pool: PgPool,
    ) -> Result<BTreeSet<String>, ServerFnError> {
        let forum_name_vec = sqlx::query!(
            "SELECT forum_name FROM forums WHERE forum_name like $1 LIMIT $2",
            forum_prefix + "%",
            limit,
        )
        .fetch_all(&db_pool)
        .await?;

        let mut forum_name_set = BTreeSet::<String>::new();

        for row in forum_name_vec {
            forum_name_set.insert(row.forum_name);
        }

        Ok(forum_name_set)
    }

    pub async fn get_popular_forum_names(
        limit: i64,
        db_pool: PgPool,
    ) -> Result<Vec<String>, ServerFnError> {
        let forum_record_vec = sqlx::query!(
            "SELECT * FROM forums ORDER BY num_members DESC, forum_name LIMIT $1",
            limit
        )
        .fetch_all(&db_pool)
        .await?;

        let mut forum_name_vec = Vec::<String>::with_capacity(forum_record_vec.len());

        for forum in forum_record_vec {
            forum_name_vec.push(forum.forum_name);
        }

        Ok(forum_name_vec)
    }

    pub async fn get_subscribed_forum_names(
        user_id: i64,
        db_pool: PgPool,
    ) -> Result<Vec<String>, ServerFnError> {
        let forum_record_vec = sqlx::query!(
            "SELECT f.forum_name FROM forums f \
            JOIN forum_subscriptions s ON \
                f.forum_id = s.forum_id AND \
                f.creator_id = $1 \
            ORDER BY forum_name",
            user_id,
        )
        .fetch_all(&db_pool)
        .await?;

        let mut forum_name_vec = Vec::<String>::with_capacity(forum_record_vec.len());

        for forum in forum_record_vec {
            forum_name_vec.push(forum.forum_name);
        }

        Ok(forum_name_vec)
    }

    pub async fn create_forum(
        name: &str,
        description: &str,
        is_nsfw: bool,
        user_id: i64,
        db_pool: PgPool,
    ) -> Result<Forum, ServerFnError> {
        if name.is_empty() {
            return Err(ServerFnError::new("Cannot create Sphere with empty name."));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() && (c.is_lowercase() || c.is_numeric()))
        {
            return Err(ServerFnError::new(
                "Sphere name can only contain alphanumeric lowercase characters.",
            ));
        }

        let forum = sqlx::query_as!(
            Forum,
            "INSERT INTO forums (forum_name, description, is_nsfw, creator_id) VALUES ($1, $2, $3, $4) RETURNING *",
            name,
            description,
            is_nsfw,
            user_id
        )
            .fetch_one(&db_pool)
            .await?;

        Ok(forum)
    }

    pub async fn subscribe(
        forum_id: i64,
        user_id: i64,
        db_pool: PgPool,
    ) -> Result<(), ServerFnError> {
        sqlx::query!(
            "INSERT INTO forum_subscriptions (user_id, forum_id) VALUES ($1, $2)",
            user_id,
            forum_id
        )
        .execute(&db_pool)
        .await?;

        sqlx::query!(
            "UPDATE forums SET num_members = num_members + 1 WHERE forum_id = $1",
            forum_id
        )
        .execute(&db_pool)
        .await?;

        Ok(())
    }

    pub async fn unsubscribe(
        forum_id: i64,
        user_id: i64,
        db_pool: PgPool,
    ) -> Result<(), ServerFnError> {
        sqlx::query!(
            "DELETE FROM forum_subscriptions WHERE user_id = $1 AND forum_id = $2",
            user_id,
            forum_id,
        )
        .execute(&db_pool)
        .await?;

        sqlx::query!(
            "UPDATE forums SET num_members = num_members - 1 WHERE forum_id = $1",
            forum_id
        )
        .execute(&db_pool)
        .await?;

        Ok(())
    }
}

#[server]
pub async fn is_forum_available(forum_name: String) -> Result<bool, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_existence = ssr::is_forum_available(&forum_name, db_pool).await?;
    Ok(forum_existence)
}

#[server]
pub async fn get_forum_by_name(forum_name: String) -> Result<Forum, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum = ssr::get_forum_by_name(&forum_name, db_pool).await?;
    Ok(forum)
}

#[server]
pub async fn get_matching_forum_names(
    forum_prefix: String,
) -> Result<BTreeSet<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_name_set =
        ssr::get_matching_forum_names(forum_prefix, FORUM_FETCH_LIMIT, db_pool).await?;
    Ok(forum_name_set)
}

#[server]
pub async fn get_subscribed_forum_names() -> Result<Vec<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user = get_user().await?;
    let forum_name_vec = ssr::get_subscribed_forum_names(user.user_id, db_pool).await?;
    Ok(forum_name_vec)
}

#[server]
pub async fn get_popular_forum_names() -> Result<Vec<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let forum_name_vec = ssr::get_popular_forum_names(FORUM_FETCH_LIMIT, db_pool).await?;
    Ok(forum_name_vec)
}

#[server]
pub async fn get_forum_contents(
    forum_name: String,
    sort_type: SortType,
) -> Result<ForumContent, ServerFnError> {
    let db_pool = get_db_pool()?;
    let user_id = match get_user().await {
        Ok(user) => Some(user.user_id),
        Err(_) => None,
    };

    let forum_content =
        ssr::get_forum_contents(forum_name.as_str(), sort_type, user_id, db_pool).await?;

    Ok(forum_content)
}

#[server]
pub async fn create_forum(
    forum_name: String,
    description: String,
    is_nsfw: Option<String>,
) -> Result<(), ServerFnError> {
    log::trace!("Create Sphere '{forum_name}', {description}, {is_nsfw:?}");
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    let new_forum_path: &str = &(FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name.as_str());

    ssr::create_forum(
        forum_name.as_str(),
        description.as_str(),
        is_nsfw.is_some(),
        user.user_id,
        db_pool,
    )
    .await?;

    // Redirect to the new forum
    leptos_axum::redirect(new_forum_path);
    Ok(())
}

#[server]
pub async fn subscribe(forum_id: i64) -> Result<(), ServerFnError> {
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    ssr::subscribe(forum_id, user.user_id, db_pool).await?;

    Ok(())
}

#[server]
pub async fn unsubscribe(forum_id: i64) -> Result<(), ServerFnError> {
    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    ssr::unsubscribe(forum_id, user.user_id, db_pool).await?;
    Ok(())
}

/// Get the current forum name from the path. When the current path does not contain a forum, returns the last valid forum. Used to avoid sending a request when leaving a page
pub fn get_forum_name_memo(params: Memo<ParamsMap>) -> Memo<String> {
    create_memo(move |current_forum_name: Option<&String>| {
        if let Some(new_forum_name) =
            params.with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned())
        {
            log::trace!(
                "Current forum name {current_forum_name:?}, new forum name: {new_forum_name}"
            );
            new_forum_name
        } else {
            log::trace!("No valid forum name, keep current value: {current_forum_name:?}");
            current_forum_name.cloned().unwrap_or_default()
        }
    })
}

/// Component to create new forums
#[component]
pub fn CreateForum() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let create_forum_result = state.create_forum_action.value();
    // check if the server has returned an error
    let has_error = move || create_forum_result.with(|val| matches!(val, Some(Err(_))));

    let forum_name = create_rw_signal(String::new());
    let is_forum_available = create_resource(
        move || (forum_name.get(), state.create_forum_action.version().get()),
        move |(forum_name, _)| async {
            if forum_name.is_empty() {
                None
            } else {
                Some(is_forum_available(forum_name).await)
            }
        },
    );

    let is_name_taken = create_rw_signal(false);
    let description = create_rw_signal(String::new());
    let is_name_empty = move || forum_name.with(|forum_name| forum_name.is_empty());
    let is_name_alphanumeric =
        move || forum_name.with(|forum_name| forum_name.chars().all(char::is_alphanumeric));
    let are_inputs_invalid = create_memo(move |_| {
        is_name_empty()
            || is_name_taken()
            || !is_name_alphanumeric()
            || description.with(|description| description.is_empty())
    });

    view! {
        <div class="flex flex-col gap-2 mx-auto w-4/5 2xl:w-1/3">
            <ActionForm action=state.create_forum_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Settle a Sphere!"</h2>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            name="forum_name"
                            placeholder="Name"
                            autocomplete="off"
                            class="input input-bordered input-primary h-input_l flex-none w-1/2"
                            autofocus
                            on:input=move |ev| {
                                forum_name.update(|value| *value = event_target_value(&ev).to_lowercase());
                            }
                            prop:value=forum_name
                        />
                        <Suspense fallback=move || view! { <LoadingIcon/> }>
                        {
                            move || is_forum_available.map(|result| match result {
                                None | Some(Ok(true)) => {
                                    is_name_taken.set(false);
                                    View::default()
                                },
                                Some(Ok(false)) => {
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-input_l flex content-center">
                                            <ErrorIcon/>
                                            <span>"Unavailable."</span>
                                        </div>
                                    }.into_view()
                                },
                                Some(Err(e)) => {
                                    log::error!("Error while checking forum existence: {e}");
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-input_l flex content-center">
                                            <ErrorIcon/>
                                            <span>"Server error."</span>
                                        </div>
                                    }.into_view()
                                },
                            })

                        }
                        </Suspense>
                        <div class="alert alert-error h-input_l flex content-center" class:hidden=move || is_name_empty() || is_name_alphanumeric()>
                            <ErrorIcon/>
                            <span>"Only alphanumeric characters."</span>
                        </div>
                    </div>
                    <FormTextEditor
                        name="description"
                        placeholder="Description"
                        content=description
                    />
                    <div class="form-control">
                        <label class="cursor-pointer label p-0">
                            <span class="label-text">"NSFW content"</span>
                            <input type="checkbox" name="is_nsfw" class="checkbox checkbox-primary"/>
                        </label>
                    </div>
                    <Suspense fallback=move || view! { <LoadingIcon/> }>
                        <button type="submit" class="btn btn-active btn-secondary" disabled=are_inputs_invalid>"Create"</button>
                    </Suspense>
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

/// Component to display a forum's banner
#[component]
pub fn ForumBanner() -> impl IntoView {
    let params = use_params_map();
    let forum_name = get_forum_name_memo(params);
    let forum_path = move || {
        FORUM_ROUTE_PREFIX.to_owned()
            + "/"
            + params
                .with(|params| params.get(FORUM_ROUTE_PARAM_NAME).cloned())
                .unwrap_or_default()
                .as_ref()
    };

    let forum = create_resource(
        move || forum_name(),
        move |forum_name| {
            log::debug!("Load data for forum: {forum_name}");
            get_forum_by_name(forum_name)
        },
    );

    view! {
        <div class="flex flex-col gap-2 mt-2 mx-2 w-full">
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                {
                    move || {
                         forum.map(|result| match result {
                            Ok(forum) => {
                                let forum_banner_image = format!("url({})", forum.banner_url.clone().unwrap_or(String::from("/banner.jpg")));
                                view! {
                                    <a
                                        href=forum_path()
                                        class="bg-cover bg-center bg-no-repeat rounded w-full h-24 flex items-center justify-center"
                                        style:background-image=forum_banner_image
                                    >
                                        <div class="p-3 backdrop-blur bg-black/50 rounded-lg flex justify-center gap-3">
                                            <LogoIcon class="h-12 w-12"/>
                                            <span class="text-4xl">{forum_name()}</span>
                                        </div>
                                    </a>
                                }.into_view()
                            },
                            Err(e) => {
                                log::info!("Error: {e}");
                                view! { <ErrorIcon/> }.into_view()
                            },
                        })
                    }
                }
            </Transition>
            <Outlet/>
        </div>
    }
}

/// Component to display a forum's contents
#[component]
pub fn ForumContents() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let params = use_params_map();
    let forum_name = get_forum_name_memo(params);
    let forum_content = create_resource(
        move || {
            (
                forum_name(),
                state.create_post_action.version().get(),
                state.post_sort_type.get(),
            )
        },
        move |(forum_name, _, sort_type)| get_forum_contents(forum_name, sort_type),
    );

    view! {
        <SuspenseUnpack
            resource=forum_content let:forum_content
        >
            <ForumToolbar forum=&forum_content.forum/>
            <ForumPostMiniatures post_vec=&forum_content.post_vec/>
        </SuspenseUnpack>
    }
}

/// Component to display the forum toolbar
#[component]
pub fn ForumToolbar<'a>(forum: &'a ForumWithSubscription) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_id = forum.forum.forum_id;
    let forum_name = create_rw_signal(forum.forum.forum_name.clone());
    let is_subscribed = create_rw_signal(forum.subscription_id.is_some());

    view! {
        <div class="flex w-full justify-between content-center">
            <PostSortWidget/>
            <div class="flex gap-1">
                <div class="tooltip" data-tip="Join">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <StarIcon class="h-6 w-6" show_colour=is_subscribed/> }
                        let:_user
                    >
                        <button type="submit" class="btn btn-circle btn-ghost" on:click=move |_| {
                                is_subscribed.update(|value| {
                                    *value = !*value;
                                    if *value {
                                        state.subscribe_action.dispatch(Subscribe { forum_id });
                                    } else {
                                        state.unsubscribe_action.dispatch(Unsubscribe { forum_id });
                                    }
                                })
                            }
                        >
                            <StarIcon class="h-6 w-6" show_colour=is_subscribed/>
                        </button>
                    </LoginGuardButton>
                </div>
                <div class="tooltip" data-tip="Join">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <SubscribedIcon class="h-6 w-6" show_colour=is_subscribed/> }
                        let:_user
                    >
                        <button type="submit" class="btn btn-circle btn-ghost" on:click=move |_| {
                                is_subscribed.update(|value| {
                                    *value = !*value;
                                    if *value {
                                        state.subscribe_action.dispatch(Subscribe { forum_id });
                                    } else {
                                        state.unsubscribe_action.dispatch(Unsubscribe { forum_id });
                                    }
                                })
                            }
                        >
                            <SubscribedIcon class="h-6 w-6" show_colour=is_subscribed/>
                        </button>
                    </LoginGuardButton>
                </div>
                <div class="tooltip" data-tip="New">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost"
                        login_button_content=move || view! { <PlusIcon class="h-6 w-6"/> }
                        redirect_path_fn=&get_create_post_path
                        let:_user
                    >
                        <Form action=CREATE_POST_ROUTE class="flex">
                            <input type="text" name=CREATE_POST_FORUM_QUERY_PARAM class="hidden" value=forum_name/>
                            <button type="submit" class="btn btn-circle btn-ghost">
                                <PlusIcon class="h-6 w-6"/>
                            </button>
                        </Form>
                    </LoginGuardButton>
                </div>
            </div>
        </div>
    }
}

/// Component to display a given set of forum posts
#[component]
pub fn ForumPostMiniatures<'a>(post_vec: &'a Vec<Post>) -> impl IntoView {
    view! {
        <ul class="menu w-full text-lg">
            {
                post_vec.iter().map(move |post| {
                    let post_path = FORUM_ROUTE_PREFIX.to_owned() + "/" + post.forum_name.as_str() + POST_ROUTE_PREFIX + "/" + &post.post_id.to_string();
                    view! {
                        <li>
                            <a href=post_path>
                                <div class="flex flex-col gap-1">
                                    <h2 class="card-title">{post.title.clone()}</h2>
                                    <div class="flex gap-2">
                                        <ScoreIndicator score=post.score/>
                                        <AuthorWidget author=&post.creator_name/>
                                        <TimeSinceWidget timestamp=&post.create_timestamp/>
                                    </div>
                                </div>
                            </a>
                        </li>
                    }
                }).collect_view()
            }
        </ul>
    }
}
