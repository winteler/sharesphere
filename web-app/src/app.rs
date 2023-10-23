use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::auth::*;
use crate::post::*;
use crate::drawer::*;
use crate::error_template::{AppError, ErrorTemplate};
use crate::forum::*;
use crate::icons::*;
use crate::navigation_bar::*;

pub const PARAM_ROUTE_PREFIX : &str = "/:";
pub const PUBLISH_ROUTE : &str = "/publish";

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub user: RwSignal<User>,
    pub login_action: Action<Login, Result<User, ServerFnError>>,
    pub logout_action: Action<EndSession, Result<(), ServerFnError>>,
    pub create_forum_action: Action<CreateForum, Result<(), ServerFnError>>,
    pub create_post_action: Action<CreatePost, Result<(), ServerFnError>>,
    pub user_resource: Resource<(), Result<User, ServerFnError>>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            user: create_rw_signal( User::default()),
            login_action: create_server_action::<Login>(),
            logout_action: create_server_action::<EndSession>(),
            create_forum_action: create_server_action::<CreateForum>(),
            create_post_action: create_server_action::<CreatePost>(),
            user_resource: create_blocking_resource(
                move || (),
                move |_| { get_user() },
            ),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Provide global context for app
    provide_context( GlobalState::new());

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/start-axum.css"/>

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
            <div class="h-screen flex flex-col">
                <NavigationBar/>
                <main class="h-full drawer 2xl:drawer-open">
                    <input id="my-drawer" type="checkbox" class="drawer-toggle"/>
                    <div class="drawer-content flex flex-col p-2 max-2xl:items-center h-full">
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path=FORUM_ROUTE view=ForumBanner>
                                <Route path="contents/:id" view=Content/>
                                <Route path="" view=ForumContents/>
                            </Route>
                            <Route path=AUTH_CALLBACK_ROUTE view=AuthCallback/>
                            <Route path="/login" view=Login/>
                            <Route path=PUBLISH_ROUTE view=LoginGuard>
                                <Route path=CREATE_FORUM_SUFFIX view=CreateForum/>
                                <Route path=CREATE_POST_SUFFIX view=CreatePost/>
                            </Route>
                        </Routes>
                    </div>
                    <div class="drawer-side">
                        <Drawer/>
                    </div>
                </main>
            </div>
        </Router>
    }
}

/// Component to guard pages requiring a login, and enable the user to login with a redirect
#[component]
fn LoginGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            { move || {
                     state.user_resource.get().map(|user: Result<User, ServerFnError>| match user {
                        Err(e) => {
                            log::info!("Login error: {}", e);
                            view! { <Login/> }.into_view()
                        },
                        Ok(user) => {
                            if user.anonymous
                            {
                                log::info!("Not logged in.");
                                return view! { <Login/> }.into_view();
                            }
                            log::info!("Login guard, current user: {:?}", user);
                            view! { <Outlet/> }.into_view()
                        },
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
        <div class="h-full my-0 mx-auto max-w-3xl text-center">
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
        <div class="h-full my-0 mx-auto max-w-3xl text-center">
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


