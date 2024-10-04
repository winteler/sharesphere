use leptos::html;
use leptos::prelude::*;
use leptos::spawn::spawn_local;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{components::{Outlet, ParentRoute, Route, Router, Routes}, ParamSegment, StaticSegment};

use crate::auth::*;
use crate::comment::CommentSortType;
use crate::content::PostSortWidget;
use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::forum::*;
use crate::forum_management::{ForumCockpit, ForumCockpitGuard, MANAGE_FORUM_ROUTE};
use crate::icons::*;
use crate::navigation_bar::*;
use crate::post::*;
use crate::ranking::SortType;
use crate::sidebar::*;
use crate::unpack::ArcSuspenseUnpack;
use crate::user::User;

pub const PUBLISH_ROUTE: &str = "/publish";

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub login_action: ServerAction<Login>,
    pub handle_auth_redirect_action: ServerAction<AuthenticateUser>,
    pub logout_action: ServerAction<EndSession>,
    pub subscribe_action: ServerAction<Subscribe>,
    pub unsubscribe_action: ServerAction<Unsubscribe>,
    pub edit_post_action: ServerAction<EditPost>,
    pub create_forum_action: ServerAction<CreateForum>,
    pub post_sort_type: RwSignal<SortType>,
    pub comment_sort_type: RwSignal<SortType>,
    pub user: Resource<Result<Option<User>, ServerFnError>>,
}

impl GlobalState {
    pub fn default() -> Self {
        let handle_auth_redirect_action = ServerAction::<AuthenticateUser>::new();
        let logout_action = ServerAction::<EndSession>::new();
        let create_forum_action = ServerAction::<CreateForum>::new();
        Self {
            login_action: ServerAction::<Login>::new(),
            handle_auth_redirect_action,
            logout_action,
            subscribe_action: ServerAction::<Subscribe>::new(),
            unsubscribe_action: ServerAction::<Unsubscribe>::new(),
            edit_post_action: ServerAction::<EditPost>::new(),
            create_forum_action,
            post_sort_type: RwSignal::new(SortType::Post(PostSortType::Hot)),
            comment_sort_type: RwSignal::new(SortType::Comment(CommentSortType::Best)),
            user: Resource::new(
                move || {
                    (
                        handle_auth_redirect_action.version().get(),
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

    use anyhow::Context;
    use sqlx::{postgres::PgPoolOptions, PgPool};

    use crate::auth::ssr::AuthSession;

    use super::*;

    pub const DB_URL_ENV: &str = "DATABASE_URL";

    pub fn get_session() -> Result<AuthSession, AppError> {
        use_context::<AuthSession>().ok_or_else(|| AppError::new("Auth session missing."))
    }

    pub fn get_db_pool() -> Result<PgPool, AppError> {
        use_context::<PgPool>().ok_or_else(|| AppError::new("DB pool missing."))
    }

    pub async fn create_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var(DB_URL_ENV)?)
            .await
            .with_context(|| format!("Failed to connect to DB"))
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Provide global context for app
    provide_context(GlobalState::default());

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/sharesphere.css"/>

        // sets the document title
        <Title text="Welcome to ShareSphere"/>
        <Router>
            <main class="h-screen text-white">
                <input id="my-drawer" type="checkbox" class="drawer-toggle"/>
                    <div class="drawer-content h-full flex flex-col max-2xl:items-center">
                        <NavigationBar/>
                        <div class="grow flex w-full overflow-hidden">
                            <div class="max-2xl:hidden">
                                <LeftSidebar/>
                            </div>
                            <Routes fallback=|| {
                                let mut outside_errors = Errors::default();
                                outside_errors.insert_with_default_key(AppError::NotFound);
                                view! {
                                    <ErrorTemplate outside_errors/>
                                }
                            }>
                                <Route path=StaticSegment("") view=HomePage/>
                                <ParentRoute path=(StaticSegment(FORUM_ROUTE_PREFIX), ParamSegment(FORUM_ROUTE_PARAM_NAME)) view=ForumBanner>
                                    <Route path=(StaticSegment(POST_ROUTE_PREFIX), ParamSegment(POST_ROUTE_PARAM_NAME)) view=Post/>
                                    <ParentRoute path=StaticSegment(MANAGE_FORUM_ROUTE) view=ForumCockpitGuard>
                                        <Route path=StaticSegment("") view=ForumCockpit/>
                                    </ParentRoute>
                                    <Route path=StaticSegment("") view=ForumContents/>
                                </ParentRoute>
                                <Route path=StaticSegment(AUTH_CALLBACK_ROUTE) view=AuthCallback/>
                                <ParentRoute path=StaticSegment(PUBLISH_ROUTE) view=LoginGuard>
                                    <Route path=StaticSegment(CREATE_FORUM_SUFFIX) view=CreateForum/>
                                    <Route path=StaticSegment(CREATE_POST_SUFFIX) view=CreatePost/>
                                </ParentRoute>
                            </Routes>
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
fn LoginGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <ArcSuspenseUnpack resource=state.user let:user>
        {
            match *user {
                Some(ref user) => {
                    log::debug!("Login guard, current user: {user:?}");
                    view! { <Outlet/> }.into_any()
                },
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </ArcSuspenseUnpack>
        <div class="max-2xl:hidden">
            <HomeSidebar/>
        </div>
    }
}

/// Renders a page requesting a login
#[component]
pub fn LoginWindow() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_path = RwSignal::new(String::default());

    view! {
        <div class="hero">
            <div class="hero-content flex text-center">
                <AuthErrorIcon class="h-44 w-44"/>
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">"Not authenticated"</h1>
                    <p class="pt-4">"Sorry, we had some trouble identifying you."</p>
                    <p class="pb-4">"Please login to access this page."</p>
                    <ActionForm action=state.login_action>
                        <input type="text" name="redirect_url" class="hidden" value=current_path/>
                        <button type="submit" class="btn btn-primary btn-wide rounded" on:click=move |_| get_current_path(current_path)>
                            "Login"
                        </button>
                    </ActionForm>
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
                class="flex-none bg-cover bg-center bg-no-repeat rounded w-full h-28 flex items-center justify-center"
                style="background-image: url(/banner.jpg)"
            >
                <div class="p-3 backdrop-blur bg-black/50 rounded-sm flex justify-center gap-3">
                    <span class="text-4xl select-none">"ShareSphere"</span>
                </div>
            </div>
            <PostSortWidget/>
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                { 
                    move || Suspend::new(async move { 
                        match state.user.await {
                            Ok(Some(user)) => view! { <UserHomePage user/> }.into_any(),
                            _ => view! { <DefaultHomePage/> }.into_any(),
                        }
                    })
                }
            </Transition>
        </div>
        <div class="max-2xl:hidden">
            <HomeSidebar/>
        </div>
    }
}

/// Renders the home page anonymous users.
#[component]
fn DefaultHomePage() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let post_vec = RwSignal::new(Vec::<Post>::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    // Effect for initial load and sort changes
    Effect::new(move |_| {
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
    Effect::new(move |_| {
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
fn UserHomePage(user: User) -> impl IntoView {
    let user_id = user.user_id;
    let state = expect_context::<GlobalState>();
    let post_vec = RwSignal::new(Vec::<Post>::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    // Effect for initial load and sort changes
    Effect::new(move |_| {
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
    Effect::new(move |_| {
        if additional_load_count.get() > 0 {
            is_loading.set(true);
            load_error.set(None);
            let post_count = post_vec.with_untracked(|post_vec| post_vec.len());
            spawn_local(async move {
                match get_subscribed_post_vec(
                    user_id,
                    state.post_sort_type.get_untracked(),
                    post_count,
                ).await {
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
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}
