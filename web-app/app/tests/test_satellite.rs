use app::errors::AppError;
use app::satellite::ssr::{create_satellite, get_satellite_sphere, get_satellite_vec_by_sphere_name, update_satellite};
use app::sphere::ssr::create_sphere;
use app::user::User;
pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

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
        create_satellite("1", &sphere.sphere_name, "1", false, false, &user, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );

    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user");

    let satellite_1 = create_satellite(
        "1",
        &sphere.sphere_name,
        "1",
        false,
        true,
        &user,
        &db_pool
    ).await.expect("Satellite 1 should be created");

    assert_eq!(satellite_1.satellite_name, "1");
    assert_eq!(satellite_1.sphere_name, sphere.sphere_name);
    assert_eq!(satellite_1.description, "1");
    assert_eq!(satellite_1.is_nsfw, false);
    assert_eq!(satellite_1.is_spoiler, true);

    let satellite_2 = create_satellite(
        "2",
        &sphere.sphere_name,
        "2",
        true,
        false,
        &user,
        &db_pool
    ).await.expect("Satellite 2 should be created");

    assert_eq!(satellite_2.satellite_name, "2");
    assert_eq!(satellite_2.sphere_name, sphere.sphere_name);
    assert_eq!(satellite_2.description, "2");
    assert_eq!(satellite_2.is_nsfw, true);
    assert_eq!(satellite_2.is_spoiler, false);

    let nsfw_satellite = create_satellite(
        "3",
        &nsfw_sphere.sphere_name,
        "3",
        false,
        false,
        &user,
        &db_pool
    ).await.expect("Nsfw satellite should be created");

    assert_eq!(nsfw_satellite.satellite_name, "3");
    assert_eq!(nsfw_satellite.sphere_name, nsfw_sphere.sphere_name);
    assert_eq!(nsfw_satellite.description, "3");
    assert_eq!(nsfw_satellite.is_nsfw, true);
    assert_eq!(nsfw_satellite.is_spoiler, false);

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
        "2",
        &nsfw_sphere.sphere_name,
        "2",
        true,
        true,
        &user,
        &db_pool
    ).await.expect("Nsfw satellite should be created");

    assert_eq!(
        update_satellite(satellite_1.satellite_id, "a", "error", false, true, &base_user, &db_pool).await,
        Err(AppError::InsufficientPrivileges),
    );

    let updated_satellite_1 = update_satellite(
        satellite_1.satellite_id,
        "a",
        "a",
        false,
        true,
        &user,
        &db_pool
    ).await.expect("Satellite 1 should be updated");

    assert_eq!(updated_satellite_1.satellite_id, satellite_1.satellite_id);
    assert_eq!(updated_satellite_1.satellite_name, "a");
    assert_eq!(updated_satellite_1.sphere_name, sphere_1.sphere_name);
    assert_eq!(updated_satellite_1.description, "a");
    assert_eq!(updated_satellite_1.is_nsfw, false);
    assert_eq!(updated_satellite_1.is_spoiler, true);

    let updated_nsfw_satellite = update_satellite(
        nsfw_satellite.satellite_id,
        "b",
        "b",
        false,
        false,
        &user,
        &db_pool
    ).await.expect("Nsfw satellite should be updated");

    assert_eq!(updated_nsfw_satellite.satellite_id, nsfw_satellite.satellite_id);
    assert_eq!(updated_nsfw_satellite.satellite_name, "b");
    assert_eq!(updated_nsfw_satellite.sphere_name, nsfw_sphere.sphere_name);
    assert_eq!(updated_nsfw_satellite.description, "b");
    assert_eq!(updated_nsfw_satellite.is_nsfw, true);
    assert_eq!(updated_nsfw_satellite.is_spoiler, false);

    Ok(())
}