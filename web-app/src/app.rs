use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::auth::*;
use crate::drawer::*;
use crate::error_template::{AppError, ErrorTemplate};
use crate::forum::*;
use crate::icons::*;
use crate::navigation_bar::*;

pub const PUBLISH_ROUTE : &str = "/publish";

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub user: RwSignal<User>,
    pub logout_action: Action<EndSession, Result<(), ServerFnError>>,
}

impl GlobalState {
    pub fn new(cx: Scope) -> Self {
        Self {
            user: create_rw_signal(cx, User::default()),
            logout_action: create_server_action::<EndSession>(cx)
        }
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    // Provide global context for app
    provide_context(cx, GlobalState::new(cx));

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/start-axum.css"/>

        // sets the document title
        <Title text="Welcome to [[ProjectName]]"/>

        // content for this welcome page
        <Router fallback=|cx| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { cx,
                <ErrorTemplate outside_errors/>
            }
            .into_view(cx)
        }>
            <div class="h-screen flex flex-col">
                <NavigationBar/>
                <main class="h-full drawer lg:drawer-open">
                    <input id="my-drawer" type="checkbox" class="drawer-toggle"/>
                    <div class="drawer-content container mx-auto h-full">
                        <Routes>
                            <Route path="/" view=HomePage/>
                            <Route path=AUTH_CALLBACK_ROUTE view=AuthCallback/>
                            <Route path="/login" view=Login/>
                            // TODO: give path to requested page in ProtectedRoute redirect as parameter
                            <Route path=PUBLISH_ROUTE view=LoginGuard>
                                <Route path=CREATE_FORUM_ROUTE view=CreateForum/>
                            </Route>
                        </Routes>
                    </div>
                    <div class="drawer-side">
                        <Drawer/>
                    </div>
                </main>
                //<Footer/>
            </div>
        </Router>
    }
}

/// Components to guard pages requiring a login, and enable the user to login with a redirect
#[component]
fn LoginGuard(cx: Scope) -> impl IntoView {
    // TODO add check for logged in (resource or context?), display Outlet if authenticated, redirect to auth otherwise

    let state = expect_context::<GlobalState>(cx);
    let user_signal = state.user;

    let auth_resource = create_blocking_resource(
        cx,
        move || {
            (
                state.user.get(),
                state.logout_action.version(),
            )
        },
        move |_| {
            let url = window().location().pathname().unwrap_or(String::from("/"));
            login(cx, url)
        }
    );

    view! { cx,
        <Transition fallback=move || view! { cx, <LoadingIcon/> }>
            { move || {
                    auth_resource.read(cx).map(|user: Result<User, ServerFnError>| match user {
                        Err(e) => {
                            log!("Login error: {}", e);
                            return view! {cx, <div>"Error."</div>}.into_view(cx)
                        },
                        Ok(user) => {
                            if user.anonymous
                            {
                                return view! {cx, <div>"Error."</div>}.into_view(cx);
                            }
                            user_signal.set(user.clone());
                            log!("Current user: {:?}", user);
                            return view! {cx, <Outlet/>}.into_view(cx)
                        },
                    });
                }
            }
        </Transition>
    }
}

/// Renders a page requesting a login
#[component]
fn Login(cx: Scope) -> impl IntoView {
    use crate::navigation_bar::get_current_path_closure;
    let start_auth = create_server_action::<Login>(cx);
    let current_path = create_rw_signal(cx, String::default());
    let get_current_path = get_current_path_closure(current_path);

    view! { cx,
        <div class="h-full my-0 mx-auto max-w-3xl text-center">
            <p class="bg-white px-10 py-10 text-black rounded-lg">"Login required to access this page."</p>
            <form action=start_auth.url() method="post" rel="external">
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
fn HomePage(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    view! { cx,
        <div class="h-full my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
            <p class="bg-white px-10 py-10 text-black rounded-lg">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
            <button
                class="m-8 bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                //class="m-10 btn btn-active btn-accent"
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


