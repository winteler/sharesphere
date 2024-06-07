use std::fmt;

use leptos::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::app::ssr::get_db_pool;
use crate::auth::LoginGuardButton;
#[cfg(feature = "ssr")]
use crate::auth::ssr::check_user;
use crate::comment::CommentSortType;
use crate::icons::{MinusIcon, PlusIcon, ScoreIcon};
use crate::post::PostSortType;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[repr(i16)]
pub enum VoteValue {
    Up = 1,
    None = 0,
    Down = -1,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SortType {
    Post(PostSortType),
    Comment(CommentSortType),
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Vote {
    pub vote_id: i64,
    pub user_id: i64,
    pub comment_id: Option<i64>,
    pub post_id: i64,
    pub value: VoteValue,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct VoteInfo {
    pub vote_id: i64,
    pub value: VoteValue,
}

impl From<i16> for VoteValue {
    fn from(value: i16) -> VoteValue {
        if value > 0 {
            VoteValue::Up
        } else if value == 0 {
            VoteValue::None
        } else {
            VoteValue::Down
        }
    }
}

impl fmt::Display for SortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sort_type_name = match self {
            SortType::Post(post_sort_type) => post_sort_type.to_string(),
            SortType::Comment(comment_sort_type) => comment_sort_type.to_string(),
        };
        write!(f, "{sort_type_name}")
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::auth::User;
    use crate::errors::AppError;
    use crate::ranking::{SortType, Vote, VoteInfo, VoteValue};

    impl SortType {
        pub fn to_order_by_code(self) -> &'static str {
            match self {
                SortType::Post(post_sort_type) => post_sort_type.to_order_by_code(),
                SortType::Comment(comment_sort_type) => comment_sort_type.to_order_by_code(),
            }
        }
    }

    fn get_vote_deltas(vote: VoteValue, previous_vote: VoteValue) -> (i32, i32) {
        let score_delta = (vote as i32) - (previous_vote as i32);
        let minus_delta = if vote == VoteValue::Down && previous_vote != VoteValue::Down {
            1
        } else if vote != VoteValue::Down && previous_vote == VoteValue::Down {
            -1
        } else {
            0
        };

        (score_delta, minus_delta)
    }
    async fn update_content_score(
        vote: VoteValue,
        post_id: i64,
        comment_id: Option<i64>,
        previous_vote_info: Option<VoteInfo>,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        let previous_vote = match previous_vote_info {
            Some(vote_info) => vote_info.value,
            None => VoteValue::None,
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

    pub async fn vote_on_content(
        vote_value: VoteValue,
        post_id: i64,
        comment_id: Option<i64>,
        previous_vote_info: Option<VoteInfo>,
        user: &User,
        db_pool: PgPool,
    ) -> Result<Option<Vote>, AppError> {
        if previous_vote_info
            .as_ref()
            .is_some_and(|vote_info: &VoteInfo| vote_info.value == vote_value)
        {
            return Err(AppError::new("Identical to previous vote."));
        }

        let vote = if previous_vote_info.is_some() {
            let vote_id = previous_vote_info.as_ref().unwrap().vote_id;
            if vote_value != VoteValue::None {
                log::debug!("Update vote {vote_id} with value {vote_value:?}");
                println!("Update vote {vote_id} where post_id = {post_id}, comment_id = {comment_id:?}, user_id = {}, with value {}", user.user_id, vote_value as i16);
                Some(
                    sqlx::query_as!(
                        Vote,
                        "UPDATE votes SET value = $1 \
                        WHERE vote_id = $2 AND \
                              post_id = $3 AND \
                              user_id = $4 \
                        RETURNING *",
                        vote_value as i16,
                        vote_id,
                        post_id,
                        //comment_id, TODO where condition on nullable field is causing issue
                        user.user_id,
                    )
                    .fetch_one(&db_pool)
                    .await?,
                )
            } else {
                log::debug!("Delete vote {vote_id}");
                sqlx::query!(
                    "DELETE from votes \
                    WHERE vote_id = $1 AND \
                          post_id = $2 AND \
                          user_id = $3",
                    vote_id,
                    post_id,
                    user.user_id,
                )
                .execute(&db_pool)
                .await?;
                None
            }
        } else {
            log::debug!("Create vote for post {post_id}, comment {comment_id:?}, user {} with value {vote_value:?}", user.user_id);
            Some(
                sqlx::query_as!(
                    Vote,
                    "INSERT INTO votes (post_id, comment_id, user_id, value) VALUES ($1, $2, $3, $4) RETURNING *",
                    post_id,
                    comment_id,
                    user.user_id,
                    vote_value as i16,
                )
                    .fetch_one(&db_pool)
                    .await?)
        };

        update_content_score(
            vote_value,
            post_id,
            comment_id,
            previous_vote_info,
            &db_pool,
        )
        .await?;

        Ok(vote)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_get_vote_deltas() {
            let mut vote = VoteValue::Up;
            let mut previous_vote = VoteValue::None;
            let (mut score_delta, mut minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (1, 0));

            vote = VoteValue::None;
            previous_vote = VoteValue::Up;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (-1, 0));

            vote = VoteValue::Down;
            previous_vote = VoteValue::None;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (-1, 1));

            vote = VoteValue::None;
            previous_vote = VoteValue::Down;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (1, -1));

            vote = VoteValue::Up;
            previous_vote = VoteValue::Down;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (2, -1));

            vote = VoteValue::Down;
            previous_vote = VoteValue::Up;
            (score_delta, minus_delta) = get_vote_deltas(vote, previous_vote);
            assert_eq!((score_delta, minus_delta), (-2, 1));
        }
    }
}

#[server]
pub async fn vote_on_content(
    vote_value: VoteValue,
    post_id: i64,
    comment_id: Option<i64>,
    previous_vote_info: Option<VoteInfo>,
) -> Result<Option<Vote>, ServerFnError> {
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let vote = ssr::vote_on_content(
        vote_value,
        post_id,
        comment_id,
        previous_vote_info,
        &user,
        db_pool,
    )
    .await?;

    Ok(vote)
}

/// Component to display a post's score
#[component]
pub fn ScoreIndicator(score: i32) -> impl IntoView {
    view! {
        <div class="w-fit flex rounded-btn px-1 gap-1 items-center">
            <ScoreIcon/>
            <div class="min-w-6 text-sm">
                {score}
            </div>
        </div>
    }
}

/// Dynamic score indicator, that can be updated through the given signal
#[component]
pub fn DynScoreIndicator(score: RwSignal<i32>) -> impl IntoView {
    view! {
        <div class="flex rounded-btn gap-1 items-center">
            <ScoreIcon/>
            <div class="w-fit text-sm text-right">
                {move || score.get()}
            </div>
        </div>
    }
}

/// Component to display and modify a content's score
#[component]
pub fn VotePanel<'a>(
    post_id: i64,
    comment_id: Option<i64>,
    score: i32,
    vote: &'a Option<Vote>,
) -> impl IntoView {

    let (vote_id, vote_value, initial_score) = match vote {
        Some(vote) => (
            Some(vote.vote_id),
            Some(vote.value),
            score - (vote.value as i32),
        ),
        None => (None, None, score),
    };

    let score = create_rw_signal(score);
    let vote = create_rw_signal(vote_value.unwrap_or(VoteValue::None));

    let vote_action = create_server_action::<VoteOnContent>();

    let upvote_button_css = get_vote_button_css(vote, true);
    let downvote_button_css = get_vote_button_css(vote, false);

    view! {
        <div class="flex items-center gap-1">
            <LoginGuardButton
                login_button_class="p-1 rounded-full hover:bg-success"
                login_button_content=move || view! { <PlusIcon/> }
                let:_user
            >
                <button
                    class=upvote_button_css()
                    on:click=move |_| {
                        on_content_vote(
                            vote,
                            score,
                            post_id,
                            comment_id,
                            initial_score,
                            vote_id,
                            vote_value,
                            vote_action,
                            true
                        );
                    }
                >
                    <PlusIcon/>
                </button>
            </LoginGuardButton>
            <DynScoreIndicator score=score/>
            <LoginGuardButton
                login_button_class="p-1 rounded-full hover:bg-error"
                login_button_content=move || view! { <MinusIcon/> }
                let:_user
            >
                <button
                    class=downvote_button_css()
                    on:click=move |_| {
                        on_content_vote(
                            vote,
                            score,
                            post_id,
                            comment_id,
                            initial_score,
                            vote_id,
                            vote_value,
                            vote_action,
                            false
                        );
                    }
                >
                    <MinusIcon/>
                </button>
            </LoginGuardButton>
        </div>
    }
}

// Function to react to an post's upvote or downvote button being clicked.
pub fn on_content_vote(
    vote: RwSignal<VoteValue>,
    score: RwSignal<i32>,
    post_id: i64,
    comment_id: Option<i64>,
    initial_score: i32,
    current_vote_id: Option<i64>,
    current_vote_value: Option<VoteValue>,
    vote_action: Action<VoteOnContent, Result<Option<Vote>, ServerFnError>>,
    is_upvote: bool,
) {
    vote.update(|vote| update_vote_value(vote, is_upvote));

    log::trace!("Content vote value {:?}", vote.get_untracked());

    let previous_vote_info = if vote_action.version().get_untracked() == 0
        && current_vote_id.is_some()
        && current_vote_value.is_some()
    {
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
    score.set(initial_score + (vote.get_untracked() as i32));
}

// Function to react to an post's upvote or downvote button being clicked.
pub fn get_on_content_vote_closure(
    vote: RwSignal<VoteValue>,
    score: RwSignal<i32>,
    post_id: i64,
    comment_id: Option<i64>,
    initial_score: i32,
    current_vote_id: Option<i64>,
    current_vote_value: Option<VoteValue>,
    vote_action: Action<VoteOnContent, Result<Option<Vote>, ServerFnError>>,
    is_upvote: bool,
) -> impl Fn(ev::MouseEvent) {
    move |_| {
        vote.update(|vote| update_vote_value(vote, is_upvote));

        log::trace!("Content vote value {:?}", vote.get_untracked());

        let previous_vote_info = if vote_action.version().get_untracked() == 0
            && current_vote_id.is_some()
            && current_vote_value.is_some()
        {
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
        score.set(initial_score + (vote.get_untracked() as i32));
    }
}

// Function to obtain the css classes of a vote button
pub fn get_vote_button_css(vote: RwSignal<VoteValue>, is_upvote: bool) -> impl Fn() -> String {
    let (button_css, activated_value) = match is_upvote {
        true => ("bg-success", VoteValue::Up),
        false => ("bg-error", VoteValue::Down),
    };

    move || {
        if vote.get() == activated_value {
            format!("p-1 rounded-full {button_css}")
        } else {
            format!("p-1 rounded-full hover:{button_css}")
        }
    }
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
    use crate::ranking::{update_vote_value, VoteValue};

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
