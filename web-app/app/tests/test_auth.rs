use std::ops::Add;

use chrono::Days;
use leptos::ServerFnError;

use app::auth::ssr::SqlUser;
use app::auth::User;
use app::errors::AppError;
use app::forum::ssr::create_forum;
use app::forum_management::ssr::ban_user_from_forum;
use app::role::{AdminRole, PermissionLevel};
use app::role::ssr::set_user_forum_role;

use crate::common::{create_test_user, create_user, get_db_pool};

mod common;

#[tokio::test]
async fn test_sql_user_get_from_oidc_id() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let oidc_id = "id";
    let username = "username";
    let email = "user@user.com";
    let user = create_user(oidc_id, username, email, &db_pool).await;
    let sql_user = SqlUser::get_from_oidc_id(&user.oidc_id, db_pool).await?;

    assert_eq!(sql_user.user_id, user.user_id);
    assert_eq!(sql_user.oidc_id, oidc_id);
    assert_eq!(sql_user.username, username);
    assert_eq!(sql_user.email, email);
    assert_eq!(sql_user.admin_role, AdminRole::None);

    Ok(())
}

#[tokio::test]
async fn test_user_get() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let creator_user = create_user("user", "user", "user@user.com", &db_pool).await;
    let test_user = create_test_user(&db_pool).await;

    let forum_a = create_forum("a", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_b = create_forum("b", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_c = create_forum("c", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_d = create_forum("d", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_e = create_forum("e", "test", false, &creator_user, db_pool.clone()).await?;

    // reload creator_user so that it has the updated roles after creating forums.
    let creator_user = User::get(creator_user.user_id, &db_pool).await.expect("Could not get creator user.");

    set_user_forum_role(forum_a.forum_id, &forum_a.forum_name, test_user.user_id, PermissionLevel::Moderate, &creator_user, &db_pool).await?;
    set_user_forum_role(forum_b.forum_id, &forum_b.forum_name, test_user.user_id, PermissionLevel::Elect, &creator_user, &db_pool).await?;

    ban_user_from_forum(test_user.user_id, &forum_c.forum_name, &creator_user, Some(0), db_pool.clone()).await?;
    let forum_ban_d = ban_user_from_forum(test_user.user_id, &forum_d.forum_name, &creator_user, Some(1), db_pool.clone()).await?.expect("Expected forum ban for forum d.");
    ban_user_from_forum(test_user.user_id, &forum_e.forum_name, &creator_user, None, db_pool.clone()).await?.expect("Expected forum ban for forum e.");

    let result_user = User::get(test_user.user_id, &db_pool).await.expect("Could not get result user.");

    assert_eq!(result_user.check_can_moderate_forum(&forum_a.forum_name), Ok(()));
    assert_eq!(result_user.check_can_moderate_forum(&forum_b.forum_name), Ok(()));
    assert_eq!(result_user.check_can_moderate_forum(&forum_c.forum_name), Err(AppError::InsufficientPrivileges));
    assert_eq!(result_user.check_can_moderate_forum(&forum_d.forum_name), Err(AppError::InsufficientPrivileges));
    assert_eq!(result_user.check_can_moderate_forum(&forum_e.forum_name), Err(AppError::InsufficientPrivileges));

    assert_eq!(result_user.check_can_publish_on_forum(&forum_a.forum_name), Ok(()));
    assert_eq!(result_user.check_can_publish_on_forum(&forum_b.forum_name), Ok(()));
    assert_eq!(result_user.check_can_publish_on_forum(&forum_c.forum_name), Ok(()));
    assert_eq!(result_user.check_can_publish_on_forum(&forum_d.forum_name), Err(AppError::ForumBanUntil(forum_ban_d.create_timestamp.add(Days::new(1)))));
    assert_eq!(result_user.check_can_publish_on_forum(&forum_e.forum_name), Err(AppError::PermanentForumBan));

    // TODO test global ban when ssr function is implemented

    Ok(())
}