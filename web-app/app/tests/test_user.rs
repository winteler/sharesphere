use std::collections::BTreeSet;

use crate::common::{create_test_user, create_user, get_db_pool};
use app::errors::AppError;
use app::role::AdminRole;
use app::user;
use app::user::ssr::{create_or_update_user, set_user_preferences};
use app::user::User;

mod common;

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

    let username_set = user::ssr::get_matching_username_set("1", num_users as i64, &db_pool).await?;

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

    let username_set = user::ssr::get_matching_username_set("", num_users as i64, &db_pool).await?;

    assert_eq!(username_set.len(), num_users);

    Ok(())
}

#[tokio::test]
async fn test_create_or_update_user() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;

    let oidc_id = "a";
    let username = "b";
    let email = "c";

    let user = create_or_update_user(oidc_id, username, email, &db_pool).await.expect("Should create user");

    assert_eq!(user.oidc_id, oidc_id);
    assert_eq!(user.username, username);
    assert_eq!(user.email, email);
    assert_eq!(user.admin_role, AdminRole::None);
    assert_eq!(user.hide_nsfw, false);
    assert_eq!(user.days_hide_spoiler, None);

    let loaded_user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(loaded_user.user_id, user.user_id);
    assert_eq!(loaded_user.oidc_id, user.oidc_id);
    assert_eq!(loaded_user.username, user.username);
    assert_eq!(loaded_user.email, user.email);
    assert_eq!(loaded_user.admin_role, user.admin_role);
    assert_eq!(loaded_user.hide_nsfw, user.hide_nsfw);
    assert_eq!(loaded_user.days_hide_spoiler, user.days_hide_spoiler);

    Ok(())
}

#[tokio::test]
async fn test_set_user_preferences() {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    
    set_user_preferences(true, None, &user, &db_pool).await.expect("Should set user preferences");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.hide_nsfw, true);
    assert_eq!(user.days_hide_spoiler, None);

    set_user_preferences(false, Some(1), &user, &db_pool).await.expect("Should set user preferences");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.hide_nsfw, false);
    assert_eq!(user.days_hide_spoiler, Some(1));

    set_user_preferences(true, Some(10), &user, &db_pool).await.expect("Should set user preferences");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.hide_nsfw, true);
    assert_eq!(user.days_hide_spoiler, Some(10));

    set_user_preferences(false, None, &user, &db_pool).await.expect("Should set user preferences");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.hide_nsfw, false);
    assert_eq!(user.days_hide_spoiler, None);

    assert!(set_user_preferences(false, Some(0), &user, &db_pool).await.is_err());
    assert!(set_user_preferences(true, Some(-1), &user, &db_pool).await.is_err());
}
