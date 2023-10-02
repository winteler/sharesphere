use leptos::*;

/// Navigation bar component
#[component]
pub fn Drawer(
    ) -> impl IntoView {
    view! {
        <label for="my-drawer" class="drawer-overlay"></label>
        <ul class="menu p-4 w-80 h-full bg-base-200 text-base-content">
            //<!-- Sidebar content here -->
            <li><a>"Sidebar Item 1"</a></li>
            <li><a>"Sidebar Item 2"</a></li>
        </ul>
    }
}