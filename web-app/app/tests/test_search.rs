use std::collections::BTreeSet;
use app::errors::AppError;
use app::search::ssr::{get_matching_sphere_header_vec, get_matching_username_set, search_post};
use app::{sphere};
use app::sphere::ssr::create_sphere;
use app::sphere_management::ssr::set_sphere_icon_url;
use app::user::User;
use crate::common::{create_test_user, create_user, get_db_pool};
use crate::data_factory::create_simple_post;

mod common;
mod data_factory;


#[tokio::test]
async fn test_get_matching_username_set() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;

    let num_users = 20usize;
    let mut expected_username_set = BTreeSet::<String>::new();
    for i in 0..num_users {
        expected_username_set.insert(
            create_user(
                i.to_string().as_str(),
                &db_pool,
            ).await.username
        );
    }

    let username_set = get_matching_username_set("1", num_users as i64, &db_pool).await?;

    let mut previous_username = None;
    for username in username_set {
        assert_eq!(username.chars().next().unwrap(), '1');
        if let Some(previous_username) = previous_username {
            assert!(previous_username < username)
        }
        previous_username = Some(username);
    }

    for i in num_users..2 * num_users {
        expected_username_set.insert(
            create_user(
                i.to_string().as_str(),
                &db_pool,
            ).await.username
        );
    }

    let username_set = get_matching_username_set("", num_users as i64, &db_pool).await?;

    assert_eq!(username_set.len(), num_users);

    Ok(())
}

#[tokio::test]
async fn test_get_matching_sphere_header_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let num_spheres = 20usize;
    let mut expected_sphere_name_vec = Vec::new();
    for i in 0..num_spheres {
        expected_sphere_name_vec.push(
            sphere::ssr::create_sphere(
                i.to_string().as_str(),
                "sphere",
                i % 2 == 0,
                &user,
                &db_pool,
            ).await?.sphere_name,
        );
    }

    let user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded.");

    let first_sphere_icon_url = Some("a");
    set_sphere_icon_url(expected_sphere_name_vec.first().unwrap(), first_sphere_icon_url, &user, &db_pool).await.expect("Sphere icon should be set.");

    let sphere_header_vec = get_matching_sphere_header_vec("1", num_spheres as i64, &db_pool).await?;

    let mut previous_sphere_name = None;
    for sphere_header in sphere_header_vec {
        assert_eq!(sphere_header.icon_url, None);
        assert_eq!(sphere_header.sphere_name.chars().next().unwrap(), '1');
        if let Some(previous_sphere_name) = previous_sphere_name {
            assert!(previous_sphere_name < sphere_header.sphere_name)
        }
        previous_sphere_name = Some(sphere_header.sphere_name.clone());
    }

    for i in num_spheres..2 * num_spheres {
        expected_sphere_name_vec.push(
            sphere::ssr::create_sphere(
                i.to_string().as_str(),
                "sphere",
                i % 2 == 0,
                &user,
                &db_pool,
            )
                .await?
                .sphere_name,
        );
    }

    let sphere_header_vec = get_matching_sphere_header_vec("", num_spheres as i64, &db_pool).await?;

    assert_eq!(sphere_header_vec.len(), num_spheres);
    assert_eq!(sphere_header_vec.first().unwrap().icon_url.as_deref(), first_sphere_icon_url);

    Ok(())
}

#[tokio::test]
async fn test_search_post() {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let sphere_1 = create_sphere("1", "1", false, &user, &db_pool).await.expect("Sphere 1 should be created.");
    let sphere_2 = create_sphere("2", "2", false, &user, &db_pool).await.expect("Sphere 2 should be created.");

    let post_1 = create_simple_post(&sphere_1.sphere_name, None, "One apple a day", "keeps the doctor away.", None, &user, &db_pool).await;
    let post_2 = create_simple_post(&sphere_1.sphere_name, None, "Bonjour", "Adieu.", None, &user, &db_pool).await;
    let post_3 = create_simple_post(&sphere_1.sphere_name, None, "Et re-bonjour", "À la prochaine.", None, &user, &db_pool).await;
    let post_4 = create_simple_post(&sphere_2.sphere_name, None, "Guten morgen", "Wie geht's?", None, &user, &db_pool).await;

    let no_match_post_vec = search_post("no match", &db_pool).await.expect("No match search should run");
    assert!(no_match_post_vec.is_empty());
    
    let apple_post_vec = search_post("apple", &db_pool).await.expect("Apple search should run");
    assert_eq!(apple_post_vec.len(), 1);
    assert_eq!(apple_post_vec.first(), Some(&post_1));

    let bonjour_post_vec = search_post("bonjour", &db_pool).await.expect("Bonjour search should run");
    assert_eq!(bonjour_post_vec.len(), 2);
    assert_eq!(bonjour_post_vec.first(), Some(&post_2));
    assert_eq!(bonjour_post_vec.get(1), Some(&post_3));

    let geht_post_vec = search_post("geht", &db_pool).await.expect("Geht search should run");
    assert_eq!(geht_post_vec.len(), 1);
    assert_eq!(geht_post_vec.first(), Some(&post_4));
}