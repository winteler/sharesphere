use leptos::*;

/// Navigation bar component
#[component]
pub fn Drawer(
    cx: Scope) -> impl IntoView {
    view! { cx,
        <label for="my-drawer" class="drawer-overlay"></label>
        <ul class="menu p-4 w-80 h-full bg-base-200 text-base-content">
            //<!-- Sidebar content here -->
            <li><a>"Sidebar Item 1"</a></li>
            <li><a>"Sidebar Item 2"</a></li>
        </ul>
    }
}

#[component]
pub fn DrawerContent(
    cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="h-full px-6 py-4">
            <h2 class="text-lg font-semibold">"Drawer"</h2>
            <p class="text-gray-500">"This is a drawer."</p>
        </div>
    }
}