use const_format::concatcp;
use leptos::{component, IntoView, RwSignal, Show, SignalGet, view};

use crate::icons::SettingsIcon;

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

/// Component to manage a forum
#[component]
pub fn ModerateCommentButton(
    can_moderate: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || can_moderate.get()>
            <button class="btn btn-circle btn-sm btn-ghost"><SettingsIcon/></button>
        </Show>
    }
}