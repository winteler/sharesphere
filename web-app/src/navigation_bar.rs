use leptos::*;
use crate::app::{GlobalState};
use crate::auth::*;

fn get_current_url_closure(url_signal: RwSignal<String>) -> impl FnMut(leptos::ev::MouseEvent) -> () {
    move |_| {
        let url = window().location().href().unwrap_or(String::from("/"));
        url_signal.set(url.clone());
    }
}

fn get_current_path_closure(url_signal: RwSignal<String>) -> impl FnMut(leptos::ev::MouseEvent) -> () {
    move |_| {
        let url = window().location().pathname().unwrap_or(String::from("/"));
        url_signal.set(url.clone());
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
                <label for="my-drawer" class="drawer-button lg:hidden btn btn-square btn-ghost"><SideBarIcon/></label>
                <div class="flex-1">
                    <a href="/" class="btn btn-ghost normal-case text-l text-white">
                        <StacksIcon/>
                        "[[ProjectName]]"
                    </a>
                </div>
            </div>
            <div class="navbar-end gap-1">
                <div class="join sm:max-lg:hidden">
                    <div>
                        <div>
                            <input class="input join-item input-md" placeholder="Search"/>
                        </div>
                    </div>
                    <button class="btn join-item button-md"><SearchIcon/></button>
                </div>
                <button class="btn btn-ghost btn-circle lg:hidden">
                    <SearchIcon/>
                </button>
                <UserProfile/>
            </div>
        </div>
    }
}

#[component]
pub fn UserProfile(cx: Scope) -> impl IntoView {
    use crate::auth::*;
    let state = expect_context::<GlobalState>(cx);
    let logout = create_server_action::<EndSession>(cx);

    let user = create_resource(
        cx,
        move || {
            (
                state.user.get(),
                logout.version().get(),
            )
        },
        move |_| { get_user(cx) },
    );

    view! { cx,
        <Transition fallback=move || view! {cx, <UserIcon/>}>
        {move || {
            user.read(cx).map(|user| match user {
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
    use crate::auth::*;
    let start_auth = create_server_action::<StartAuth>(cx);
    let current_path = create_rw_signal(cx, String::default());
    let get_current_path = get_current_path_closure(current_path);

    view! { cx,
        <form action=start_auth.url() method="post" rel="external">
            <input type="text" name="redirect_url" class="hidden" value=current_path/>
            <button type="submit" class="btn btn-ghost" on:click=get_current_path>
                <UserIcon/>
            </button>
        </form>
    }
}

#[component]
pub fn LoggedInMenu(cx: Scope, user: User) -> impl IntoView {
    use crate::auth::*;

    let end_session = create_server_action::<EndSession>(cx);
    let current_url = create_rw_signal(cx, String::default());
    let get_current_url = get_current_url_closure(current_url);

    view! { cx,
        <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-circle avatar">
                <div class="rounded-full">
                    <UserIcon/>
                </div>
            </label>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52">
                <li><a>"Settings"</a></li>
                <li>
                    <form action=end_session.url() method="post" rel="external">
                        <input type="text" name="redirect_url" class="hidden" value=current_url/>
                        <button type="submit" on:click=get_current_url>
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
pub fn UserIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" width="44" height="44" viewBox="0 0 24 24" fill="none" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-6 w-6 icon icon-tabler icon-tabler-user stroke-white">
              <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
              <path d="M8 7a4 4 0 1 0 8 0a4 4 0 0 0 -8 0" />
              <path d="M6 21v-2a4 4 0 0 1 4 -4h4a4 4 0 0 1 4 4v2" />
        </svg>
    }
}

#[component]
pub fn StacksIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg class="w-7 h-7 text-white p-1 bg-indigo-500 rounded-full"
             fill="none"
             stroke="currentColor"
             stroke-linecap="round"
             stroke-linejoin="round"
             stroke-width="2"
             viewBox="0 0 24 24">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
        </svg>
    }
}

#[component]
pub fn SideBarIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-6 h-6 stroke-white">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
        </svg>
    }
}

#[component]
pub fn SearchIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="h-5 w-5 stroke-white">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
    }
}
