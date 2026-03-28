use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// Main page for notifications
#[component]
pub fn NotificationHome() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <SuspenseUnpack resource=state.user let:user>
        {
            match user {
                Some(_) => view! { <NotificationList/> }.into_any(),
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </SuspenseUnpack>
        <HomeSidebar/>
    }
}

/// Displays a user's profile
#[component]
pub fn ProfileHome() -> impl IntoView {
    view! {
        <UserProfile/>
        <HomeSidebar/>
    }
}