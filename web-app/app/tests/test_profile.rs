use app::embed::Link;
use app::errors::AppError;
use app::post::{PostWithSphereInfo};
use app::post::ssr::create_post;
use app::profile::ssr::{get_user_post_vec};
use app::ranking::SortType;
use app::satellite::ssr::create_satellite;

use crate::common::{create_user, get_db_pool};
use crate::data_factory::create_sphere_with_posts;
use crate::utils::{sort_post_vec, test_post_vec, POST_SORT_TYPE_ARRAY};

mod common;
mod data_factory;
mod utils;

#[tokio::test]
async fn test_get_user_post_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user_1 = create_user("1", &db_pool).await;
    let mut user_2 = create_user("2", &db_pool).await;

    let sphere1_name = "1";
    let sphere2_name = "2";
    let num_post = 10usize;

    let (_, _, mut user_1_expected_post_vec) = create_sphere_with_posts(
        sphere1_name,
        None,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        (0..num_post).map(|i| (i % 2) == 0).collect(),
        &mut user_1,
        &db_pool,
    ).await.expect("Should create sphere 1 with posts.");
    
    let satellite = create_satellite(sphere1_name, "1", "test", None, false, false, &user_1, &db_pool).await.expect("Should create satellite 1");
    let satellite_post = create_post(
        sphere1_name,
        Some(satellite.satellite_id),
        "satellite",
        "satellite",
        None,
        Link::default(),
        false,
        false,
        false,
        None,
        &user_1,
        &db_pool,
    ).await.expect("Should create satellite post");
    user_1_expected_post_vec.push(PostWithSphereInfo::from_post(satellite_post, None, None));

    let (_, _, mut user_2_expected_post_vec) = create_sphere_with_posts(
        sphere2_name,
        None,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        (0..num_post).map(|i| (i % 2) == 0).collect(),
        &mut user_2,
        &db_pool,
    ).await.expect("Should create sphere 2 with posts.");

    let sphere_2_post = create_post(
        sphere2_name,
        None,
        "sphere_2",
        "sphere_2",
        None,
        Link::default(),
        false,
        false,
        false,
        None,
        &user_1,
        &db_pool,
    ).await.expect("Should create satellite post");
    user_1_expected_post_vec.push(PostWithSphereInfo::from_post(sphere_2_post, None, None));
    
    for sort_type in POST_SORT_TYPE_ARRAY {
        let user_1_post_vec = get_user_post_vec(
            &user_1.username,
            SortType::Post(sort_type),
            (num_post + 2) as i64,
            0,
            &db_pool,
        ).await?;
        sort_post_vec(&mut user_1_expected_post_vec, sort_type);
        test_post_vec(&user_1_post_vec, &user_1_expected_post_vec, sort_type);

        let user_2_post_vec = get_user_post_vec(
            &user_2.username,
            SortType::Post(sort_type),
            num_post as i64,
            0,
            &db_pool,
        ).await?;
        sort_post_vec(&mut user_2_expected_post_vec, sort_type);
        test_post_vec(&user_2_post_vec, &user_2_expected_post_vec, sort_type);
    }
    
    Ok(())
}