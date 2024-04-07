use leptos::*;
use leptos_router::Form;

use crate::app::GlobalState;
use crate::auth::*;
use crate::forum::*;
use crate::icons::*;
use crate::post::{CREATE_POST_FORUM_QUERY_PARAM, CREATE_POST_ROUTE};

pub fn get_current_url(url: RwSignal<String>) {
    let url_str = window().location().href().unwrap_or(String::from("/"));
    log::debug!("Current url: {url_str}");
    url.update(|value| *value = url_str);
}

pub fn get_current_path(path: RwSignal<String>) {
    let path_str = window().location().pathname().unwrap_or(String::from("/"));
    log::debug!("Current path: {path_str}");
    path.update(|value | *value = path_str);
}

pub fn get_forum_from_path(path: &String) -> Option<String> {
    if path.starts_with(FORUM_ROUTE_PREFIX) {
        let mut path_part_it = path.split("/");
        Some(String::from(path_part_it.nth(2).unwrap_or("")))
    }
    else {
        None
    }
}

pub fn get_forum_name(forum_name: RwSignal<String>) {
    let path = window().location().pathname().unwrap_or(String::default());
    forum_name.update(|name| *name = get_forum_from_path(&path).unwrap_or_default());
}

pub fn get_create_post_path(create_post_route: RwSignal<String>) {
    let path = window().location().pathname().unwrap_or(String::default());
    log::debug!("Current path: {path}");

    let current_forum = get_forum_from_path(&path);

    if let Some(forum_name) = current_forum {
        create_post_route.update(|value| *value = format!("{CREATE_POST_ROUTE}?{CREATE_POST_FORUM_QUERY_PARAM}={forum_name}"));
    } else {
        create_post_route.update(|value| *value = String::from(CREATE_POST_ROUTE));
    };
}

/// Navigation bar component
#[component]
pub fn NavigationBar(
    ) -> impl IntoView
{
    view! {
        <div class="flex-none navbar bg-blue-500">
            <div class="navbar-start">
                <label for="my-drawer" class="drawer-button 2xl:hidden btn btn-square btn-ghost"><SideBarIcon/></label>
                <div class="flex-1">
                    <a href="/" class="btn btn-ghost normal-case text-l">
                        <LogoIcon/>
                        <label class="max-2xl:hidden">"ShareSphere"</label>
                    </a>
                </div>
            </div>
            <div class="navbar-end gap-1">
                <div class="join max-2xl:hidden">
                    <div>
                        <div>
                            <input class="input join-item input-md" placeholder="Search"/>
                        </div>
                    </div>
                    <button class="btn join-item button-md"><SearchIcon/></button>
                </div>
                <button class="btn btn-ghost btn-circle 2xl:hidden">
                    <SearchIcon/>
                </button>
                <PlusMenu/>
                <UserProfile/>
            </div>
        </div>
    }
}

#[component]
pub fn UserProfile() -> impl IntoView {
    view! {
        <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle rounded-full"
                login_button_content=move || view! { <UserIcon/> }
                let: value
        >
            <LoggedInMenu user=value/>
        </LoginGuardButton>
    }
}

#[component]
pub fn LoggedInMenu<'a>(
    user: &'a User,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_url = create_rw_signal(String::default());

    view! {
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full">
                <UserIcon/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52">
                <li><a href="#">"Settings"</a></li>
                <li>
                    <form action=state.logout_action.url() method="post" rel="external" class="flex">
                        <input type="text" name="redirect_url" class="hidden" value=current_url/>
                        <button type="submit" class="w-full text-left" on:click=move |_| get_current_url(current_url)>
                            "Logout"
                        </button>
                    </form>
                </li>
                <li><span>{format!("Logged in as: {}", user.username)}</span></li>
            </ul>
        </div>
    }
}

#[component]
pub fn PlusMenu() -> impl IntoView {
    let current_forum = create_rw_signal(String::default());
    let create_sphere_str = "Settle a Sphere!";
    let create_post_str = "Share a Post!";
    view! {
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full">
                <PlusIcon class="h-6 w-6"/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box">
                <li>
                    <LoginGuardButton
                        login_button_content=move || view! { <span class="whitespace-nowrap">{create_sphere_str}</span> }
                        redirect_path_fn=&(|redirect_path: RwSignal<String>| redirect_path.update(|value: &mut String| *value = String::from(CREATE_FORUM_ROUTE)))
                    >
                        <a href=CREATE_FORUM_ROUTE class="whitespace-nowrap">{create_sphere_str}</a>
                    </LoginGuardButton>
                </li>
                <li>
                    <LoginGuardButton
                        login_button_content=move || view! { <span class="whitespace-nowrap">{create_post_str}</span> }
                        redirect_path_fn=&get_create_post_path
                    >
                        <Form action=CREATE_POST_ROUTE class="flex">
                            <input type="text" name=CREATE_POST_FORUM_QUERY_PARAM class="hidden" value=current_forum/>
                            <button type="submit" class="whitespace-nowrap" on:click=move |_| get_forum_name(current_forum)>
                                {create_post_str}
                            </button>
                        </Form>
                    </LoginGuardButton>
                </li>
            </ul>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_forum_from_path() {
        assert_eq!(get_forum_from_path(&String::from("test")), None);
        assert_eq!(get_forum_from_path(&String::from("/forums/test")), Some(String::from("test")));
    }
}
