use leptos::html;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{components::{Outlet, ParentRoute, Route, Router, Routes}, ParamSegment, StaticSegment};

use sharesphere_utils::error_template::ErrorTemplate;
use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::*;
use sharesphere_utils::unpack::{handle_additional_load, handle_initial_load, SuspenseUnpack};
use sharesphere_utils::routes::{USER_ROUTE_PREFIX, USER_ROUTE_PARAM_NAME, SATELLITE_ROUTE_PARAM_NAME, SATELLITE_ROUTE_PREFIX, SPHERE_ROUTE_PREFIX, SPHERE_ROUTE_PARAM_NAME, POST_ROUTE_PREFIX, POST_ROUTE_PARAM_NAME, PUBLISH_ROUTE, CREATE_POST_SUFFIX, SEARCH_ROUTE, CREATE_SPHERE_SUFFIX};
use sharesphere_auth::auth::*;
use sharesphere_auth::auth_widget::LoginWindow;
use sharesphere_auth::user::{DeleteUser, SetUserSettings, User, UserState};
use sharesphere_components::navigation_bar::NavigationBar;
use sharesphere_components::profile::UserProfile;
use sharesphere_components::search::{Search, SphereSearch};
use sharesphere_content::post::{CreatePost, Post};
use sharesphere_core::post::{get_sorted_post_vec, get_subscribed_post_vec, PostMiniatureList, PostWithSphereInfo};
use sharesphere_core::ranking::PostSortWidget;
use sharesphere_core::sidebar::{HomeSidebar, LeftSidebar};
use sharesphere_core::sphere::CreateSphere;
use sharesphere_core::state::GlobalState;
use sharesphere_sphere::satellite::{CreateSatellitePost, SatelliteBanner, SatelliteContent};
use sharesphere_sphere::sphere::{CreateSphere, SphereBanner, SphereContents};
use sharesphere_sphere::sphere_management::{SphereCockpit, SphereCockpitGuard, MANAGE_SPHERE_ROUTE};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // TODO refine with nonce, add more constraints
                <Meta
                    http_equiv="Content-Security-Policy" 
                    content=move || {
                        // this will insert the CSP with nonce on the server, be empty on client
                        use_nonce().map(|nonce| {
                            format!(
                                "default-src 'self';
                                script-src 'strict-dynamic' 'nonce-{nonce}' 'wasm-unsafe-eval';
                                img-src 'self' https: data:;
                                media-src 'self' https:;
                                frame-src 'self' https:;
                                style-src 'self' 'nonce-{nonce}';
                                connect-src 'self' https: ws://localhost:3001/ ws://127.0.0.1:3001/;"
                            )
                        }).unwrap_or_default()
                    }
                />
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
    let logout_action = ServerAction::<EndSession>::new();
    let delete_user_action = ServerAction::<DeleteUser>::new();
    let create_sphere_action = ServerAction::<CreateSphere>::new();
    let set_settings_action = ServerAction::<SetUserSettings>::new();
    let user = Resource::new(
        move || {
            (
                logout_action.version().get(),
                delete_user_action.version().get(),
                create_sphere_action.version().get(),
                set_settings_action.version().get(),
            )
        },
        move |_| get_user(),
    );
    let user_state = UserState {
        login_action: ServerAction::<Login>::new(),
        user
    };
    let state = GlobalState::new(
        user,
        logout_action,
        delete_user_action,
        create_sphere_action,
        set_settings_action,
    );
    provide_context(user_state);
    provide_context(state);

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
                    <div class="grow flex w-full overflow-hidden min-h-0">
                        <div class="max-2xl:hidden flex flex-col">
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
                            <ParentRoute path=(StaticSegment(SPHERE_ROUTE_PREFIX), ParamSegment(SPHERE_ROUTE_PARAM_NAME)) view=SphereBanner>
                                <ParentRoute path=(StaticSegment(SATELLITE_ROUTE_PREFIX), ParamSegment(SATELLITE_ROUTE_PARAM_NAME)) view=SatelliteBanner>
                                    <Route path=(StaticSegment(POST_ROUTE_PREFIX), ParamSegment(POST_ROUTE_PARAM_NAME)) view=Post/>
                                    <ParentRoute path=StaticSegment(PUBLISH_ROUTE) view=LoginGuard>
                                        <Route path=StaticSegment(CREATE_POST_SUFFIX) view=CreateSatellitePost/>
                                    </ParentRoute>
                                    <Route path=StaticSegment("") view=SatelliteContent/>
                                </ParentRoute>
                                <Route path=(StaticSegment(POST_ROUTE_PREFIX), ParamSegment(POST_ROUTE_PARAM_NAME)) view=Post/>
                                <ParentRoute path=StaticSegment(MANAGE_SPHERE_ROUTE) view=SphereCockpitGuard>
                                    <Route path=StaticSegment("") view=SphereCockpit/>
                                </ParentRoute>
                                <Route path=StaticSegment(SEARCH_ROUTE) view=SphereSearch/>
                                <Route path=StaticSegment("") view=SphereContents/>
                            </ParentRoute>
                            <Route path=(StaticSegment(USER_ROUTE_PREFIX), ParamSegment(USER_ROUTE_PARAM_NAME)) view=UserProfile/>
                            <Route path=StaticSegment(AUTH_CALLBACK_ROUTE) view=AuthCallback/>
                            <ParentRoute path=StaticSegment(PUBLISH_ROUTE) view=LoginGuardHome>
                                <Route path=StaticSegment(CREATE_SPHERE_SUFFIX) view=CreateSphere/>
                                <Route path=StaticSegment(CREATE_POST_SUFFIX) view=CreatePost/>
                            </ParentRoute>
                            <Route path=StaticSegment(SEARCH_ROUTE) view=Search/>
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

/// Login guard with home sidebar
#[component]
fn LoginGuardHome() -> impl IntoView {
    view! {
        <LoginGuard/>
        <div class="max-2xl:hidden">
            <HomeSidebar/>
        </div>
    }
}

/// Component to guard pages requiring a login, and enable the user to login with a redirect
#[component]
fn LoginGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <SuspenseUnpack resource=state.user let:user>
        {
            match user {
                Some(_) => view! { <Outlet/> }.into_any(),
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </SuspenseUnpack>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <div class="flex flex-col flex-1 w-full overflow-y-auto pt-2 px-2 gap-2">
            <div
                class="flex-none bg-cover bg-center bg-no-repeat bg-[url('/banner.jpg')] rounded-sm w-full h-40 flex items-center justify-center"
            >
                <div class="p-3 backdrop-blur-sm bg-black/50 rounded-xs flex justify-center gap-3">
                    <LogoIcon class="h-12 w-12"/>
                    <span class="text-4xl select-none">"ShareSphere"</span>
                </div>
            </div>
            <PostSortWidget sort_signal=state.post_sort_type/>
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
    let post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_post_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let initial_load = get_sorted_post_vec(state.post_sort_type.get(), 0).await;
            handle_initial_load(initial_load, post_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = post_vec.read_untracked().len();
                let additional_load = get_sorted_post_vec(state.post_sort_type.get_untracked(), num_post).await;
                handle_additional_load(additional_load, post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <PostMiniatureList
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
    let post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_post_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let initial_load = get_subscribed_post_vec(user_id, state.post_sort_type.get(), 0).await;
            handle_initial_load(initial_load, post_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = post_vec.read_untracked().len();
                let additional_load = get_subscribed_post_vec(user_id, state.post_sort_type.get_untracked(), num_post).await;
                handle_additional_load(additional_load, post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <PostMiniatureList
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}
