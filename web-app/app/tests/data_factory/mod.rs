use leptos::ServerFnError;
use sqlx::PgPool;
use app::comment::Comment;
use app::post::Post;

pub async fn set_post_score(
    post_id: i64,
    score: i32,
    db_pool: PgPool,
) -> Result<Post, ServerFnError> {
    let post = sqlx::query_as!(
        Post,
        "UPDATE posts SET score = $1 WHERE post_id = $2 RETURNING *",
        score,
        post_id,
    )
        .fetch_one(&db_pool)
        .await?;

    Ok(post)
}

pub async fn set_comment_score(
    comment_id: i64,
    score: i32,
    db_pool: PgPool,
) -> Result<Comment, ServerFnError> {
    let comment = sqlx::query_as!(
        Comment,
        "UPDATE comments SET score = $1 WHERE comment_id = $2 RETURNING *",
        score,
        comment_id,
    )
        .fetch_one(&db_pool)
        .await?;

    Ok(comment)
}
