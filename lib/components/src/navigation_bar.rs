use leptos::prelude::*;
use leptos_router::components::Form;

use sharesphere_utils::icons::*;
use sharesphere_utils::routes::{get_create_post_path, get_current_path, get_current_url, get_profile_path, get_sphere_name, CREATE_POST_ROUTE, CREATE_POST_SPHERE_QUERY_PARAM, CREATE_SPHERE_ROUTE};

use sharesphere_auth::auth::LoginGuardButton;
use sharesphere_auth::user::User;
use sharesphere_core::state::GlobalState;
use crate::search::{SearchButton};

/// Navigation bar component
#[component]
pub fn NavigationBar() -> impl IntoView
{
    view! {
        <div class="flex-none navbar bg-blue-500">
            <div class="navbar-start flex gap-1">
                <label
                    for="my-drawer"
                    class="drawer-button 2xl:hidden button-rounded-ghost"
                >
                    <SideBarIcon/>
                </label>
                <a href="/" class="button-ghost text-l font-semibold flex gap-1">
                    <LogoIcon/>
                    <div>"ShareSphere"</div>
                </a>
            </div>
            <div class="navbar-end flex 2xl:gap-1">
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
            login_button_class="button-rounded-ghost"
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
            <button tabindex="0" class="button-rounded-ghost">
                <UserIcon/>
            </button>
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
            <button tabindex="0" class="button-rounded-ghost">
                <PlusIcon/>
            </button>
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