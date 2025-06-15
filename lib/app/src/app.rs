use leptos::ev::TouchEvent;
use leptos::html::Div;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{components::{Outlet, ParentRoute, Route, Router, Routes}, ParamSegment, StaticSegment};

use sharesphere_utils::error_template::ErrorTemplate;
use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::*;
use sharesphere_utils::unpack::{handle_additional_load, reset_additional_load, SuspenseUnpack};
use sharesphere_utils::routes::{USER_ROUTE_PREFIX, USER_ROUTE_PARAM_NAME, SATELLITE_ROUTE_PARAM_NAME, SATELLITE_ROUTE_PREFIX, SPHERE_ROUTE_PREFIX, SPHERE_ROUTE_PARAM_NAME, POST_ROUTE_PREFIX, POST_ROUTE_PARAM_NAME, PUBLISH_ROUTE, CREATE_POST_SUFFIX, SEARCH_ROUTE, CREATE_SPHERE_SUFFIX, TERMS_AND_CONDITIONS_ROUTE, PRIVACY_POLICY_ROUTE, RULES_ROUTE, CONTENT_POLICY_ROUTE, ABOUT_SHARESPHERE_ROUTE};
use sharesphere_auth::auth::*;
use sharesphere_auth::auth_widget::LoginWindow;
use sharesphere_auth::user::{DeleteUser, SetUserSettings, User, UserState};
use sharesphere_components::policy::{AboutShareSphere, ContentPolicy, PrivacyPolicy, Rules, TermsAndConditions};
use sharesphere_components::navigation_bar::NavigationBar;
use sharesphere_components::profile::UserProfile;
use sharesphere_components::search::{Search, SphereSearch};
use sharesphere_content::post::{CreatePost, Post};
use sharesphere_core::post::{get_sorted_post_vec, get_subscribed_post_vec, PostListWithInitLoad, PostWithSphereInfo, POST_BATCH_SIZE};
use sharesphere_core::ranking::PostSortWidget;
use sharesphere_core::sidebar::{HomeSidebar, LeftSidebar};
use sharesphere_core::sphere::CreateSphere;
use sharesphere_core::state::GlobalState;
use sharesphere_sphere::satellite::{CreateSatellitePost, SatelliteBanner, SatelliteContent};
use sharesphere_sphere::sphere::{CreateSphere, SphereBanner, SphereContents};
use sharesphere_sphere::sphere_management::{SphereCockpit, SphereCockpitGuard, MANAGE_SPHERE_ROUTE};
use sharesphere_utils::node_utils::is_fully_scrolled;
use sharesphere_utils::widget::{BannerContent, RefreshButton};

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

    let swipe_start_x = RwSignal::new(None);
    let swipe_start_y = RwSignal::new(None);
    let swipe_id = RwSignal::new(None);

    let on_touch_start = move |ev: TouchEvent| {
        if let Some(touch) = ev.touches().item(0) {
            swipe_start_x.set(Some(touch.client_x()));
            swipe_start_y.set(Some(touch.client_y()));
            swipe_id.set(Some(touch.identifier()));
        }
    };
    let on_touch_end = move |ev: TouchEvent| {
        log::debug!("Touch end. {:?}, {:?}, {:?}", swipe_start_x.get_untracked(), swipe_start_y.get_untracked(), swipe_id.get_untracked());
        if let Some(touch) = ev.changed_touches().item(0) {
            log::debug!("Touch x: {}, touch y: {}, touch id: {}", touch.client_x(), touch.client_x(), touch.identifier());
            if swipe_id.get_untracked().is_some_and(|swipe_id| swipe_id == touch.identifier()) {
                let threshold = 50;
                let delta_x = touch.client_x() - swipe_start_x.get_untracked().unwrap_or(touch.client_x());
                let delta_y = touch.client_y() - swipe_start_y.get_untracked().unwrap_or(touch.client_y());
                match (delta_x, delta_y) {
                    (delta_x, delta_y) if delta_x < -threshold && delta_y.abs() < threshold => {
                        log::debug!("Swipe left: delta_x = {delta_x}, delta_y = {delta_y}");
                        handle_left_swipe(state.show_left_sidebar, state.show_right_sidebar);
                    },
                    (delta_x, delta_y) if delta_x > threshold && delta_y.abs() < threshold => {
                        log::debug!("Swipe right: delta_x = {delta_x}, delta_y = {delta_y}");
                        handle_right_swipe(state.show_left_sidebar, state.show_right_sidebar);
                    },
                    _ => log::debug!("No swipe: delta_x = {delta_x}, delta_y = {delta_y}"),
                }
            }
        }
        swipe_start_x.set(None);
        swipe_start_y.set(None);
        swipe_id.set(None);
    };

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/sharesphere.css"/>

        // sets the document title
        <Title text="Welcome to ShareSphere"/>
        <Router>
            <main
                class="h-screen w-screen overflow-hidden text-white relative"
                on:touchstart=on_touch_start
                on:touchend=on_touch_end
            >
                <div class="h-full flex flex-col max-2xl:items-center">
                    <NavigationBar/>
                    <div class="grow flex w-full overflow-hidden min-h-0">
                        <LeftSidebar/>
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
                            <Route path=StaticSegment(ABOUT_SHARESPHERE_ROUTE) view=AboutShareSphere/>
                            <Route path=StaticSegment(TERMS_AND_CONDITIONS_ROUTE) view=TermsAndConditions/>
                            <Route path=StaticSegment(PRIVACY_POLICY_ROUTE) view=PrivacyPolicy/>
                            <Route path=StaticSegment(CONTENT_POLICY_ROUTE) view=ContentPolicy/>
                            <Route path=StaticSegment(RULES_ROUTE) view=Rules/>
                        </Routes>
                    </div>
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
        <HomeSidebar/>
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
    let refresh_count = RwSignal::new(0);
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let div_ref = NodeRef::<Div>::new();

    view! {
        <div
            class="flex flex-col flex-1 w-full overflow-x-hidden overflow-y-auto px-2"
            on:scroll=move |_| if is_fully_scrolled(div_ref) && !is_loading.get_untracked() {
                additional_load_count.update(|value| *value += 1);
            }
            node_ref=div_ref
        >
            <div class="relative flex-none rounded-sm w-full h-16 2xl:h-32 mt-2 flex items-center justify-center">
                <BannerContent title="ShareSphere" icon_url=None banner_url=None/>
            </div>
            <div class="sticky top-0 bg-base-100 py-2 flex justify-between items-center">
                <PostSortWidget sort_signal=state.post_sort_type is_tooltip_bottom=true/>
                <RefreshButton refresh_count is_tooltip_bottom=true/>
            </div>
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                {
                    move || Suspend::new(async move {
                        match state.user.await {
                            Ok(Some(user)) => view! { <UserHomePage user refresh_count additional_load_count is_loading div_ref/> }.into_any(),
                            _ => view! { <DefaultHomePage refresh_count additional_load_count is_loading div_ref/> }.into_any(),
                        }
                    })
                }
            </Transition>
        </div>
        <HomeSidebar/>
    }
}

/// Renders the home page anonymous users.
#[component]
fn DefaultHomePage(
    refresh_count: RwSignal<usize>,
    additional_load_count: RwSignal<i64>,
    is_loading: RwSignal<bool>,
    div_ref: NodeRef<Div>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let additional_post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let load_error = RwSignal::new(None);

    let post_vec_resource = Resource::new(
        move || (state.post_sort_type.get(), refresh_count.get()),
        move |(sort_type, _)| async move {
            #[cfg(feature = "hydrate")]
            is_loading.set(true);
            reset_additional_load(additional_post_vec, additional_load_count, Some(div_ref));
            let result = get_sorted_post_vec(sort_type, 0).await;
            #[cfg(feature = "hydrate")]
            is_loading.set(false);
            result
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = (POST_BATCH_SIZE as usize) + additional_post_vec.read_untracked().len();
                let additional_load = get_sorted_post_vec(state.post_sort_type.get_untracked(), num_post).await;
                handle_additional_load(additional_load, additional_post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <PostListWithInitLoad
            post_vec_resource
            additional_post_vec
            is_loading=is_loading
            load_error=load_error
            add_y_overflow_auto=false
        />
    }
}

/// Renders the home page of a given user.
#[component]
fn UserHomePage(
    user: User,
    refresh_count: RwSignal<usize>,
    additional_load_count: RwSignal<i64>,
    is_loading: RwSignal<bool>,
    div_ref: NodeRef<Div>,
) -> impl IntoView {
    let user_id = user.user_id;
    let state = expect_context::<GlobalState>();
    let additional_post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let load_error = RwSignal::new(None);

    let post_vec_resource = Resource::new(
        move || (state.post_sort_type.get(), refresh_count.get()),
        move |(sort_type, _)|  async move {
            #[cfg(feature = "hydrate")]
            is_loading.set(true);
            reset_additional_load(additional_post_vec, additional_load_count, Some(div_ref));
            let result = get_subscribed_post_vec(user_id, sort_type, 0).await;
            #[cfg(feature = "hydrate")]
            is_loading.set(false);
            result
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = (POST_BATCH_SIZE as usize) + additional_post_vec.read_untracked().len();
                let additional_load = get_subscribed_post_vec(user_id, state.post_sort_type.get_untracked(), num_post).await;
                handle_additional_load(additional_load, additional_post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <PostListWithInitLoad
            post_vec_resource
            additional_post_vec
            is_loading
            load_error
            add_y_overflow_auto=false
        />
    }
}

fn handle_right_swipe(
    show_left_sidebar: RwSignal<bool>,
    show_right_sidebar: RwSignal<bool>,
) {
    if show_right_sidebar.get_untracked() {
        show_right_sidebar.set(false);
    } else if !show_left_sidebar.get_untracked() {
        show_left_sidebar.set(true);
    }
}

fn handle_left_swipe(
    show_left_sidebar: RwSignal<bool>,
    show_right_sidebar: RwSignal<bool>,
) {
    if show_left_sidebar.get_untracked() {
        show_left_sidebar.set(false);
    } else if !show_right_sidebar.get_untracked() {
        show_right_sidebar.set(true);
    }
}
