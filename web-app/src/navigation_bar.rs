use cfg_if::cfg_if;
use leptos::*;
use leptos_router::{A, Form};

use crate::app::{GlobalState};
use crate::auth::*;
use crate::post::{CREATE_POST_FORUM_QUERY_PARAM, CREATE_POST_ROUTE};
use crate::icons::*;
use crate::forum::*;

pub fn get_current_url_closure(url_signal: RwSignal<String>) -> impl FnMut(leptos::ev::MouseEvent) -> () {
    move |_| {
        let url = window().location().href().unwrap_or(String::from("/"));
        log::info!("Current url: {url}");
        url_signal.set(url.clone());
    }
}

pub fn get_current_path_closure(url_signal: RwSignal<String>) -> impl FnMut(leptos::ev::MouseEvent) -> () {
    move |_| {
        let path = window().location().pathname().unwrap_or(String::from("/"));
        log::info!("Current path: {path}");
        url_signal.set(path.clone());
    }
}

/// Navigation bar component
#[component]
pub fn NavigationBar(
    ) -> impl IntoView
{
    view! {
        <div class="navbar bg-blue-500">
            <div class="navbar-start">
                <label for="my-drawer" class="drawer-button 2xl:hidden btn btn-square btn-ghost"><SideBarIcon/></label>
                <div class="flex-1">
                    <a href="/" class="btn btn-ghost normal-case text-l text-white">
                        <StacksIcon/>
                        <label class="max-2xl:hidden">"[[ProjectName]]"</label>
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
    let state = expect_context::<GlobalState>();

    view! {
        <Transition fallback=move || {
            view! {
                <button class="btn btn-ghost btn-circle rounded-full">
                    <UserIcon/>
                </button>
            }
        }>
        {move || {
            state.user_resource.get().map(|user| match user {
                Err(e) => {
                    log::info!("Get user error: {}", e);
                    view! { <LoginButton/> }.into_view()
                },
                Ok(user) => {
                    if user.anonymous
                    {
                        return view! { <LoginButton/> }.into_view();
                    }
                    view! { <LoggedInMenu user=user/> }.into_view()
                },
            })
        }}
        </Transition>
    }
}

#[component]
pub fn LoginButton() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_path = create_rw_signal( String::default());
    let get_current_path = get_current_path_closure(current_path);

    view! {
        <form action=state.login_action.url() method="post" rel="external">
            <input type="text" name="redirect_url" class="hidden" value=current_path/>
            <button type="submit" class="btn btn-ghost btn-circle rounded-full" on:click=get_current_path>
                <UserIcon/>
            </button>
        </form>
    }
}

#[component]
pub fn LoggedInMenu( user: User) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let current_url = create_rw_signal( String::default());
    let get_current_url = get_current_url_closure(current_url);

    view! {
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full avatar">
                <UserIcon/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52">
                <li><a href="#">"Settings"</a></li>
                <li>
                    <form action=state.logout_action.url() method="post" rel="external" class="flex">
                        <input type="text" name="redirect_url" class="hidden" value=current_url/>
                        <button type="submit" class="w-full text-left" on:click=get_current_url>
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
    let get_current_forum = move |_| {
        let path = window().location().pathname().unwrap_or(String::default());
        log::info!("Current path: {path}");
        let mut path_part_it = path.split("/");
        current_forum.update(|forum_name| *forum_name = String::from(path_part_it.nth(2).unwrap_or("")));
    };
    let create_post_route = move || {
        cfg_if! {
            if #[cfg(feature = "ssr")] {
                String::from(CREATE_POST_ROUTE)
            }
            else {
                use crate::app::{PUBLISH_ROUTE};
                use crate::post::{CREATE_POST_SUFFIX};
                let path = window().location().pathname().unwrap_or(String::default());
                log::info!("Current path: {path}");
                let mut path_part_it = path.split("/");
                let forum_name = String::from(path_part_it.nth(2).unwrap_or(""));
                if path.starts_with(&(String::from(FORUM_ROUTE_PREFIX) + "/")) && !forum_name.is_empty() {
                    FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name.as_ref() + PUBLISH_ROUTE + CREATE_POST_SUFFIX
                }
                else {
                    String::from(CREATE_POST_ROUTE)
                }
            }
        }
    };

    view! {
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full avatar">
                <PlusIcon/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box">
                <li><a href=CREATE_FORUM_ROUTE>"[[Forum]]"</a></li>
                <li><A href=create_post_route>"[[Post-1]]"</A></li>
                <li>
                    <Form action=CREATE_POST_ROUTE class="flex">
                        <input type="text" name=CREATE_POST_FORUM_QUERY_PARAM class="hidden" value=current_forum/>
                        <button type="submit" on:click=get_current_forum class="w-full text-left">
                            "[[Post]]"
                        </button>
                    </Form>
                </li>
            </ul>
        </div>
    }
}


