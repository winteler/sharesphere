use leptos::*;

use crate::auth::{LoginGuardButton};
use crate::icons::{MinusIcon, PlusIcon, ScoreIcon};

/// Component to display a post's score
#[component]
pub fn ScoreIndicator(score: i32) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <ScoreIcon/>
            {score}
        </div>
    }
}

/// Component to display and modify post's score
#[component]
pub fn VotePanel(score: i32) -> impl IntoView {
    view! {
        <div class="flex items-center gap-1">
            <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle btn-sm hover:btn-success"
                login_button_content=move || view! { <PlusIcon/> }
            >
                <button class="btn btn-ghost btn-circle btn-sm hover:btn-success">
                    <PlusIcon/>
                </button>
            </LoginGuardButton>
            <ScoreIndicator score=score/>
            <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle btn-sm hover:btn-error"
                login_button_content=move || view! { <MinusIcon/> }
            >
                <button class="btn btn-ghost btn-circle btn-sm hover:btn-error">
                    <MinusIcon/>
                </button>
            </LoginGuardButton>
        </div>
    }
}