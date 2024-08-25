use std::ops::Add;

use chrono::Days;

use app::errors::AppError;
use app::forum_management::ssr::ban_user_from_forum;
use app::role::ssr::set_user_forum_role;
use app::role::{AdminRole, PermissionLevel};
use app::user::{ssr::SqlUser, User};
use app::{forum, forum_management};

use crate::common::{create_user, get_db_pool};
use crate::data_factory::create_forum_with_post;

mod common;
mod data_factory;

#[tokio::test]
async fn test_sql_user_get_by_username() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let oidc_id = "id";
    let username = "username";
    let email = "user@user.com";
    let user = app::user::ssr::create_user(oidc_id, username, email, &db_pool).await.expect("Sql user should be created");
    let sql_user = SqlUser::get_by_username(&user.username, &db_pool).await?;

    assert_eq!(sql_user.user_id, user.user_id);
    assert_eq!(sql_user.oidc_id, oidc_id);
    assert_eq!(sql_user.username, username);
    assert_eq!(sql_user.email, email);
    assert_eq!(sql_user.admin_role, AdminRole::None);

    Ok(())
}

#[tokio::test]
async fn test_sql_user_get_from_oidc_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let oidc_id = "id";
    let username = "username";
    let email = "user@user.com";
    let user = app::user::ssr::create_user(oidc_id, username, email, &db_pool).await.expect("Sql user should be created");
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
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;

    // Create common rule to enable bans
    let rule = forum_management::ssr::add_rule(None, 0, "test", "test", &admin, &db_pool).await.expect("Rule should be added.");

    let (forum_a, _post_a) = create_forum_with_post("a", &creator_user, &db_pool).await;
    let (forum_b, _post_b) = create_forum_with_post("b", &creator_user, &db_pool).await;
    let (forum_c, post_c) = create_forum_with_post("c", &creator_user, &db_pool).await;
    let (forum_d, post_d) = create_forum_with_post("d", &creator_user, &db_pool).await;
    let (forum_e, post_e) = create_forum_with_post("e", &creator_user, &db_pool).await;

    // reload creator_user so that it has the updated roles after creating forums.
    let creator_user = User::get(creator_user.user_id, &db_pool).await.expect("Creator user should be created.");

    set_user_forum_role(test_user.user_id, &forum_a.forum_name, PermissionLevel::Moderate, &creator_user, &db_pool).await?;
    set_user_forum_role(test_user.user_id, &forum_b.forum_name, PermissionLevel::Manage, &creator_user, &db_pool).await?;

    assert_eq!(
        ban_user_from_forum(test_user.user_id, &forum_c.forum_name, post_c.post_id, None, rule.rule_id, &creator_user, Some(0), &db_pool).await.expect("User ban should be created for forum c."),
        None
    );
    let forum_ban_d = ban_user_from_forum(test_user.user_id, &forum_d.forum_name, post_d.post_id, None, rule.rule_id, &creator_user, Some(1), &db_pool).await
        ?
        .expect("User should have ban for forum d.");
    ban_user_from_forum(test_user.user_id, &forum_e.forum_name, post_e.post_id, None, rule.rule_id, &creator_user, None, &db_pool).await
        .expect("User ban should be created for forum e.")
        .expect("User should have ban for forum e.");

    let result_user = User::get(test_user.user_id, &db_pool).await.expect("result_user should be available in DB.");

    assert_eq!(result_user.check_permissions(&forum_a.forum_name, PermissionLevel::Moderate), Ok(()));
    assert_eq!(result_user.check_permissions(&forum_b.forum_name, PermissionLevel::Moderate), Ok(()));
    assert_eq!(result_user.check_permissions(&forum_c.forum_name, PermissionLevel::Moderate), Err(AppError::InsufficientPrivileges));
    assert_eq!(result_user.check_permissions(&forum_d.forum_name, PermissionLevel::Moderate), Err(AppError::InsufficientPrivileges));
    assert_eq!(result_user.check_permissions(&forum_e.forum_name, PermissionLevel::Moderate), Err(AppError::InsufficientPrivileges));

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
    let manage_mod = create_user("elect", &db_pool).await;
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
    set_user_forum_role(manage_mod.user_id, &forum.forum_name, PermissionLevel::Manage, &lead_user, &db_pool)
        .await.expect("Moderate role should be assignable by lead_user.");
    set_user_forum_role(simple_mod.user_id, &forum.forum_name, PermissionLevel::Ban, &lead_user, &db_pool)
        .await.expect("Moderate role should be assignable by lead_user.");
    let manage_mod = User::get(manage_mod.user_id, &db_pool).await.expect("Should be able to get elect mod.");
    let simple_mod = User::get(simple_mod.user_id, &db_pool).await.expect("Should be able to get simple mod.");
    admin.admin_role = AdminRole::Admin;
    global_moderator.admin_role = AdminRole::Moderator;

    // normal user, simple moderator, global moderator cannot set any role
    assert_eq!(
        std_user.check_can_set_user_forum_role(PermissionLevel::Moderate, test_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        simple_mod.check_can_set_user_forum_role(PermissionLevel::Moderate, test_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        global_moderator.check_can_set_user_forum_role(PermissionLevel::Moderate, test_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );

    // manage mods can set user role for normal users, moderators with a lower level but not manage mods and leaders
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Ban, test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::None, simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Ban, simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Manage, simple_mod.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Lead, simple_mod.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Moderate, manage_mod.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Moderate, lead_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        manage_mod.check_can_set_user_forum_role(PermissionLevel::Manage, lead_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );

    // lead users and admin can set user role for everyone (lead user cannot set for himself, as this function is not used for leader changes)
    assert_eq!(
        lead_user.check_can_set_user_forum_role(PermissionLevel::Ban, test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(PermissionLevel::Manage, test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(PermissionLevel::Manage, simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(PermissionLevel::None, manage_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        lead_user.check_can_set_user_forum_role(PermissionLevel::Manage, lead_user.user_id, forum_name, &db_pool).await,
        Err(AppError::InsufficientPrivileges)
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(PermissionLevel::Lead, test_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(PermissionLevel::Manage, simple_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(PermissionLevel::Moderate, manage_mod.user_id, forum_name, &db_pool).await,
        Ok(())
    );
    assert_eq!(
        admin.check_can_set_user_forum_role(PermissionLevel::None, lead_user.user_id, forum_name, &db_pool).await,
        Ok(())
    );

    Ok(())
}

#[tokio::test]
async fn test_create_user() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user_1_value = "1";
    let sql_user_1 = app::user::ssr::create_user(user_1_value, user_1_value, user_1_value, &db_pool).await.expect("Sql user 1 should be created");
    assert_eq!(sql_user_1.oidc_id, user_1_value);
    assert_eq!(sql_user_1.username, user_1_value);
    assert_eq!(sql_user_1.email, user_1_value);
    assert_eq!(sql_user_1.admin_role, AdminRole::None);
    assert_eq!(sql_user_1.is_deleted, false);

    // test cannot create user with duplicate oidc_id, username or email
    let user_2_value = "2";
    assert!(app::user::ssr::create_user(user_1_value, user_2_value, user_2_value, &db_pool).await.is_err());
    assert!(app::user::ssr::create_user(user_2_value, user_1_value, user_2_value, &db_pool).await.is_err());
    assert!(app::user::ssr::create_user(user_2_value, user_2_value, user_1_value, &db_pool).await.is_err());

    let sql_user_2 = app::user::ssr::create_user(user_2_value, user_2_value, user_2_value, &db_pool).await.expect("Sql user 2 should be created");
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