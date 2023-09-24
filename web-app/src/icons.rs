use leptos::*;

/// Renders a loading icon
#[component]
pub fn LoadingIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <span class="loading loading-spinner loading-md"></span>
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
pub fn PlusIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 stroke-white" width="44" height="44" viewBox="0 0 24 24" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12 5l0 14" />
            <path d="M5 12l14 0" />
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

#[component]
pub fn ErrorIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
    }
}