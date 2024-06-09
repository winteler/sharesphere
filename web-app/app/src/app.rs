use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::auth::*;
use crate::comment::CommentSortType;
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::forum::*;
use crate::forum_management::{ForumCockpit, MANAGE_FORUM_ROUTE, ModeratePost};
use crate::icons::*;
use crate::navigation_bar::*;
use crate::post::*;
use crate::ranking::SortType;
use crate::sidebar::*;
use crate::widget::PostSortWidget;

pub const PARAM_ROUTE_PREFIX: &str = "/:";
pub const PUBLISH_ROUTE: &str = "/publish";

#[derive(Copy, Clone)]
pub struct ModerateState {
    pub can_moderate: Signal<bool>,
    pub moderate_post_action: Action<ModeratePost, Result<Post, ServerFnError>>,
}

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub login_action: Action<Login, Result<User, ServerFnError>>,
    pub logout_action: Action<EndSession, Result<(), ServerFnError>>,
    pub subscribe_action: Action<Subscribe, Result<(), ServerFnError>>,
    pub unsubscribe_action: Action<Unsubscribe, Result<(), ServerFnError>>,
    pub edit_post_action: Action<EditPost, Result<Post, ServerFnError>>,
    pub create_forum_action: Action<CreateForum, Result<(), ServerFnError>>,
    pub current_forum_name: Option<Memo<String>>,
    pub current_post_id: Option<Memo<i64>>,
    pub post_sort_type: RwSignal<SortType>,
    pub comment_sort_type: RwSignal<SortType>,
    pub user: Resource<(usize, usize, usize), Result<Option<User>, ServerFnError>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let login_action = create_server_action::<Login>();
        let logout_action = create_server_action::<EndSession>();
        let create_forum_action = create_server_action::<CreateForum>();
        Self {
            login_action,
            logout_action,
            subscribe_action: create_server_action::<Subscribe>(),
            unsubscribe_action: create_server_action::<Unsubscribe>(),
            edit_post_action: create_server_action::<EditPost>(),
            create_forum_action,
            current_forum_name: None,
            current_post_id: None,
            post_sort_type: create_rw_signal(SortType::Post(PostSortType::Hot)),
            comment_sort_type: create_rw_signal(SortType::Comment(CommentSortType::Best)),
            user: create_local_resource(
                move || {
                    (
                        login_action.version().get(),
                        logout_action.version().get(),
                        create_forum_action.version().get(),
                    )
                },
                move |_| get_user(),
            ),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::env;
    use std::sync::OnceLock;

    use anyhow::Context;
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use tokio::runtime::Handle;

    use crate::auth::ssr::AuthSession;

    use super::*;

    pub const DB_URL_ENV: &str = "DATABASE_URL";

    pub fn get_session() -> Result<AuthSession, AppError> {
        use_context::<AuthSession>().ok_or_else(|| AppError::new("Auth session missing."))
    }

    pub async fn create_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var(DB_URL_ENV)?)
            .await
            .with_context(|| format!("Failed to connect to DB"))
    }

    struct DbPoolGetter {
        pool: Result<PgPool, AppError>,
    }

    impl DbPoolGetter {
        fn new() -> Self {
            // Create the runtime
            let handle = Handle::current();
            let pool = std::thread::spawn(move || {
                // Using Handle::block_on to run async code in the new thread.
                handle.block_on(async {
                    create_db_pool()
                        .await
                        .or_else(|_| Err(AppError::new("Pool missing.")))
                })
            })
            .join()
            .unwrap_or(Err(AppError::new("Failed to create DB pool.")));

            Self { pool }
        }
    }

    static POOL: OnceLock<DbPoolGetter> = OnceLock::new();

    pub fn get_db_pool() -> Result<PgPool, AppError> {
        POOL.get_or_init(|| DbPoolGetter::new()).pool.clone()
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Provide global context for app
    provide_context(GlobalState::new());

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/sharesphere.css"/>

        // sets the document title
        <Title text="Welcome to ShareSphere"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
        }>
            <main class="h-screen text-white">
                <input id="my-drawer" type="checkbox" class="drawer-toggle"/>
                <div class="drawer-content h-full flex flex-col max-2xl:items-center">
                    <NavigationBar/>
                    <div class="grow flex w-full overflow-hidden">
                        <div class="max-2xl:hidden">
                            <LeftSidebar/>
                        </div>
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path=FORUM_ROUTE view=ForumBanner>
                                <Route path=POST_ROUTE view=Post/>
                                <Route path=MANAGE_FORUM_ROUTE view=ForumCockpit/>
                                <Route path="" view=ForumContents/>
                            </Route>
                            <Route path=AUTH_CALLBACK_ROUTE view=AuthCallback/>
                            <Route path=PUBLISH_ROUTE view=LoginPageGuard>
                                <Route path=CREATE_FORUM_SUFFIX view=CreateForum/>
                                <Route path=CREATE_POST_SUFFIX view=CreatePost/>
                            </Route>
                        </Routes>
                        <div class="max-2xl:hidden">
                            <RightSidebar/>
                        </div>
                    </div>
                </div>
                <div class="drawer-side">
                    <label for="my-drawer" class="drawer-overlay"></label>
                    <LeftSidebar/>
                </div>
            </main>
        </Router>
    }
}

/// Component to guard pages requiring a login, and enable the user to login with a redirect
#[component]
fn LoginPageGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            {
                move || {
                     state.user.with(|user| match user {
                        Some(Ok(Some(user))) => {
                            log::debug!("Login guard, current user: {user:?}");
                            view! { <Outlet/> }.into_view()
                        },
                        Some(_) => {
                            view! { <LoginWindow/> }.into_view()
                        },
                        None => {
                            log::trace!("Resource not loaded yet.");
                            view! { <Outlet/> }.into_view()
                        }
                    })
                }
            }
        </Transition>
    }
}

/// Renders a page requesting a login
#[component]
fn LoginWindow() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_path = create_rw_signal(String::default());

    view! {
        <div class="hero min-h-screen">
            <div class="hero-content flex text-center">
                <AuthErrorIcon class="h-44 w-44"/>
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">"Not authenticated"</h1>
                    <p class="pt-4">"Sorry, we had some trouble identifying you."</p>
                    <p class="pb-4">"Please login to access this page."</p>
                    <form action=state.login_action.url() method="post" rel="external">
                        <input type="text" name="redirect_url" class="hidden" value=current_path/>
                        <button type="submit" class="btn btn-primary btn-wide rounded" on:click=move |_| get_current_path(current_path)>
                            "Login"
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <div class="flex flex-col flex-1 w-full overflow-y-auto  pt-2 px-2 gap-2">
            <div
                class="flex-none bg-cover bg-center bg-no-repeat rounded w-full h-24 flex items-center justify-center"
                style="background-image: url(/banner.jpg)"
            >
                <div class="p-3 backdrop-blur bg-black/50 rounded-lg flex justify-center gap-3">
                    <span class="text-4xl select-none">"ShareSphere"</span>
                </div>
            </div>
            <PostSortWidget/>
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                { move || {
                     state.user.map(|user| match user {
                        Ok(Some(user)) => {
                            log::trace!("Authenticated, current user: {user:?}");
                            view! { <UserHomePage user=user/> }.into_view()
                        },
                        _ => {
                            view! { <DefaultHomePage/> }.into_view()
                        }
                    })

                }}
            </Transition>
        </div>
    }
}

/// Renders the home page anonymous users.
#[component]
fn DefaultHomePage() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let post_vec = create_rw_signal(Vec::<Post>::new());
    let additional_load_count = create_rw_signal(0);
    let is_loading = create_rw_signal(false);
    let load_error = create_rw_signal(None);
    let list_ref = create_node_ref::<html::Ul>();

    // Effect for initial load and sort changes
    create_effect(move |_| {
        let sort_type = state.post_sort_type.get();
        is_loading.set(true);
        load_error.set(None);
        post_vec.update(|post_vec| post_vec.clear());
        spawn_local(async move {
            match get_sorted_post_vec(sort_type, 0).await {
                Ok(new_post_vec) => {
                    post_vec.update(|post_vec| {
                        if let Some(list_ref) = list_ref.get_untracked() {
                            list_ref.set_scroll_top(0);
                        }
                        *post_vec = new_post_vec;
                    });
                }
                Err(e) => load_error.set(Some(AppError::from(&e))),
            }
            is_loading.set(false);
        });
    });

    // Effect for additional load upon reaching end of scroll
    create_effect(move |_| {
        if additional_load_count.get() > 0 {
            is_loading.set(true);
            let post_count = post_vec.with_untracked(|post_vec| post_vec.len());
            spawn_local(async move {
                match get_sorted_post_vec(state.post_sort_type.get_untracked(), post_count).await {
                    Ok(mut new_post_vec) => {
                        post_vec.update(|post_vec| post_vec.append(&mut new_post_vec))
                    }
                    Err(e) => load_error.set(Some(AppError::from(&e))),
                }
                is_loading.set(false);
            });
        }
    });

    view! {
        <ForumPostMiniatures
            post_vec=post_vec
            is_loading=is_loading
            load_error=load_error
            additional_load_count=additional_load_count
            list_ref=list_ref
        />
    }
}

/// Renders the home page of a given user.
#[component]
fn UserHomePage<'a>(user: &'a User) -> impl IntoView {
    let user_id = user.user_id;
    let state = expect_context::<GlobalState>();
    let post_vec = create_rw_signal(Vec::<Post>::new());
    let additional_load_count = create_rw_signal(0);
    let is_loading = create_rw_signal(false);
    let load_error = create_rw_signal(None);
    let list_ref = create_node_ref::<html::Ul>();

    // Effect for initial load and sort changes
    create_effect(move |_| {
        let sort_type = state.post_sort_type.get();
        is_loading.set(true);
        load_error.set(None);
        post_vec.update(|post_vec| post_vec.clear());
        spawn_local(async move {
            match get_subscribed_post_vec(user_id, sort_type, 0).await {
                Ok(new_post_vec) => {
                    post_vec.update(|post_vec| {
                        if let Some(list_ref) = list_ref.get_untracked() {
                            list_ref.set_scroll_top(0);
                        }
                        *post_vec = new_post_vec;
                    });
                }
                Err(e) => load_error.set(Some(AppError::from(&e))),
            }
            is_loading.set(false);
        });
    });

    // Effect for additional load upon reaching end of scroll
    create_effect(move |_| {
        if additional_load_count.get() > 0 {
            is_loading.set(true);
            load_error.set(None);
            let post_count = post_vec.with_untracked(|post_vec| post_vec.len());
            spawn_local(async move {
                match get_subscribed_post_vec(
                    user_id,
                    state.post_sort_type.get_untracked(),
                    post_count,
                )
                .await
                {
                    Ok(mut new_post_vec) => {
                        post_vec.update(|post_vec| post_vec.append(&mut new_post_vec))
                    }
                    Err(e) => load_error.set(Some(AppError::from(&e))),
                }
                is_loading.set(false);
            });
        }
    });

    view! {
        <ForumPostMiniatures
            post_vec=post_vec
            is_loading=is_loading
            load_error=load_error
            additional_load_count=additional_load_count
            list_ref=list_ref
        />
    }
}
