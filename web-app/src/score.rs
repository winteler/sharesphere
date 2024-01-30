use cfg_if::cfg_if;
use leptos::*;
use serde::{Deserialize, Serialize};

use crate::auth::{LoginGuardButton};
use crate::icons::{MinusIcon, PlusIcon, ScoreIcon};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::auth::{get_db_pool, get_user};
    }
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PostVote {
    pub id: i64,
    pub creator_id: i64,
    pub post_id: i64,
    pub value: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentVote {
    pub id: i64,
    pub creator_id: i64,
    pub comment_id: i64,
    pub value: i32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[server]
pub async fn vote_on_post(
    post_id: i64,
    vote: i16,
    previous_vote_id: Option<i64>,
    previous_vote: Option<i16>,
) -> Result<Option<PostVote>, ServerFnError> {

    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if post_id < 1 {
        return Err(ServerFnError::ServerError(String::from("Invalid post id.")));
    }

    if vote == previous_vote.unwrap_or_default() {
        return Err(ServerFnError::ServerError(String::from("Identical to previous vote.")));
    }

    // TODO: add unique index to prevent multiple votes by same user

    let post_vote = if previous_vote_id.is_some() {
        if vote != 0 {
            Some(sqlx::query_as!(
                PostVote,
                "UPDATE post_votes SET value = $1 WHERE id = $2 RETURNING *",
                vote,
                previous_vote_id.unwrap(),
            )
                .fetch_one(&db_pool)
                .await?)
        } else {
            sqlx::query!(
                "DELETE from post_votes WHERE id = $1",
                previous_vote_id.unwrap(),
            )
                .execute(&db_pool)
                .await?;
            None
        }
    } else {
        Some(sqlx::query_as!(
            PostVote,
            "INSERT INTO post_votes (creator_id, post_id, value) VALUES ($1, $2, $3) RETURNING *",
            user.id,
            post_id,
            vote,
        )
            .fetch_one(&db_pool)
            .await?)
    };

    let post_score_delta = vote - previous_vote.unwrap_or_default();

    sqlx::query!(
            "UPDATE posts set score = score + $1, score_minus = score_minus + $2, timestamp = CURRENT_TIMESTAMP where id = $3",
            i32::from(post_score_delta),
            i32::from(-post_score_delta.signum()),
            post_id,
        )
        .execute(&db_pool)
        .await?;

    Ok(post_vote)
}

/// Component to display a post's score
#[component]
pub fn ScoreIndicator(score: i32) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <ScoreIcon/>
            <div class="w-2 text-sm text-center">
                {score}
            </div>
        </div>
    }
}

/// Dynamic score indicator, that can be updated through the given signal
#[component]
pub fn DynScoreIndicator(score: RwSignal<i32>) -> impl IntoView {
    view! {
        <div class="flex rounded-btn px-1 gap-1 items-center">
            <ScoreIcon/>
            <div class="w-2 text-sm text-center">
                {move || score.get()}
            </div>
        </div>
    }
}

/// Component to display and modify post's score
#[component]
pub fn VotePanel(
    score: RwSignal<i32>,
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
            <DynScoreIndicator score=score/>
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