use leptos::*;
use serde::{Deserialize, Serialize};

use crate::icons::{ScoreIcon};

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PostVote {
    pub vote_id: i64,
    pub creator_id: i64,
    pub post_id: i64,
    pub value: i16,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentVote {
    pub vote_id: i64,
    pub creator_id: i64,
    pub comment_id: i64,
    pub post_id: i64,
    pub value: i16,
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
        return Err(ServerFnError::new("Invalid post id."));
    }

    if vote == previous_vote.unwrap_or_default() {
        return Err(ServerFnError::new("Identical to previous vote."));
    }

    // TODO: add unique index to prevent multiple votes by same user

    let post_vote = if previous_vote_id.is_some() {
        if vote != 0 {
            Some(sqlx::query_as!(
                PostVote,
                "UPDATE post_votes SET value = $1 WHERE vote_id = $2 RETURNING *",
                vote,
                previous_vote_id.unwrap(),
            )
                .fetch_one(&db_pool)
                .await?)
        } else {
            sqlx::query!(
                "DELETE from post_votes WHERE vote_id = $1",
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
            user.user_id,
            post_id,
            vote,
        )
            .fetch_one(&db_pool)
            .await?)
    };

    let post_score_delta = vote - previous_vote.unwrap_or_default();

    sqlx::query!(
            "UPDATE posts set score = score + $1, score_minus = score_minus + $2, scoring_timestamp = CURRENT_TIMESTAMP where post_id = $3",
            i32::from(post_score_delta),
            i32::from(-post_score_delta.signum()),
            post_id,
        )
        .execute(&db_pool)
        .await?;

    Ok(post_vote)
}

#[server]
pub async fn vote_on_comment(
    comment_id: i64,
    post_id: i64,
    vote: i16,
    previous_vote_id: Option<i64>,
    previous_vote: Option<i16>,
) -> Result<Option<CommentVote>, ServerFnError> {

    let user = get_user().await?;
    let db_pool = get_db_pool()?;

    if comment_id < 1 {
        return Err(ServerFnError::new("Invalid post id."));
    }

    if vote == previous_vote.unwrap_or_default() {
        return Err(ServerFnError::new("Identical to previous vote."));
    }

    // TODO: add unique index to prevent multiple votes by same user

    let comment_vote = if previous_vote_id.is_some() {
        if vote != 0 {
            log::info!("Update vote");
            Some(sqlx::query_as!(
                CommentVote,
                "UPDATE comment_votes SET value = $1 WHERE vote_id = $2 RETURNING *",
                vote,
                previous_vote_id.unwrap(),
            )
                .fetch_one(&db_pool)
                .await?)
        } else {
            log::info!("Delete vote");
            sqlx::query!(
                "DELETE from comment_votes WHERE vote_id = $1",
                previous_vote_id.unwrap(),
            )
                .execute(&db_pool)
                .await?;
            None
        }
    } else {
        log::info!("Create vote");
        Some(sqlx::query_as!(
            CommentVote,
            "INSERT INTO comment_votes (creator_id, comment_id, post_id, value) VALUES ($1, $2, $3, $4) RETURNING *",
            user.user_id,
            comment_id,
            post_id,
            vote,
        )
            .fetch_one(&db_pool)
            .await?)
    };

    let comment_score_delta = vote - previous_vote.unwrap_or_default();

    sqlx::query!(
            "UPDATE comments set score = score + $1, score_minus = score_minus + $2, timestamp = CURRENT_TIMESTAMP where comment_id = $3",
            i32::from(comment_score_delta),
            i32::from(-comment_score_delta.signum()),
            comment_id,
        )
        .execute(&db_pool)
        .await?;

    Ok(comment_vote)
}

pub fn get_vote_button_css(
    vote: RwSignal<i16>,
    is_upvote: bool,
) -> impl Fn() -> String {
    let (button_css, activated_value) = match is_upvote {
        true => ("success", 1),
        false => ("error", -1),
    };

    move || {
        if vote() == activated_value {
            format!("btn btn-circle btn-sm btn-{button_css}")
        } else {
            format!("btn btn-circle btn-sm btn-ghost hover:btn-{button_css}")
        }
    }
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
