use leptos::*;
use serde::{Deserialize, Serialize};

use crate::icons::{ScoreIcon};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};

#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", sqlx(type_name = "user_role", rename_all = "lowercase"))]
pub enum VoteValue {
    Down = -1,
    None = 0,
    Up = 1,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Vote {
    pub vote_id: i64,
    pub creator_id: i64,
    pub comment_id: Option<i64>,
    pub post_id: i64,
    pub value: i16,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct VoteInfo {
    pub vote_id: i64,
    pub value: i16,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use leptos::ServerFnError;
    use sqlx::PgPool;
    use crate::score::{VoteInfo};


    fn get_vote_deltas(
        vote: i16,
        previous_vote: i16,
    ) -> (i32, i32) {

        let score_delta = i32::from(vote - previous_vote);
        let minus_delta = if vote == -1 && previous_vote != -1 {
            1
        } else if vote != -1 && previous_vote == -1 {
            -1
        } else {
            0
        };

        (score_delta, minus_delta)
    }
    pub async fn update_content_score(
        vote: i16,
        post_id: i64,
        comment_id: Option<i64>,
        previous_vote_info: Option<VoteInfo>,
        db_pool: &PgPool,
    ) -> Result<(), ServerFnError> {

        let previous_vote = match previous_vote_info {
            Some(vote_info) => vote_info.value,
            None => 0
        };

        let (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);

        if comment_id.is_some() {
                sqlx::query!(
                "UPDATE comments set score = score + $1, score_minus = score_minus + $2 where comment_id = $3",
                score_delta,
                minus_delta,
                comment_id,
            )
                .execute(db_pool)
                .await?;
        } else {
            sqlx::query!(
                "UPDATE posts set score = score + $1, score_minus = score_minus + $2, scoring_timestamp = CURRENT_TIMESTAMP where post_id = $3",
                score_delta,
                minus_delta,
                post_id,
            )
                .execute(db_pool)
                .await?;
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_get_vote_deltas() {

            let mut vote = 1i16;
            let mut previous_vote = 0i16;
            let (mut score_delta, mut minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (1, 0));

            vote = 0i16;
            previous_vote = 1i16;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (-1, 0));

            vote = -1i16;
            previous_vote = 0i16;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (-1, 1));

            vote = 0i16;
            previous_vote = -1i16;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (1, -1));

            vote = 1i16;
            previous_vote = -1i16;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (2, -1));

            vote = -1i16;
            previous_vote = 1i16;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (-2, 1));
        }
    }
}

#[server]
pub async fn vote_on_content(
    vote_value: i16,
    post_id: i64,
    comment_id: Option<i64>,
    previous_vote_info: Option<VoteInfo>,
) -> Result<Option<Vote>, ServerFnError> {

    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if previous_vote_info.as_ref().is_some_and(|vote_info: &VoteInfo| vote_info.value == vote_value) {
        return Err(ServerFnError::new("Identical to previous vote."));
    }

    // TODO: add unique index to prevent multiple votes by same user

    let vote = if previous_vote_info.is_some() {
        let vote_id = previous_vote_info.as_ref().unwrap().vote_id;
        if vote_value != 0 {
            log::trace!("Update vote {vote_id} with value {vote_value}");
            Some(sqlx::query_as!(
                Vote,
                "UPDATE votes SET value = $1 WHERE vote_id = $2 RETURNING *",
                vote_value,
                vote_id
            )
                .fetch_one(&db_pool)
                .await?)
        } else {
            log::trace!("Delete vote {vote_id}");
            sqlx::query!(
                "DELETE from votes WHERE vote_id = $1",
                vote_id,
            )
                .execute(&db_pool)
                .await?;
            None
        }
    } else {
        log::trace!("Create vote for post {post_id}, comment {:?}, user {} with value {vote_value}", comment_id, user.user_id);
        Some(sqlx::query_as!(
            Vote,
            "INSERT INTO votes (post_id, comment_id, creator_id, value) VALUES ($1, $2, $3, $4) RETURNING *",
            post_id,
            comment_id,
            user.user_id,
            vote_value,
        )
            .fetch_one(&db_pool)
            .await?)
    };

    ssr::update_content_score(vote_value, post_id, comment_id, previous_vote_info, &db_pool).await?;

    Ok(vote)
}

/// Component to display a post's score
#[component]
pub fn ScoreIndicator(score: i32) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <ScoreIcon/>
            <div class="w-3 text-sm text-right">
                {score}
            </div>
        </div>
    }
}

/// Dynamic score indicator, that can be updated through the given signal
#[component]
pub fn DynScoreIndicator(score: RwSignal<i32>) -> impl IntoView {
    view! {
        <div class="flex rounded-btn pr-1 gap-1 items-center">
            <ScoreIcon/>
            <div class="w-3 text-sm text-right">
                {move || score.get()}
            </div>
        </div>
    }
}

// Function to react to an post's upvote or downvote button being clicked.
pub fn get_on_content_vote_closure(
    vote: RwSignal<i16>,
    score: RwSignal<i32>,
    post_id: i64,
    comment_id: Option<i64>,
    initial_score: i32,
    current_vote_id: Option<i64>,
    current_vote_value: Option<i16>,
    vote_action: Action<VoteOnContent, Result<Option<Vote>, ServerFnError>>,
    is_upvote: bool,
) -> impl Fn(ev::MouseEvent) {

    move |_| {
        vote.update(|vote| update_vote_value(vote, is_upvote));

        log::info!("Content vote value {}", vote.get_untracked());

        let previous_vote_info = if vote_action.version().get_untracked() == 0 &&
            current_vote_id.is_some() &&
            current_vote_value.is_some() {
            Some(VoteInfo {
                vote_id: current_vote_id.unwrap(),
                value: current_vote_value.unwrap(),
            })
        } else {
            match vote_action.value().get_untracked() {
                Some(Ok(Some(vote))) => Some(VoteInfo {
                    vote_id: vote.vote_id,
                    value: vote.value,
                }),
                _ => None,
            }
        };

        vote_action.dispatch(VoteOnContent {
            vote_value: vote.get_untracked(),
            post_id,
            comment_id,
            previous_vote_info,
        });
        score.update(|score| *score = initial_score + i32::from(vote.get_untracked()));
    }
}

// Function to obtain the css classes of a vote button
pub fn get_vote_button_css(
    vote: RwSignal<i16>,
    is_upvote: bool,
) -> impl Fn() -> String {
    let (button_css, activated_value) = match is_upvote {
        true => ("btn-success", 1),
        false => ("btn-error", -1),
    };

    move || {
        let vote_value = vote();
        if vote() == activated_value {
            log::info!("Activated vote button, value: {vote_value}, css: {button_css}");
            format!("btn btn-circle btn-sm {button_css}")
        } else {
            log::info!("Deactivated vote button, value: {vote_value}, css: {button_css}");
            format!("btn btn-circle btn-sm btn-ghost hover:{button_css}")
        }
    }
}

pub fn update_vote_value(vote: &mut i16, is_upvote: bool) {
    *vote = match *vote {
        1 => if is_upvote { 0 } else { -1 },
        -1 => if is_upvote { 1 } else { 0 },
        _ => if is_upvote { 1 } else { -1 },
    };
}

#[cfg(test)]
mod tests {
    use crate::score::{update_vote_value};

    #[test]
    fn test_update_vote_value() {
        let mut vote = 12i16;
        update_vote_value(&mut vote, true);
        assert_eq!(vote, 1);
        vote = -20i16;
        update_vote_value(&mut vote, false);
        assert_eq!(vote, -1);
        update_vote_value(&mut vote, true);
        assert_eq!(vote, 1);
        update_vote_value(&mut vote, true);
        assert_eq!(vote, 0);
        update_vote_value(&mut vote, false);
        assert_eq!(vote, -1);
        update_vote_value(&mut vote, false);
        assert_eq!(vote, 0);
        update_vote_value(&mut vote, true);
        assert_eq!(vote, 1);
        update_vote_value(&mut vote, false);
        assert_eq!(vote, -1);
    }
}