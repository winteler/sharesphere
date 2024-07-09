use sqlx::PgPool;

use app::comment::Comment;
use app::errors::AppError;
use app::ranking::Vote;

pub async fn get_comment_by_id(
    comment_id: i64,
    db_pool: &PgPool,
) -> Result<Comment, AppError> {
    let comment = sqlx::query_as!(
            Comment,
            "SELECT *
            FROM comments
            WHERE comment_id = $1",
            comment_id
        )
        .fetch_one(db_pool)
        .await?;

    Ok(comment)
}

pub async fn get_user_comment_vote(
    comment: &Comment,
    user_id: i64,
    db_pool: &PgPool,
) -> Result<Vote, AppError> {
    let vote = sqlx::query_as!(
            Vote,
            "SELECT *
            FROM votes
            WHERE
                post_id = $1 AND
                comment_id = $2 AND
                user_id = $3",
            comment.post_id,
            comment.comment_id,
            user_id,
        )
        .fetch_one(db_pool)
        .await?;

    Ok(vote)
}