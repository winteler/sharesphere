use leptos::ServerFnError;
use sqlx::PgPool;

use app::{comment, forum, post, ranking};
use app::auth::User;
use app::comment::Comment;
use app::forum::Forum;
use app::post::Post;
use app::ranking::VoteValue;

pub async fn create_forum_with_post(
    forum_name: &str,
    user: &User,
    db_pool:& PgPool,
) -> (Forum, Post) {
    let forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &user,
        db_pool.clone(),
    ).await.expect("Should be able to create forum.");

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        None,
        false,
        None,
        &user,
        &db_pool,
    ).await.expect("Should be able to create post.");

    (forum, post)
}

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
        &user,
        db_pool.clone(),
    ).await?;

    let mut expected_post_vec = Vec::<Post>::with_capacity(num_posts);
    for i in 0..num_posts {
        let mut post = post::ssr::create_post(
            forum_name,
            i.to_string().as_str(),
            "body",
            None,
            false,
            None,
            user,
            &db_pool,
        ).await?;

        if let Some(score_vec) = &score_vec {
            if i < score_vec.len() {
                post = set_post_score(post.post_id, score_vec[i], &db_pool).await?;
            }
        }

        expected_post_vec.push(post);
    }

    Ok((forum, expected_post_vec))
}

pub async fn create_post_with_comments(
    forum_name: &str,
    post_title: &str,
    num_comments: usize,
    parent_index_vec: Vec<Option<i64>>,
    score_vec: Vec<i32>,
    vote_vec: Vec<Option<VoteValue>>,
    user: &User,
    db_pool: PgPool,
) -> Result<Post, ServerFnError> {
    let post = post::ssr::create_post(
        forum_name,
        post_title,
        "body",
        None,
        false,
        None,
        user,
        &db_pool,
    ).await?;

    let mut comment_id_vec = Vec::<i64>::new();

    for i in 0..num_comments {
        let parent_id = parent_index_vec.get(i).cloned().unwrap_or(None);

        let comment = comment::ssr::create_comment(
            post.post_id,
            parent_id,
            i.to_string().as_str(),
            None,
            user,
            db_pool.clone(),
        ).await?;

        comment_id_vec.push(comment.comment_id);


        if let Some(score) = score_vec.get(i) {
            set_comment_score(comment.comment_id, *score, db_pool.clone()).await?;
        }

        if let Some(Some(vote)) = vote_vec.get(i) {
            ranking::ssr::vote_on_content(
                *vote,
                post.post_id,
                Some(comment.comment_id),
                None,
                user,
                &db_pool,
            ).await?;
        }
    }

    Ok(post)
}

pub async fn set_post_score(
    post_id: i64,
    score: i32,
    db_pool: &PgPool,
) -> Result<Post, ServerFnError> {
    let post = sqlx::query_as!(
        Post,
        "UPDATE posts SET score = $1, scoring_timestamp = CURRENT_TIMESTAMP WHERE post_id = $2 RETURNING *",
        score,
        post_id,
    )
        .fetch_one(db_pool)
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
