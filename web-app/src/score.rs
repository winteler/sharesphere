use leptos::*;

use crate::auth::{LoginGuardButton};
use crate::icons::{MinusIcon, PlusIcon, ScoreIcon};

/// Component to display a post's score
#[component]
pub fn ScoreIndicator(score: i32) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center text-sm">
            <ScoreIcon/>
            {score}
        </div>
    }
}

/// Component to display and modify post's score
#[component]
pub fn VotePanel<'a>(
    score: &'a RwSignal<i32>,
    #[prop(into)]
    on_up_vote: Callback<ev::MouseEvent>,
    #[prop(into)]
    on_down_vote: Callback<ev::MouseEvent>,
) -> impl IntoView {

    view! {
        <div class="flex items-center gap-1">
            <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle btn-sm hover:btn-success"
                login_button_content=move || view! { <PlusIcon/> }
            >
                <button
                    class="btn btn-ghost btn-circle btn-sm hover:btn-success"
                    on:click=on_up_vote
                >
                    <PlusIcon/>
                </button>
            </LoginGuardButton>
            <ScoreIndicator score=score.get()/>
            <LoginGuardButton
                login_button_class="btn btn-ghost btn-circle btn-sm hover:btn-error"
                login_button_content=move || view! { <MinusIcon/> }
            >
                <button
                    class="btn btn-ghost btn-circle btn-sm hover:btn-error"
                    on:click=on_down_vote
                >
                    <MinusIcon/>
                </button>
            </LoginGuardButton>
        </div>
    }
}