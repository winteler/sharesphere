use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::auth::*;
use crate::comment::CreateComment;
use crate::post::*;
use crate::sidebar::*;
use crate::errors::AppError;
use crate::error_template::{ErrorTemplate};
use crate::forum::*;
use crate::icons::*;
use crate::navigation_bar::*;

pub const PARAM_ROUTE_PREFIX : &str = "/:";
pub const PUBLISH_ROUTE : &str = "/publish";

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub login_action: Action<Login, Result<User, ServerFnError>>,
    pub logout_action: Action<EndSession, Result<(), ServerFnError>>,
    pub create_forum_action: Action<CreateForum, Result<(), ServerFnError>>,
    pub create_post_action: Action<CreatePost, Result<(), ServerFnError>>,
    pub create_comment_action: Action<CreateComment, Result<(), ServerFnError>>,
    pub current_forum_name: Option<Memo<String>>,
    pub current_post_id: Option<Memo<i64>>,
    pub user: Resource<(), Result<User, ServerFnError>>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            login_action: create_server_action::<Login>(),
            logout_action: create_server_action::<EndSession>(),
            create_forum_action: create_server_action::<CreateForum>(),
            create_post_action: create_server_action::<CreatePost>(),
            create_comment_action: create_server_action::<CreateComment>(),
            current_forum_name: None,
            current_post_id: None,
            user: create_local_resource(
                move || (),
                move |_| get_user(),
            ),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use anyhow::Context;
    use cfg_if::cfg_if;
    use sqlx::{PgPool, postgres::{PgPoolOptions}};
    use std::env;
    use crate::auth::ssr::AuthSession;
    use super::*;

    pub const DB_URL_ENV : &str = "DATABASE_URL";

    pub fn get_session() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>().ok_or_else(|| ServerFnError::new("Auth session missing."))
    }

    pub async fn create_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var(DB_URL_ENV)?)
            .await
            .with_context(|| format!("Failed to connect to DB"))
    }

    cfg_if! {
        if #[cfg(feature = "server")] {
            use std::sync::OnceLock;
            use tokio::runtime::Handle;

            struct DbPoolGetter {
                pool: Result<PgPool, ServerFnError>,
            }

            impl DbPoolGetter {
                fn new() -> Self {
                    // Create the runtime
                    let handle = Handle::current();
                    let pool = std::thread::spawn(move || {
                        // Using Handle::block_on to run async code in the new thread.
                        handle.block_on(async {
                            create_db_pool().await.or_else(|_| Err(ServerFnError::new("Pool missing.")))
                        })
                    }).join().expect("Failed to create DB pool.");

                    Self {
                        pool: pool,
                    }
                }
}

            static POOL: OnceLock<DbPoolGetter> = OnceLock::new();

            pub fn get_db_pool() -> Result<PgPool, ServerFnError> {
                POOL.get_or_init(|| DbPoolGetter::new()).pool.clone()
            }
        } else {
            pub fn get_db_pool() -> Result<PgPool, ServerFnError> {
                use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Pool missing."))
            }
        }
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
        <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css"/>

        // sets the document title
        <Title text="Welcome to [[ProjectName]]"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main class="h-screen drawer 2xl:drawer-close">
                <input id="my-drawer" type="checkbox" class="drawer-toggle"/>
                <div class="drawer-content h-full flex flex-col max-2xl:items-center">
                    <NavigationBar/>
                    <div class="flex h-full w-full">
                        <div class="h-full max-2xl:hidden">
                            <LeftSidebar/>
                        </div>
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path=FORUM_ROUTE view=ForumBanner>
                                <Route path=POST_ROUTE view=Post/>
                                <Route path="" view=ForumContents/>
                            </Route>
                            <Route path=AUTH_CALLBACK_ROUTE view=AuthCallback/>
                            <Route path="/login" view=Login/>
                            <Route path=PUBLISH_ROUTE view=LoginPageGuard>
                                <Route path=CREATE_FORUM_SUFFIX view=CreateForum/>
                                <Route path=CREATE_POST_SUFFIX view=CreatePost/>
                            </Route>
                        </Routes>
                        <div class="h-full max-2xl:hidden">
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
                        Some(Ok(user)) => {
                            if user.anonymous
                            {
                                log::info!("Not logged in.");
                                return view! { <Login/> }.into_view();
                            }
                            log::info!("Login guard, current user: {:?}", user);
                            view! { <Outlet/> }.into_view()
                        },
                        Some(Err(e)) => {
                            log::info!("Login error: {}", e);
                            view! { <Login/> }.into_view()
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
fn Login() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_path = create_rw_signal( String::default());
    let get_current_path = get_current_path_closure(current_path);

    view! {
        <div class="my-0 mx-auto max-w-3xl text-center">
            <p class="bg-white px-10 py-10 text-black rounded-lg">"Login required to access this page."</p>
            <form action=state.login_action.url() method="post" rel="external">
                <input type="text" name="redirect_url" class="hidden" value=current_path/>
                <button type="submit" class="btn btn-primary" on:click=get_current_path>
                    "Login"
                </button>
            </form>
        </div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let (count, set_count) = create_signal( 0);

    view! {
        <div class="my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
            <p class="bg-white px-10 py-10 text-black rounded-lg">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
            <button
                class="m-8 bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                on:click=move |_| set_count.update(|count| *count += 1)
            >
                "Something's here | "
                {
                    move || if count() == 0 {
                        "Click me!".to_string()
                    } else {
                        count().to_string()
                    }
                }
                " | Some more text"
            </button>
        </div>
    }
}


