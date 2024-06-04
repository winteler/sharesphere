use const_format::concatcp;
use leptos::{component, IntoView, RwSignal, Show, SignalGet, view};

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
pub fn ModeratePostButton(
    can_moderate: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || can_moderate.get()>
            <button class="btn btn-circle btn-sm btn-ghost"><HammerIcon/></button>
        </Show>
    }
}

/// Component to moderate a comment
#[component]
pub fn ModerateCommentButton(
    can_moderate: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || can_moderate.get()>
            <button class="btn btn-circle btn-sm btn-ghost"><HammerIcon/></button>
        </Show>
    }
}