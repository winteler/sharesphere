#![allow(dead_code)]

use sqlx::PgPool;

use app::colors::Color;
use app::comment::ssr::create_comment;
use app::comment::Comment;
use app::errors::AppError;
use app::post::{LinkType, Post, PostWithSphereInfo};
use app::ranking::VoteValue;
use app::satellite::ssr::create_satellite;
use app::satellite::Satellite;
use app::sphere::Sphere;
use app::sphere_category::SphereCategory;
use app::sphere_management::ssr::set_sphere_icon_url;
use app::user::User;
use app::{comment, post, ranking, sphere, sphere_category};

pub async fn create_sphere_with_post(
    sphere_name: &str,
    user: &mut User,
    db_pool: &PgPool,
) -> (Sphere, Post) {
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        user,
        db_pool,
    ).await.expect("Should be able to create sphere.");

    *user = User::get(user.user_id, db_pool).await.expect("Should reload user.");

    let post = post::ssr::create_post(
        sphere_name,
        None,
        "post",
        "body",
        None,
        None,
        LinkType::None,
        None,
        false,
        false,
        false,
        None, 
        user,
        db_pool,
    ).await.expect("Should be able to create post.");

    (sphere, post)
}

pub async fn create_sphere_with_post_and_comment(
    sphere_name: &str,
    user: &mut User,
    db_pool: &PgPool,
) -> (Sphere, Post, Comment) {
    let (sphere, post) = create_sphere_with_post(sphere_name, user, db_pool).await;

    let comment = create_comment(post.post_id, None, "comment", None, false, user, db_pool).await.expect("Comment should be created.");

    (sphere, post, comment)
}

pub async fn create_sphere_with_posts(
    sphere_name: &str,
    sphere_icon_url: Option<&str>,
    num_posts: usize,
    score_vec: Option<Vec<i32>>,
    category_vec: Vec<bool>,
    user: &mut User,
    db_pool: &PgPool,
) -> Result<(Sphere, SphereCategory, Vec<PostWithSphereInfo>), AppError> {
    let mut sphere = sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        user,
        db_pool,
    ).await?;

    *user = User::get(user.user_id, db_pool).await.expect("Should reload user.");

    set_sphere_icon_url(sphere_name, sphere_icon_url, user, db_pool).await.expect("Should set icon url.");
    sphere.icon_url = sphere_icon_url.map(|x| x.to_string());

    let sphere_category = sphere_category::ssr::set_sphere_category(
        sphere_name,
        "create_posts",
        Color::Blue,
        "test",
        true,
        user,
        db_pool,
    ).await.expect("Sphere category should be created.");

    let expected_post_vec = create_posts(
        &sphere,
        None,
        num_posts,
        score_vec,
        Some(&sphere_category),
        category_vec,
        user,
        db_pool,
    ).await?;

    Ok((sphere, sphere_category, expected_post_vec))
}

pub async fn create_posts(
    sphere: &Sphere,
    satellite_id: Option<i64>,
    num_posts: usize,
    score_vec: Option<Vec<i32>>,
    sphere_category: Option<&SphereCategory>,
    category_vec: Vec<bool>,
    user: &User,
    db_pool: &PgPool,
) -> Result<Vec<PostWithSphereInfo>, AppError> {

    let mut expected_post_vec = Vec::<PostWithSphereInfo>::with_capacity(num_posts);
    for i in 0..num_posts {
        let category_id = match (category_vec.get(i), sphere_category) {
            (Some(has_category), Some(sphere_category)) if *has_category => Some(sphere_category.category_id),
            _ => None,
        };
        let mut post = post::ssr::create_post(
            &sphere.sphere_name,
            satellite_id,
            i.to_string().as_str(),
            "body",
            None,
            None,
            LinkType::None,
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

        let sphere_category_header = match (category_id, sphere_category) {
            (Some(_), Some(sphere_category)) => Some(sphere_category.clone().into()),
            _ => None,
        };
        expected_post_vec.push(PostWithSphereInfo::from_post(post, sphere_category_header, sphere.icon_url.clone()));
    }

    Ok(expected_post_vec)
}

pub async fn create_sphere_with_satellite(
    sphere_name: &str,
    satellite_name: &str,
    is_nsfw_satellite: bool,
    is_spoiler_satellite: bool,
    user: &mut User,
    db_pool: &PgPool,
) -> Result<(Sphere, Satellite), AppError> {
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        user,
        db_pool,
    ).await?;

    *user = User::get(user.user_id, db_pool).await.expect("Should reload user.");

    let satellite = create_satellite(
        &sphere.sphere_name,
        satellite_name,
        "test",
        None,
        is_nsfw_satellite,
        is_spoiler_satellite,
        user,
        db_pool,
    ).await.expect("Satellite should be inserted");

    Ok((sphere, satellite))
}

pub async fn create_sphere_with_satellite_vec(
    sphere_name: &str,
    num_satellites: usize,
    user: &mut User,
    db_pool: &PgPool,
) -> Result<(Sphere, Vec<Satellite>), AppError> {
    let sphere = sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        user,
        db_pool,
    ).await?;
    
    *user = User::get(user.user_id, db_pool).await.expect("Should reload user.");
    
    let mut satellite_vec = Vec::new();
    for i in 0..num_satellites {
        let satellite = create_satellite(
            &sphere.sphere_name,
            i.to_string().as_str(),
            "test",
            None,
            false,
            false,
            user,
            db_pool,
        ).await.expect("Satellite 1 should be inserted");
        
        satellite_vec.push(satellite);
    }
    
    Ok((sphere, satellite_vec))
}

pub async fn create_post_with_comments(
    sphere_name: &str,
    post_title: &str,
    num_comments: usize,
    parent_index_vec: Vec<Option<i64>>,
    score_vec: Vec<i32>,
    vote_vec: Vec<Option<VoteValue>>,
    user: &User,
    db_pool: &PgPool,
) -> Result<Post, AppError> {
    let post = post::ssr::create_post(
        sphere_name,
        None,
        post_title,
        "body",
        None,
        None,
        LinkType::None,
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
