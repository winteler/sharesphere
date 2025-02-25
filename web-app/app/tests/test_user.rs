use crate::common::{create_test_user, get_db_pool};
use app::errors::AppError;
use app::role::AdminRole;
use app::user::ssr::{create_or_update_user, set_user_settings};
use app::user::User;

mod common;

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
    assert_eq!(user.show_nsfw, true);
    assert_eq!(user.days_hide_spoiler, None);

    let loaded_user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(loaded_user.user_id, user.user_id);
    assert_eq!(loaded_user.oidc_id, user.oidc_id);
    assert_eq!(loaded_user.username, user.username);
    assert_eq!(loaded_user.email, user.email);
    assert_eq!(loaded_user.admin_role, user.admin_role);
    assert_eq!(loaded_user.show_nsfw, user.show_nsfw);
    assert_eq!(loaded_user.days_hide_spoiler, user.days_hide_spoiler);

    Ok(())
}

#[tokio::test]
async fn test_set_user_settings() {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    
    set_user_settings(true, true, None, &user, &db_pool).await.expect("Should set user settings");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.is_nsfw, true);
    assert_eq!(user.show_nsfw, true);
    assert_eq!(user.days_hide_spoiler, None);

    set_user_settings(true, false, Some(1), &user, &db_pool).await.expect("Should set user settings");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.is_nsfw, true);
    assert_eq!(user.show_nsfw, false);
    assert_eq!(user.days_hide_spoiler, Some(1));

    set_user_settings(false, true, Some(10), &user, &db_pool).await.expect("Should set user settings");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.is_nsfw, false);
    assert_eq!(user.show_nsfw, true);
    assert_eq!(user.days_hide_spoiler, Some(10));

    set_user_settings(false, false, None, &user, &db_pool).await.expect("Should set user preferences");
    let user = User::get(user.user_id, &db_pool).await.expect("Should get user");
    assert_eq!(user.is_nsfw, false);
    assert_eq!(user.show_nsfw, false);
    assert_eq!(user.days_hide_spoiler, None);

    assert!(set_user_settings(false, false, Some(0), &user, &db_pool).await.is_err());
    assert!(set_user_settings(false, true, Some(-1), &user, &db_pool).await.is_err());
}
