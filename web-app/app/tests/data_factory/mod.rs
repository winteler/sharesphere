use leptos::ServerFnError;
use sqlx::PgPool;

pub async fn set_post_score(
    post_id: i64,
    score: i32,
    db_pool: PgPool,
) -> Result<(), ServerFnError> {
    sqlx::query!(
        "UPDATE posts set score = $1 where post_id = $2",
        score,
        post_id,
    )
    .execute(&db_pool)
    .await?;

    Ok(())
}

pub async fn set_comment_score(
    comment_id: i64,
    score: i32,
    db_pool: PgPool,
) -> Result<(), ServerFnError> {
    sqlx::query!(
        "UPDATE comments set score = $1 where comment_id = $2",
        score,
        comment_id,
    )
    .execute(&db_pool)
    .await?;

    Ok(())
}
