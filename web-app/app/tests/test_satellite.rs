use app::errors::AppError;
use app::satellite::ssr::{get_satellite_sphere, get_satellite_vec_by_sphere_name};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

#[tokio::test]
async fn test_get_satellite_vec_by_sphere_name() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (sphere, expected_satellite_vec) = create_sphere_with_satellites(
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

    let (expected_sphere, expected_satellite_vec) = create_sphere_with_satellites(
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