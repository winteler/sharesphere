use leptos::prelude::*;
use leptos_router::components::Form;

use crate::app::GlobalState;
use crate::auth::LoginGuardButton;
use crate::constants::SITE_ROOT;
use crate::icons::*;
use crate::post::{CREATE_POST_ROUTE, CREATE_POST_SPHERE_QUERY_PARAM};
use crate::profile::get_profile_path;
use crate::search::{SearchButton};
use crate::sphere::*;
use crate::user::User;

pub fn get_current_url(url: RwSignal<String>) {
    let url_str = window().location().href().unwrap_or(String::from(SITE_ROOT));
    log::debug!("Current url: {url_str}");
    url.update(|value| *value = url_str);
}

pub fn get_current_path(path: RwSignal<String>) {
    let path_str = window().location().pathname().unwrap_or(String::from(SITE_ROOT));
    log::debug!("Current path: {path_str}");
    path.update(|value | *value = path_str);
}

/// # Extract the sphere name from the current path, if it exists
///
/// ```
/// use crate::app::navigation_bar::get_sphere_from_path;
///
/// assert_eq!(get_sphere_from_path("test"), None);
/// assert_eq!(get_sphere_from_path("/spheres/test"), Some(String::from("test")));
/// ```
pub fn get_sphere_from_path(path: &str) -> Option<String> {
    if path.starts_with(SPHERE_ROUTE_PREFIX) {
        let mut path_part_it = path.split("/");
        Some(String::from(path_part_it.nth(2).unwrap_or("")))
    } else {
        None
    }
}

pub fn get_sphere_name(sphere_name: RwSignal<String>) {
    let path = window().location().pathname().unwrap_or_default();
    sphere_name.update(|name| *name = get_sphere_from_path(&path).unwrap_or_default());
}

pub fn get_create_post_path(create_post_route: RwSignal<String>) {
    let path = window().location().pathname().unwrap_or_default();
    log::debug!("Current path: {path}");

    let current_sphere = get_sphere_from_path(&path);

    if let Some(sphere_name) = current_sphere {
        create_post_route.set(format!("{CREATE_POST_ROUTE}?{CREATE_POST_SPHERE_QUERY_PARAM}={sphere_name}"));
    } else {
        create_post_route.set(String::from(CREATE_POST_ROUTE));
    };
}

/// Navigation bar component
#[component]
pub fn NavigationBar() -> impl IntoView
{
    view! {
        <div class="flex-none navbar bg-blue-500">
            <div class="navbar-start">
                <label for="my-drawer" class="drawer-button 2xl:hidden btn btn-square btn-ghost"><SideBarIcon/></label>
                <div class="flex-1">
                    <a href="/" class="btn btn-ghost text-l">
                        <LogoIcon/>
                        <label class="max-2xl:hidden">"ShareSphere"</label>
                    </a>
                </div>
            </div>
            <div class="navbar-end gap-1">
                <SearchButton/>
                <PlusMenu/>
                <UserMenu/>
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn UserMenu() -> impl IntoView {
    view! {
        <LoginGuardButton
            login_button_class="btn btn-ghost btn-circle rounded-full"
            login_button_content=move || view! { <UserIcon/> }.into_any()
            redirect_path_fn=&get_current_path
            let:user
        >
            <LoggedInMenu user=user.clone()/>
        </LoginGuardButton>
    }.into_any()
}

#[component]
pub fn LoggedInMenu(
    user: User,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_url = RwSignal::new(String::default());

    view! {
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full">
                <UserIcon/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-1 p-2 shadow-sm bg-base-200 rounded-xs w-52">
                <li><a href=get_profile_path(&user.username)>"Profile"</a></li>
                <li>
                    <ActionForm action=state.logout_action attr:class="flex">
                        <input type="text" name="redirect_url" class="hidden" value=current_url/>
                        <button type="submit" class="w-full text-left" on:click=move |_| get_current_url(current_url)>
                            "Logout"
                        </button>
                    </ActionForm>
                </li>
                <li><span>{format!("Logged in as: {}", user.username)}</span></li>
            </ul>
        </div>
    }.into_any()
}

#[component]
pub fn PlusMenu() -> impl IntoView {
    let current_sphere = RwSignal::new(String::default());
    let create_sphere_str = "Settle a Sphere!";
    let create_post_str = "Share a Post!";
    view! {
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full">
                <PlusIcon class="h-6 w-6"/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content z-10 mt-3 p-2 bg-base-200 rounded-sm">
                <li>
                    <LoginGuardButton
                        login_button_content=move || view! { <span class="whitespace-nowrap">{create_sphere_str}</span> }.into_any()
                        redirect_path_fn=&(|redirect_path: RwSignal<String>| redirect_path.set(String::from(CREATE_SPHERE_ROUTE)))
                        let:_user
                    >
                        <a href=CREATE_SPHERE_ROUTE class="whitespace-nowrap">{create_sphere_str}</a>
                    </LoginGuardButton>
                </li>
                <li>
                    <LoginGuardButton
                        login_button_content=move || view! { <span class="whitespace-nowrap">{create_post_str}</span> }.into_any()
                        redirect_path_fn=&get_create_post_path
                        let:_user
                    >
                        <Form method="GET" action=CREATE_POST_ROUTE attr:class="flex">
                            <input type="text" name=CREATE_POST_SPHERE_QUERY_PARAM class="hidden" value=current_sphere/>
                            <button type="submit" class="whitespace-nowrap" on:click=move |_| get_sphere_name(current_sphere)>
                                {create_post_str}
                            </button>
                        </Form>
                    </LoginGuardButton>
                </li>
            </ul>
        </div>
    }.into_any()
}