use std::ops::Add;

use chrono::Days;

use app::auth::ssr::SqlUser;
use app::auth::User;
use app::errors::AppError;
use app::forum;
use app::forum::ssr::create_forum;
use app::forum_management::ssr::ban_user_from_forum;
use app::role::{AdminRole, PermissionLevel};
use app::role::ssr::set_user_forum_role;

use crate::common::{create_user, get_db_pool};

mod common;

#[tokio::test]
async fn test_sql_user_get_from_oidc_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let oidc_id = "id";
    let username = "username";
    let email = "user@user.com";
    let user = app::auth::ssr::create_user(oidc_id, username, email, &db_pool).await.expect("Sql user should be created");
    let sql_user = SqlUser::get_from_oidc_id(&user.oidc_id, &db_pool).await?;

    assert_eq!(sql_user.user_id, user.user_id);
    assert_eq!(sql_user.oidc_id, oidc_id);
    assert_eq!(sql_user.username, username);
    assert_eq!(sql_user.email, email);
    assert_eq!(sql_user.admin_role, AdminRole::None);

    Ok(())
}

#[tokio::test]
async fn test_user_get() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let creator_user = create_user("creator", &db_pool).await;
    let test_user = create_user("user", &db_pool).await;

    let forum_a = create_forum("a", "test", false, &creator_user, &db_pool).await?;
    let forum_b = create_forum("b", "test", false, &creator_user, &db_pool).await?;
    let forum_c = create_forum("c", "test", false, &creator_user, &db_pool).await?;
    let forum_d = create_forum("d", "test", false, &creator_user, &db_pool).await?;
    let forum_e = create_forum("e", "test", false, &creator_user, &db_pool).await?;

    // reload creator_user so that it has the updated roles after creating forums.
    let creator_user = User::get(creator_user.user_id, &db_pool).await.expect("Creator user should be created.");

    set_user_forum_role(forum_a.forum_id, &forum_a.forum_name, test_user.user_id, PermissionLevel::Moderate, &creator_user, &db_pool).await?;
    set_user_forum_role(forum_b.forum_id, &forum_b.forum_name, test_user.user_id, PermissionLevel::Elect, &creator_user, &db_pool).await?;

    ban_user_from_forum(test_user.user_id, &forum_c.forum_name, &creator_user, Some(0), &db_pool).await?;
    let forum_ban_d = ban_user_from_forum(test_user.user_id, &forum_d.forum_name, &creator_user, Some(1), &db_pool).await?.expect("User should have ban for forum d.");
    ban_user_from_forum(test_user.user_id, &forum_e.forum_name, &creator_user, None, &db_pool).await?.expect("User should have ban for forum e.");

    let result_user = User::get(test_user.user_id, &db_pool).await.expect("result_user should be available in DB.");

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

#[tokio::test]
async fn test_user_check_can_set_user_forum_role() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead_user = create_user("lead", &db_pool).await;
    let elect_mod = create_user("elect", &db_pool).await;
    let simple_mod = create_user("mod", &db_pool).await;
    let mut global_moderator = create_user("gmod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    let std_user  = create_user("std", &db_pool).await;
    let test_user = create_user("test", &db_pool).await;

    let forum_name = "forum";
    let forum = forum::ssr::create_forum(forum_name, "forum", false, &lead_user, &db_pool).await?;
    let lead_user = User::get(lead_user.user_id, &db_pool)
        .await
        .expect("Should be able to reload lead_user.");

    // set user roles
    set_user_forum_role(forum.forum_id, forum_name, elect_mod.user_id, PermissionLevel::Elect, &lead_user, &db_pool)
        .await.expect("Moderate role should be assignable by lead_user.");
    set_user_forum_role(forum.forum_id, forum_name, simple_mod.user_id, PermissionLevel::Configure, &lead_user, &db_pool)
        .await.expect("Moderate role should be assignable by lead_user.");
    let elect_mod = User::get(elect_mod.user_id, &db_pool).await.expect("Should be able to get elect mod.");
    let simple_mod = User::get(simple_mod.user_id, &db_pool).await.expect("Should be able to get simple mod.");
    admin.admin_role = AdminRole::Admin;
    global_moderator.admin_role = AdminRole::Moderator;

    // normal user, simple moderator, global moderator cannot set any role
    assert_eq!(
        std_user.check_can_set_user_forum_role(test_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        simple_mod.check_can_set_user_forum_role(test_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        global_moderator.check_can_set_user_forum_role(test_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );

    // elect mods can set user role for normal users, moderators with a lower level but not elect mods and leaders
    assert_eq!(
        elect_mod.check_can_set_user_forum_role(test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        elect_mod.check_can_set_user_forum_role(simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        elect_mod.check_can_set_user_forum_role(elect_mod.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        elect_mod.check_can_set_user_forum_role(lead_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );

    // lead users and admin can set user role for everyone (lead user cannot set for himself, as this function is not used for leader changes)
    assert_eq!(
        lead_user.check_can_set_user_forum_role(test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(elect_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(lead_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(elect_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(lead_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );

    Ok(())
}

#[tokio::test]
async fn test_create_user() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user_1_value = "1";
    let sql_user_1 = app::auth::ssr::create_user(user_1_value, user_1_value, user_1_value, &db_pool).await.expect("Sql user 1 should be created");
    assert_eq!(sql_user_1.oidc_id, user_1_value);
    assert_eq!(sql_user_1.username, user_1_value);
    assert_eq!(sql_user_1.email, user_1_value);
    assert_eq!(sql_user_1.admin_role, AdminRole::None);
    assert_eq!(sql_user_1.is_deleted, false);

    // test cannot create user with duplicate oidc_id, username or email
    let user_2_value = "2";
    assert!(app::auth::ssr::create_user(user_1_value, user_2_value, user_2_value, &db_pool).await.is_err());
    assert!(app::auth::ssr::create_user(user_2_value, user_1_value, user_2_value, &db_pool).await.is_err());
    assert!(app::auth::ssr::create_user(user_2_value, user_2_value, user_1_value, &db_pool).await.is_err());

    let sql_user_2 = app::auth::ssr::create_user(user_2_value, user_2_value, user_2_value, &db_pool).await.expect("Sql user 2 should be created");
    assert_eq!(sql_user_2.oidc_id, user_2_value);
    assert_eq!(sql_user_2.username, user_2_value);
    assert_eq!(sql_user_2.email, user_2_value);
    assert_eq!(sql_user_2.admin_role, AdminRole::None);
    assert_eq!(sql_user_2.is_deleted, false);

    let user_1 = User::get(sql_user_1.user_id, &db_pool).await.expect("Should be able to get user 1");
    assert_eq!(user_1.user_id, sql_user_1.user_id);
    assert_eq!(user_1.oidc_id, sql_user_1.oidc_id);
    assert_eq!(user_1.username, sql_user_1.username);
    assert_eq!(user_1.email, sql_user_1.email);
    assert_eq!(user_1.admin_role, sql_user_1.admin_role);
    assert_eq!(user_1.is_deleted, sql_user_1.is_deleted);

    Ok(())
}