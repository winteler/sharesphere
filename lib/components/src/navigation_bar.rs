use leptos::prelude::*;
use leptos_router::components::Form;

use sharesphere_utils::icons::*;
use sharesphere_utils::routes::{get_create_post_path, get_current_path, get_current_url, get_profile_path, get_sphere_name, CREATE_POST_ROUTE, CREATE_POST_SPHERE_QUERY_PARAM, CREATE_SPHERE_ROUTE};

use sharesphere_auth::auth::LoginGuardButton;
use sharesphere_core::state::GlobalState;
use sharesphere_utils::widget::DropdownButton;
use crate::search::{SearchButton};

/// Navigation bar component
#[component]
pub fn NavigationBar() -> impl IntoView
{
    view! {
        <div class="flex-none flex justify-between items-center w-full p-2 bg-blue-500">
            <div class="flex items-center gap-1 2xl:gap-2">
                <label
                    for="my-drawer"
                    class="drawer-button 2xl:hidden button-rounded-neutral"
                >
                    <SideBarIcon/>
                </label>
                <a href="/" class="button-ghost flex gap-1.5 items-center">
                    <LogoIcon/>
                    <div class="2xl:pt-1 2xl:pb-1.5 font-semibold">"ShareSphere"</div>
                </a>
            </div>
            <div class="flex items-center gap-1 2xl:gap-2">
                <SearchButton class="button-rounded-neutral"/>
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
            login_button_class="button-rounded-neutral"
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
    username: String,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_url = RwSignal::new(String::default());

    view! {
        <DropdownButton
            button_content=move || view! { <UserIcon/> }
            align_right=true
        >
            <ul class="mt-4 z-10 p-2 shadow-sm bg-base-200 rounded-sm w-fit flex flex-col">
                <li class="w-full button-ghost-sm"><a href=get_profile_path(&username)>"Profile"</a></li>
                <li class="w-full button-ghost-sm">
                    <ActionForm action=state.logout_action attr:class="flex">
                        <input type="text" name="redirect_url" class="hidden" value=current_url/>
                        <button type="submit" class="text-left" on:click=move |_| get_current_url(current_url)>
                            "Logout"
                        </button>
                    </ActionForm>
                </li>
                <li class="button-ghost-sm"><span>{format!("Logged in as: {}", username)}</span></li>
            </ul>
        </DropdownButton>
    }.into_any()
}

#[component]
pub fn PlusMenu() -> impl IntoView {
    let current_sphere = RwSignal::new(String::default());
    let create_sphere_str = "Settle a Sphere!";
    let create_post_str = "Share a Post!";
    view! {
        <DropdownButton
            button_content=move || view! { <PlusIcon class="navbar-icon-size"/> }
            align_right=true
        >
            <ul class="z-10 mt-4 p-2 bg-base-200 rounded-sm w-fit flex flex-col">
                <li class="button-ghost-sm w-full">
                    <LoginGuardButton
                        login_button_content=move || view! { <span class="whitespace-nowrap">{create_sphere_str}</span> }.into_any()
                        redirect_path_fn=&(|redirect_path: RwSignal<String>| redirect_path.set(String::from(CREATE_SPHERE_ROUTE)))
                        let:_user
                    >
                        <a href=CREATE_SPHERE_ROUTE class="whitespace-nowrap">{create_sphere_str}</a>
                    </LoginGuardButton>
                </li>
                <li class="button-ghost-sm w-full">
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
        </DropdownButton>
    }.into_any()
}