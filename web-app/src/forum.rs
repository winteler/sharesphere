use leptos::*;

pub const CREATE_FORUM_ROUTE : &str = "/forum";

#[component]
pub fn CreateForum(cx: Scope) -> impl IntoView {
    view! { cx,
        <h2 class="p-6 text-4xl">"Create forum"</h2>
    }
}