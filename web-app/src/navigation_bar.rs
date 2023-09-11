use leptos::*;

use crate::app::{GlobalState, PUBLISH_ROUTE};
use crate::auth::*;
use crate::icons::*;
use crate::forum::*;

pub fn get_current_url_closure(url_signal: RwSignal<String>) -> impl FnMut(leptos::ev::MouseEvent) -> () {
    move |_| {
        let url = window().location().href().unwrap_or(String::from("/"));
        log!("Current url: {url}");
        url_signal.set(url.clone());
    }
}

pub fn get_current_path_closure(url_signal: RwSignal<String>) -> impl FnMut(leptos::ev::MouseEvent) -> () {
    move |_| {
        let path = window().location().pathname().unwrap_or(String::from("/"));
        log!("Current path: {path}");
        url_signal.set(path.clone());
    }
}

/// Navigation bar component
#[component]
pub fn NavigationBar(
    cx: Scope) -> impl IntoView
{
    view! { cx,
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
pub fn UserProfile(cx: Scope) -> impl IntoView {
    let user_resource = get_user_resource(cx);

    view! { cx,
        <Transition fallback=move || {
            view! {cx,
                <button class="btn btn-ghost btn-circle rounded-full">
                    <UserIcon/>
                </button>
            }
        }>
        {move || {
            user_resource.read(cx).map(|user| match user {
                Err(e) => {
                    log!("Login error: {}", e);
                    view! {cx, <LoginButton/>}.into_view(cx)
                },
                Ok(user) => {
                    if user.anonymous
                    {
                        return view! {cx, <LoginButton/>}.into_view(cx);
                    }
                    view! {cx, <LoggedInMenu user=user/>}.into_view(cx)
                },
            })
        }}
        </Transition>
    }
}

#[component]
pub fn LoginButton(cx: Scope) -> impl IntoView {
    let state = expect_context::<GlobalState>(cx);
    let current_path = create_rw_signal(cx, String::default());
    let get_current_path = get_current_path_closure(current_path);

    view! { cx,
        <form action=state.login_action.url() method="post" rel="external">
            <input type="text" name="redirect_url" class="hidden" value=current_path/>
            <button type="submit" class="btn btn-ghost btn-circle rounded-full" on:click=get_current_path>
                <UserIcon/>
            </button>
        </form>
    }
}

#[component]
pub fn LoggedInMenu(cx: Scope, user: User) -> impl IntoView {
    let state = expect_context::<GlobalState>(cx);
    let current_url = create_rw_signal(cx, String::default());
    let get_current_url = get_current_url_closure(current_url);

    view! { cx,
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
pub fn PlusMenu(cx: Scope) -> impl IntoView {

    let create_forum_route = PUBLISH_ROUTE.to_owned() + CREATE_FORUM_ROUTE;

    view! { cx,
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle rounded-full avatar">
                <PlusIcon/>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box">
                <li><a href=create_forum_route>"[[Forum]]"</a></li>
                <li><a href="#">"[[Content]]"</a></li>
            </ul>
        </div>
    }
}


