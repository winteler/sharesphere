use const_format::concatcp;
use leptos::{component, IntoView, Show, SignalGet, use_context, view};

use crate::app::UserState;
use crate::icons::HammerIcon;

pub const MANAGE_FORUM_SUFFIX: &str = "manage";
pub const MANAGE_FORUM_ROUTE: &str = concatcp!("/", MANAGE_FORUM_SUFFIX);

/// Component to manage a forum
#[component]
pub fn ForumCockpit() -> impl IntoView {
    view! {
        <div>
            "Forum Cockpit"
        </div>
    }
}

/// Component to moderate a post
#[component]
pub fn ModeratePostButton() -> impl IntoView {
    let user_state = use_context::<UserState>();
    view! {
        <Show when=move || match user_state {
            Some(user_state) => user_state.can_moderate.get(),
            None => false,
        }>
            <button class="btn btn-circle btn-sm btn-ghost"><HammerIcon/></button>
        </Show>
    }
}

/// Component to moderate a comment
#[component]
pub fn ModerateCommentButton() -> impl IntoView {
    let user_state = use_context::<UserState>();
    view! {
        <Show when=move || match user_state {
            Some(user_state) => user_state.can_moderate.get(),
            None => false,
        }>
            <button class="btn btn-circle btn-sm btn-ghost"><HammerIcon/></button>
        </Show>
    }
}