use crate::icons::HammerIcon;
use leptos::{component, view, IntoView};

/// Displays the body of a moderated post or comment
#[component]
pub fn ModeratedBody(
    infringed_rule_title: String,
    moderator_message: String,
) -> impl IntoView {
    view! {
        <div class="flex items-stretch w-fit">
            <div class="flex justify-center items-center p-2 rounded-l bg-base-content/20">
                <HammerIcon/>
            </div>
            <div class="p-2 rounded-r bg-base-300 whitespace-pre align-middle">
                <div class="flex flex-col gap-1">
                    <div>{moderator_message}</div>
                    <div>{format!("Infringed rule: {infringed_rule_title}")}</div>
                </div>
            </div>
        </div>
    }
}