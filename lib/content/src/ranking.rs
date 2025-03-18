use leptos::prelude::*;

use sharesphere_utils::icons::{MinusIcon, PlusIcon, ScoreIcon};
use sharesphere_auth::auth::{LoginGuardedButton};
use sharesphere_core::ranking::{Vote, VoteOnContent, VoteValue};

/// Dynamic score indicator, that can be updated through the given signal
#[component]
pub fn DynScoreIndicator(
    #[prop(into)]
    score: Signal<i32>
) -> impl IntoView {
    view! {
        <div class="flex rounded-btn gap-1 items-center">
            <ScoreIcon/>
            <div class="w-fit text-sm text-right">
                {move || score.get()}
            </div>
        </div>
    }.into_any()
}

/// Component to display and modify a content's score
#[component]
pub fn VotePanel(
    post_id: i64,
    comment_id: Option<i64>,
    score: i32,
    vote: Option<Vote>,
) -> impl IntoView {
    let (vote_id, vote_value, initial_score) = match vote {
        Some(vote) => (
            Some(vote.vote_id),
            Some(vote.value),
            score - (vote.value as i32),
        ),
        None => (None, None, score),
    };

    let score = RwSignal::new(score);
    let vote = RwSignal::new(vote_value.unwrap_or(VoteValue::None));

    let vote_action = ServerAction::<VoteOnContent>::new();

    let vote_id = Memo::new(move |current_vote_id| {
        match &(*vote_action.value().read()) {
            Some(Ok(Some(vote))) => Some(vote.vote_id),
            Some(Ok(None)) => None,
            Some(Err(_)) if current_vote_id.is_some() => *current_vote_id.unwrap(),
            _ => vote_id,
        }
    });

    view! {
        <div class="flex items-center gap-1">
            <LoginGuardedButton
                button_class=get_vote_button_css(vote, true)
                button_action=move |_| {
                    on_content_vote(
                        vote,
                        vote_id,
                        score,
                        post_id,
                        comment_id,
                        initial_score,
                        vote_action,
                        true
                    );
                }
            >
                <PlusIcon/>
            </LoginGuardedButton>
            <DynScoreIndicator score=score/>
            <LoginGuardedButton
                button_class=get_vote_button_css(vote, false)
                button_action=move |_| {
                    on_content_vote(
                        vote,
                        vote_id,
                        score,
                        post_id,
                        comment_id,
                        initial_score,
                        vote_action,
                        false
                    );
                }
            >
                <MinusIcon/>
            </LoginGuardedButton>
        </div>
    }.into_any()
}

// Function to react to an post's upvote or downvote button being clicked.
pub fn on_content_vote(
    vote: RwSignal<VoteValue>,
    vote_id: Memo<Option<i64>>,
    score: RwSignal<i32>,
    post_id: i64,
    comment_id: Option<i64>,
    initial_score: i32,
    vote_action: ServerAction<VoteOnContent>,
    is_upvote: bool,
) {
    update_vote_value(&mut vote.write(), is_upvote);

    log::trace!("Content vote value {:?}", vote.get_untracked());

    vote_action.dispatch(VoteOnContent {
        vote_value: vote.get_untracked(),
        post_id,
        comment_id,
        vote_id: vote_id.get_untracked(),
    });
    score.set(initial_score + (vote.get_untracked() as i32));
}

// Function to obtain the css classes of a vote button
pub fn get_vote_button_css(vote: RwSignal<VoteValue>, is_upvote: bool) -> Signal<&'static str> {
    let activated_value = match is_upvote {
        true => VoteValue::Up,
        false => VoteValue::Down,
    };

    Signal::derive(move || match (is_upvote, vote.get() == activated_value) {
        (true, true) => "p-1 rounded-full bg-success",
        (true, false) => "p-1 rounded-full hover:bg-success",
        (false, true) => "p-1 rounded-full bg-error",
        (false, false) => "p-1 rounded-full hover:bg-error",
    })
}

pub fn update_vote_value(vote: &mut VoteValue, is_upvote: bool) {
    *vote = match *vote {
        VoteValue::Up => {
            if is_upvote {
                VoteValue::None
            } else {
                VoteValue::Down
            }
        }
        VoteValue::None => {
            if is_upvote {
                VoteValue::Up
            } else {
                VoteValue::Down
            }
        }
        VoteValue::Down => {
            if is_upvote {
                VoteValue::Up
            } else {
                VoteValue::None
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::ranking::{get_vote_button_css, update_vote_value, VoteValue};
    use leptos::prelude::*;

    #[test]
    fn test_get_vote_button_css() {
        let owner = Owner::new();
        owner.set();
        let vote_signal = RwSignal::new(VoteValue::None);
        let upvote_css = get_vote_button_css(vote_signal, true);
        let downvote_css = get_vote_button_css(vote_signal, false);

        assert_eq!(upvote_css(), String::from("p-1 rounded-full hover:bg-success"));
        assert_eq!(downvote_css(), String::from("p-1 rounded-full hover:bg-error"));

        vote_signal.set(VoteValue::Up);
        assert_eq!(upvote_css(), String::from("p-1 rounded-full bg-success"));
        assert_eq!(downvote_css(), String::from("p-1 rounded-full hover:bg-error"));

        vote_signal.set(VoteValue::Down);
        assert_eq!(upvote_css(), String::from("p-1 rounded-full hover:bg-success"));
        assert_eq!(downvote_css(), String::from("p-1 rounded-full bg-error"));
    }

    #[test]
    fn test_update_vote_value() {
        let mut vote = VoteValue::None;
        update_vote_value(&mut vote, true);
        assert_eq!(vote, VoteValue::Up);
        update_vote_value(&mut vote, true);
        assert_eq!(vote, VoteValue::None);
        update_vote_value(&mut vote, false);
        assert_eq!(vote, VoteValue::Down);
        update_vote_value(&mut vote, true);
        assert_eq!(vote, VoteValue::Up);
        update_vote_value(&mut vote, false);
        assert_eq!(vote, VoteValue::Down);
        update_vote_value(&mut vote, false);
        assert_eq!(vote, VoteValue::None);
    }
}
