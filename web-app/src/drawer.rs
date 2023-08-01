use leptos::*;

/// Navigation bar component
#[component]
pub fn Drawer(
    cx: Scope,
    show_sidebar: ReadSignal<bool>) -> impl IntoView {
    view! { cx,
        <input type="checkbox" id="drawer-toggle" class="lg:hidden relative sr-only peer" checked=show_sidebar/>

        <div class="hidden lg:block h-full bg-white shadow-lg w-64">
            <DrawerContent/>
        </div>
        <div class="lg:hidden fixed left-0 z-20 w-64 h-full transition-all duration-500 transform -translate-x-full bg-white shadow-lg peer-checked:translate-x-0">
            <DrawerContent/>
        </div>

    }
}

#[component]
pub fn DrawerContent(
    cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="h-full px-6 py-4">
            <h2 class="text-lg font-semibold">Drawer</h2>
            <p class="text-gray-500">This is a drawer.</p>
        </div>
    }
}