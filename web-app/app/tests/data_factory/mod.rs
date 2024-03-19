use leptos::ServerFnError;
use sqlx::PgPool;
use app::comment::Comment;
use app::{forum, post};
use app::auth::User;
use app::forum::Forum;
use app::post::Post;

pub async fn create_forum_with_posts(
    forum_name: &str,
    num_posts: usize,
    score_vec: Option<Vec<i32>>,
    user: &User,
    db_pool: PgPool,
) -> Result<(Forum, Vec<Post>), ServerFnError> {
    let forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        user.user_id,
        db_pool.clone(),
    ).await?;

    let mut expected_post_vec = Vec::<Post>::with_capacity(num_posts);
    for i in 0..num_posts {
        let mut post = post::ssr::create_post(
            forum_name,
            i.to_string().as_str(),
            "body",
            false,
            None,
            user,
            db_pool.clone(),
        ).await?;

        if let Some(score_vec) = &score_vec {
            if i < score_vec.len() {
                post = set_post_score(post.post_id, score_vec[i], db_pool.clone()).await?;
            }
        }

        expected_post_vec.push(post);
    }

    Ok((forum, expected_post_vec))
}

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
