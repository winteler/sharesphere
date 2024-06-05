use const_format::concatcp;
use leptos::{component, create_rw_signal, IntoView, RwSignal, Show, SignalGet, SignalSet, use_context, view};

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
pub fn ModerateButton(
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let user_state = use_context::<UserState>();
    view! {
        <Show when=move || match user_state {
            Some(user_state) => user_state.can_moderate.get(),
            None => false,
        }>
            <button
                class="btn btn-circle btn-sm btn-ghost"
                on:click=move |_| show_dialog.set(true)
            >
                <HammerIcon/>
            </button>
        </Show>
    }
}

/// Component to access a post's moderation dialog
#[component]
pub fn ModeratePostButton() -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <ModerateButton show_dialog/>
    }
}

/// Component to access a comment's moderation dialog
#[component]
pub fn ModerateCommentButton() -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <ModerateButton show_dialog/>
    }
}

/// Dialog to moderate a post
#[component]
pub fn ModeratePostDialog(
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
    }
}

/// Dialog to moderate a comment
#[component]
pub fn ModerateCommentDialog(
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
    }
}

