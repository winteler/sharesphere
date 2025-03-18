use sharesphere_auth::user::User;
use sharesphere_utils::errors::AppError;

use sharesphere_core::satellite::ssr::{get_active_satellite_vec_by_sphere_name, get_satellite_by_id, get_satellite_vec_by_sphere_name};
use sharesphere_core::sphere::ssr::create_sphere;
use sharesphere_core::satellite::ssr::{create_satellite, disable_satellite, get_satellite_sphere, update_satellite};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

#[tokio::test]
async fn test_get_satellite_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, satellite_vec) = create_sphere_with_satellite_vec(
        "1",
        2,
        &mut user,
        &db_pool,
    ).await.expect("Error creating sphere and satellites");

    let expected_satellite_1 = satellite_vec.first().expect("Should have satellite 1");
    let expected_satellite_2 = satellite_vec.get(1).expect("Should have satellite 2");

    let (_, expected_satellite_3) = create_sphere_with_satellite(
        "2",
        "3",
        true,
        true,
        &mut user,
        &db_pool,
    ).await.expect("Error creating sphere and satellite 3");
    
    let satellite_1 = get_satellite_by_id(expected_satellite_1.satellite_id, &db_pool).await.expect("Error getting satellite 1");
    let satellite_2 = get_satellite_by_id(expected_satellite_2.satellite_id, &db_pool).await.expect("Error getting satellite 2");
    let satellite_3 = get_satellite_by_id(expected_satellite_3.satellite_id, &db_pool).await.expect("Error getting satellite 3");

    assert_eq!(satellite_1, *expected_satellite_1);
    assert_eq!(satellite_2, *expected_satellite_2);
    assert_eq!(satellite_3, expected_satellite_3);

    Ok(())
}

#[tokio::test]
async fn test_get_active_satellite_vec_by_sphere_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (sphere, mut expected_satellite_vec) = create_sphere_with_satellite_vec(
        "sphere",
        5,
        &mut user,
        &db_pool,
    ).await.expect("Error creating sphere and satellites");
    
    let satellite = expected_satellite_vec.pop().expect("Should pop satellite");
    
    disable_satellite(satellite.satellite_id, &user, &db_pool).await.expect("Should disable satellite");

    let satellite_vec = get_active_satellite_vec_by_sphere_name(
        &sphere.sphere_name,
        &db_pool
    ).await.expect("Satellite vec should be loaded");

    assert_eq!(satellite_vec, expected_satellite_vec);

    Ok(())
}

#[tokio::test]
async fn test_get_satellite_vec_by_sphere_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (sphere, expected_satellite_vec) = create_sphere_with_satellite_vec(
        "sphere",
        5,
        &mut user,
        &db_pool,
    ).await.expect("Error creating sphere and satellites");
    
    let satellite_vec = get_satellite_vec_by_sphere_name(
        &sphere.sphere_name, 
        &db_pool
    ).await.expect("Satellite vec should be loaded");
    
    assert_eq!(satellite_vec, expected_satellite_vec);
    
    Ok(())
}

#[tokio::test]
async fn test_get_satellite_sphere() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (expected_sphere, expected_satellite_vec) = create_sphere_with_satellite_vec(
        "sphere",
        1,
        &mut user,
        &db_pool,
    ).await.expect("Error creating sphere and satellites");
    
    let sphere = get_satellite_sphere(
        expected_satellite_vec.first().expect("Satellite should exist").satellite_id,
        &db_pool,
    ).await.expect("Error getting satellite");
    
    assert_eq!(sphere, expected_sphere);
    
    Ok(())
}

#[tokio::test]
async fn test_create_satellite() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let sphere = create_sphere(
        "a",
        "a",
        false,
        &mut user,
        &db_pool,
    ).await.expect("Sphere should be created");

    let nsfw_sphere = create_sphere(
        "b",
        "b",
        true,
        &mut user,
        &db_pool,
    ).await.expect("Nsfw sphere should be created");


    assert_eq!(
        create_satellite("1", &sphere.sphere_name, "1", None, false, false, &user, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );

    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user");

    let satellite_1 = create_satellite(
        &sphere.sphere_name,
        "1",
        "1",
        Some("1"),
        false,
        true,
        &user,
        &db_pool
    ).await.expect("Satellite 1 should be created");

    assert_eq!(satellite_1.satellite_name, "1");
    assert_eq!(satellite_1.sphere_name, sphere.sphere_name);
    assert_eq!(satellite_1.body, "1");
    assert_eq!(satellite_1.markdown_body.as_deref(), Some("1"));
    assert_eq!(satellite_1.is_nsfw, false);
    assert_eq!(satellite_1.is_spoiler, true);
    assert_eq!(satellite_1.disable_timestamp, None);

    let satellite_2 = create_satellite(
        &sphere.sphere_name,
        "2",
        "2",
        None,
        true,
        false,
        &user,
        &db_pool
    ).await.expect("Satellite 2 should be created");

    assert_eq!(satellite_2.satellite_name, "2");
    assert_eq!(satellite_2.sphere_name, sphere.sphere_name);
    assert_eq!(satellite_2.body, "2");
    assert_eq!(satellite_2.markdown_body, None);
    assert_eq!(satellite_2.is_nsfw, true);
    assert_eq!(satellite_2.is_spoiler, false);
    assert_eq!(satellite_2.disable_timestamp, None);

    let nsfw_satellite = create_satellite(
        &nsfw_sphere.sphere_name,
        "3",
        "3",
        None,
        false,
        false,
        &user,
        &db_pool
    ).await.expect("Nsfw satellite should be created");

    assert_eq!(nsfw_satellite.satellite_name, "3");
    assert_eq!(nsfw_satellite.sphere_name, nsfw_sphere.sphere_name);
    assert_eq!(nsfw_satellite.body, "3");
    assert_eq!(nsfw_satellite.markdown_body, None);
    assert_eq!(nsfw_satellite.is_nsfw, true);
    assert_eq!(nsfw_satellite.is_spoiler, false);
    assert_eq!(nsfw_satellite.disable_timestamp, None);

    Ok(())
}

#[tokio::test]
async fn test_update_satellite() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;
    let base_user = create_user("a", &db_pool).await;

    let (sphere_1, satellite_1) = create_sphere_with_satellite(
        "1",
        "1",
        true,
        false,
        &mut user,
        &db_pool,
    ).await.expect("Sphere with satellite should be created");

    let nsfw_sphere = create_sphere(
        "2",
        "2",
        true,
        &mut user,
        &db_pool,
    ).await.expect("Nsfw sphere should be created");

    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user");

    let nsfw_satellite = create_satellite(
        &nsfw_sphere.sphere_name,
        "2",
        "2",
        Some("2"),
        true,
        true,
        &user,
        &db_pool
    ).await.expect("Nsfw satellite should be created");

    assert_eq!(
        update_satellite(satellite_1.satellite_id, "a", "error", None, false, true, &base_user, &db_pool).await,
        Err(AppError::InsufficientPrivileges),
    );

    let updated_satellite_1 = update_satellite(
        satellite_1.satellite_id,
        "a",
        "a",
        Some("a"),
        false,
        true,
        &user,
        &db_pool
    ).await.expect("Satellite 1 should be updated");

    assert_eq!(updated_satellite_1.satellite_id, satellite_1.satellite_id);
    assert_eq!(updated_satellite_1.satellite_name, "a");
    assert_eq!(updated_satellite_1.sphere_name, sphere_1.sphere_name);
    assert_eq!(updated_satellite_1.body, "a");
    assert_eq!(updated_satellite_1.markdown_body.as_deref(), Some("a"));
    assert_eq!(updated_satellite_1.is_nsfw, false);
    assert_eq!(updated_satellite_1.is_spoiler, true);
    assert_eq!(updated_satellite_1.disable_timestamp, None);

    let updated_nsfw_satellite = update_satellite(
        nsfw_satellite.satellite_id,
        "b",
        "b",
        None,
        false,
        false,
        &user,
        &db_pool
    ).await.expect("Nsfw satellite should be updated");

    assert_eq!(updated_nsfw_satellite.satellite_id, nsfw_satellite.satellite_id);
    assert_eq!(updated_nsfw_satellite.satellite_name, "b");
    assert_eq!(updated_nsfw_satellite.sphere_name, nsfw_sphere.sphere_name);
    assert_eq!(updated_nsfw_satellite.body, "b");
    assert_eq!(updated_nsfw_satellite.markdown_body, None);
    assert_eq!(updated_nsfw_satellite.is_nsfw, true);
    assert_eq!(updated_nsfw_satellite.is_spoiler, false);
    assert_eq!(updated_nsfw_satellite.disable_timestamp, None);

    Ok(())
}

#[tokio::test]
async fn test_disable_satellite() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (sphere, satellite) = create_sphere_with_satellite(
        "1",
        "1",
        false,
        false,
        &mut user,
        &db_pool,
    ).await.expect("Sphere with satellite should be created");

    let deleted_satellite = disable_satellite(satellite.satellite_id, &user, &db_pool).await.expect("Satellite should be deleted");

    assert_eq!(deleted_satellite.satellite_name, "1");
    assert_eq!(deleted_satellite.sphere_name, sphere.sphere_name);
    assert_eq!(deleted_satellite.body, "test");
    assert_eq!(deleted_satellite.is_nsfw, false);
    assert_eq!(deleted_satellite.is_spoiler, false);
    assert!(deleted_satellite.disable_timestamp.is_some_and(|delete_timestamp| delete_timestamp > deleted_satellite.timestamp));

    let satellite_vec = get_satellite_vec_by_sphere_name(&sphere.sphere_name, &db_pool).await.expect("Should get sphere satellite vec");

    assert_eq!(satellite_vec.first(), Some(&deleted_satellite));

    Ok(())
}