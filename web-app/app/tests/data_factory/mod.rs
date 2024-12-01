#![allow(dead_code)]

use sqlx::PgPool;

use app::colors::Color;
use app::comment::ssr::create_comment;
use app::comment::Comment;
use app::errors::AppError;
use app::forum::Forum;
use app::forum_category::ForumCategory;
use app::forum_management::ssr::set_forum_icon_url;
use app::post::{Post, PostWithForumInfo};
use app::ranking::VoteValue;
use app::user::User;
use app::{comment, forum, forum_category, post, ranking};

pub async fn create_forum_with_post(
    forum_name: &str,
    user: &User,
    db_pool: &PgPool,
) -> (Forum, Post) {
    let forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &user,
        db_pool,
    ).await.expect("Should be able to create forum.");

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        None,
        false,
        false,
        false,
        None,
        &user,
        db_pool,
    ).await.expect("Should be able to create post.");

    (forum, post)
}

pub async fn create_forum_with_post_and_comment(
    forum_name: &str,
    user: &User,
    db_pool: &PgPool,
) -> (Forum, Post, Comment) {
    let (forum, post) = create_forum_with_post(forum_name, user, db_pool).await;

    let comment = create_comment(post.post_id, None, "comment", None, false, user, db_pool).await.expect("Comment should be created.");

    (forum, post, comment)
}

pub async fn create_forum_with_posts(
    forum_name: &str,
    forum_icon_url: Option<&str>,
    num_posts: usize,
    score_vec: Option<Vec<i32>>,
    category_vec: Vec<bool>,
    user: &User,
    db_pool: &PgPool,
) -> Result<(Forum, ForumCategory, Vec<PostWithForumInfo>), AppError> {
    let mut forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        user,
        db_pool,
    ).await?;
    
    let user = User::get(user.user_id, db_pool).await.expect("Should reload user.");

    set_forum_icon_url(forum_name, forum_icon_url, &user, &db_pool).await.expect("Should set icon url.");
    forum.icon_url = forum_icon_url.map(|x| x.to_string());
    
    let forum_category = forum_category::ssr::set_forum_category(
        forum_name,
        "create_posts",
        Color::Blue,
        "test",
        true,
        &user,
        db_pool,
    ).await.expect("Forum category should be created.");

    let mut expected_post_vec = Vec::<PostWithForumInfo>::with_capacity(num_posts);
    for i in 0..num_posts {
        let category_id = match category_vec.get(i) {
            Some(has_category) if *has_category => Some(forum_category.category_id),
            _ => None,
        };
        let mut post = post::ssr::create_post(
            forum_name,
            i.to_string().as_str(),
            "body",
            None,
            false,
            false,
            false,
            category_id,
            &user,
            db_pool,
        ).await?;

        if let Some(score_vec) = &score_vec {
            if i < score_vec.len() {
                post = set_post_score(post.post_id, score_vec[i], db_pool).await?;
            }
        }

        let forum_category_header = category_id.map(|_| forum_category.clone().into());
        expected_post_vec.push(PostWithForumInfo::from_post(post, forum_category_header, forum.icon_url.clone()));
    }

    Ok((forum, forum_category, expected_post_vec))
}

pub async fn create_post_with_comments(
    forum_name: &str,
    post_title: &str,
    num_comments: usize,
    parent_index_vec: Vec<Option<i64>>,
    score_vec: Vec<i32>,
    vote_vec: Vec<Option<VoteValue>>,
    user: &User,
    db_pool: &PgPool,
) -> Result<Post, AppError> {
    let post = post::ssr::create_post(
        forum_name,
        post_title,
        "body",
        None,
        false,
        false,
        false,
        None,
        user,
        db_pool,
    ).await?;

    let mut comment_id_vec = Vec::<i64>::new();

    for i in 0..num_comments {
        let parent_id = parent_index_vec.get(i).cloned().unwrap_or(None);

        let comment = comment::ssr::create_comment(
            post.post_id,
            parent_id,
            i.to_string().as_str(),
            None,
            false,
            user,
            db_pool,
        ).await?;

        comment_id_vec.push(comment.comment_id);


        if let Some(score) = score_vec.get(i) {
            set_comment_score(comment.comment_id, *score, db_pool).await?;
        }

        if let Some(Some(vote)) = vote_vec.get(i) {
            ranking::ssr::vote_on_content(
                *vote,
                post.post_id,
                Some(comment.comment_id),
                None,
                user,
                db_pool,
            ).await?;
        }
    }

    Ok(post)
}

pub async fn set_post_score(
    post_id: i64,
    score: i32,
    db_pool: &PgPool,
) -> Result<Post, AppError> {
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
    db_pool: &PgPool,
) -> Result<Comment, AppError> {
    let comment = sqlx::query_as!(
        Comment,
        "UPDATE comments SET score = $1 WHERE comment_id = $2 RETURNING *",
        score,
        comment_id,
    )
        .fetch_one(db_pool)
        .await?;

    Ok(comment)
}
