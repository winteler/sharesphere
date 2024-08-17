use std::ops::Add;

use chrono::Days;

use app::comment::ssr::create_comment;
use app::errors::AppError;
use app::forum::ssr::create_forum;
use app::forum_management::ssr;
use app::forum_management::ssr::{ban_user_from_forum, get_forum_ban_vec, is_user_forum_moderator, moderate_comment, moderate_post, remove_user_ban};
use app::post::ssr::create_post;
use app::role::AdminRole;
use app::role::ssr::set_user_admin_role;
use app::user::User;

use crate::common::*;

mod common;

#[tokio::test]
async fn test_get_forum_rule_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum_1 = create_forum("1", "a", false, &user, &db_pool).await?;
    let forum_2 = create_forum("2", "b", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let common_rule = ssr::set_rule(None, 0, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let forum_1_rule_1 = ssr::set_rule(Some(&forum_1.forum_name), 1, "forum_1_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");
    let forum_1_rule_2 = ssr::set_rule(Some(&forum_1.forum_name), 2, "forum_1_rule_2", "test", &user, &db_pool).await.expect("Rule should be created.");
    let forum_2_rule_1 = ssr::set_rule(Some(&forum_2.forum_name), 1, "forum_2_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");

    let forum_1_rule_vec = ssr::get_forum_rule_vec(&forum_1.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_1_rule_vec.len(), 3);
    assert_eq!(forum_1_rule_vec.get(0), Some(&common_rule));
    assert_eq!(forum_1_rule_vec.get(1), Some(&forum_1_rule_1));
    assert_eq!(forum_1_rule_vec.get(2), Some(&forum_1_rule_2));

    let forum_2_rule_vec = ssr::get_forum_rule_vec(&forum_2.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_2_rule_vec.len(), 2);
    assert_eq!(forum_2_rule_vec.get(0), Some(&common_rule));
    assert_eq!(forum_2_rule_vec.get(1), Some(&forum_2_rule_1));

    Ok(())
}

#[tokio::test]
async fn test_set_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";

    assert_eq!(ssr::set_rule(None, 0, title, description, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(ssr::set_rule(None, 0, title, description, &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let common_rule = ssr::set_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(common_rule.forum_id, None);
    assert_eq!(common_rule.forum_name, None);
    assert_eq!(common_rule.priority, 0);
    assert_eq!(common_rule.title, title);
    assert_eq!(common_rule.description, description);
    assert_eq!(common_rule.user_id, admin.user_id);

    assert_eq!(ssr::set_rule(Some(&forum.forum_name), 1, title, description, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let rule_1 = ssr::set_rule(Some(&forum.forum_name), 1, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    let rule_2 = ssr::set_rule(Some(&forum.forum_name), 2, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(rule_1.forum_id, Some(forum.forum_id));
    assert_eq!(rule_1.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_1.priority, 1);
    assert_eq!(rule_1.title, title);
    assert_eq!(rule_1.description, description);
    assert_eq!(rule_1.user_id, lead.user_id);

    assert_eq!(rule_2.forum_id, Some(forum.forum_id));
    assert_eq!(rule_2.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_2.priority, 2);
    assert_eq!(rule_2.title, title);
    assert_eq!(rule_2.description, description);
    assert_eq!(rule_2.user_id, admin.user_id);

    Ok(())
}

#[tokio::test]
async fn test_get_forum_ban_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead = create_user("test", &db_pool).await;
    let banned_user_1 = create_user("1", &db_pool).await;
    let banned_user_2 = create_user("2", &db_pool).await;

    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let ban_user_1 = ban_user_from_forum(
        banned_user_1.user_id,&
        forum.forum_name,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    let ban_user_2 = ban_user_from_forum(
        banned_user_2.user_id,&
        forum.forum_name,
        &lead,
        Some(7),
        &db_pool
    ).await.expect("User 2 should be banned").expect("User 2 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 2);
    assert!(banned_user_vec.contains(&ban_user_1));
    assert!(banned_user_vec.contains(&ban_user_2));

    Ok(())
}

#[tokio::test]
async fn test_remove_user_ban() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead = create_user("test", &db_pool).await;
    let mut global_mod = create_user("global", &db_pool).await;
    global_mod.admin_role = AdminRole::Moderator;
    let banned_user_1 = create_user("1", &db_pool).await;

    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let ban_user_1 = ban_user_from_forum(
        banned_user_1.user_id,&
        forum.forum_name,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    assert_eq!(remove_user_ban(ban_user_1.ban_id, &banned_user_1, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_user_ban(ban_user_1.ban_id, &lead, &db_pool).await, Ok(ban_user_1));

    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, &db_pool).await.expect("Should load forum bans");
    assert!(banned_user_vec.is_empty());

    let ban_user_1 = ban_user_from_forum(
        banned_user_1.user_id,&
        forum.forum_name,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    assert_eq!(remove_user_ban(ban_user_1.ban_id, &global_mod, &db_pool).await, Ok(ban_user_1));

    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, &db_pool).await.expect("Should load forum bans");
    assert!(banned_user_vec.is_empty());

    // TODO add test to remove global ban when possible to create it

    Ok(())
}

#[tokio::test]
async fn test_moderate_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, &db_pool).await?;
    let post = create_post(&forum.forum_name, "a", "body", None, false, None, &test_user, &db_pool).await?;

    assert!(moderate_post(post.post_id, "unauthorized", &unauthorized_user, &db_pool).await.is_err());

    let moderated_post = moderate_post(post.post_id, "test", &test_user, &db_pool).await?;
    assert_eq!(moderated_post.moderator_id, Some(test_user.user_id));
    assert_eq!(moderated_post.moderator_name, Some(test_user.username));
    assert_eq!(moderated_post.moderated_body, Some(String::from("test")));

    let remoderated_post = moderate_post(post.post_id, "global", &global_moderator, &db_pool).await?;
    assert_eq!(remoderated_post.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_post.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_post.moderated_body, Some(String::from("global")));

    Ok(())
}

#[tokio::test]
async fn test_moderate_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, &db_pool).await?;
    let post = create_post(&forum.forum_name, "a", "body", None, false, None, &test_user, &db_pool).await?;
    let comment = create_comment(post.post_id, None, "comment", None, &test_user, &db_pool).await?;

    assert!(moderate_comment(comment.comment_id, "unauthorized", &unauthorized_user, &db_pool).await.is_err());

    let moderated_comment = moderate_comment(comment.comment_id, "test", &test_user, &db_pool).await?;
    assert_eq!(moderated_comment.moderator_id, Some(test_user.user_id));
    assert_eq!(moderated_comment.moderator_name, Some(test_user.username));
    assert_eq!(moderated_comment.moderated_body, Some(String::from("test")));

    let remoderated_comment = moderate_comment(comment.comment_id, "global", &global_moderator, &db_pool).await?;
    assert_eq!(remoderated_comment.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_comment.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_comment.moderated_body, Some(String::from("global")));

    Ok(())
}

#[tokio::test]
async fn test_ban_user_from_forum() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    // set user role in the DB, needed to test that global Moderators/Admin cannot be banned
    global_moderator.admin_role = AdminRole::Moderator;
    admin.admin_role = AdminRole::Admin;
    set_user_admin_role(global_moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await?;
    set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await?;
    let unauthorized_user = create_user("user", &db_pool).await;
    let banned_user = create_user("banned", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, &db_pool).await?;
    let test_user = User::get(test_user.user_id, &db_pool).await.expect("Should be able to reload user.");

    // unauthorized used cannot ban
    assert!(ban_user_from_forum(banned_user.user_id, &forum.forum_name, &unauthorized_user, None, &db_pool).await.is_err());
    // ban with 0 days has no effect
    assert_eq!(ban_user_from_forum(unauthorized_user.user_id, &forum.forum_name, &test_user, Some(0), &db_pool).await?, None);
    let post = create_post(&forum.forum_name, "a", "b", None, false, None, &unauthorized_user, &db_pool).await?;

    // cannot ban moderators
    assert!(ban_user_from_forum(test_user.user_id, &forum.forum_name, &global_moderator, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_forum(global_moderator.user_id, &forum.forum_name, &test_user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_forum(admin.user_id, &forum.forum_name, &test_user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_forum(test_user.user_id, &forum.forum_name, &admin, Some(1), &db_pool).await.is_err());

    // forum moderator can ban ordinary users
    let user_ban = ban_user_from_forum(unauthorized_user.user_id, &forum.forum_name, &test_user, Some(1), &db_pool).await?.expect("User ban from forum should be possible.");
    assert_eq!(user_ban.user_id, unauthorized_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, test_user.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(1))));

    // banned user cannot create new content
    let unauthorized_user = User::get(unauthorized_user.user_id, &db_pool).await.expect("Should be able to reload user.");
    assert!(create_post(&forum.forum_name, "c", "d", None, false, None, &unauthorized_user, &db_pool).await.is_err());
    assert!(create_comment(post.post_id, None, "a", None, &unauthorized_user, &db_pool).await.is_err());

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_forum(banned_user.user_id, &forum.forum_name, &global_moderator, Some(2), &db_pool).await?.expect("User ban from forum should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, global_moderator.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(2))));

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_forum(banned_user.user_id, &forum.forum_name, &admin, None, &db_pool).await?.expect("User ban from forum should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, admin.user_id);
    assert_eq!(user_ban.until_timestamp, None);

    // banned user cannot create new content
    let banned_user = User::get(banned_user.user_id, &db_pool).await.expect("Should be possible to reload banned user.");
    assert!(create_post(&forum.forum_name, "c", "d", None, false, None, &banned_user, &db_pool).await.is_err());
    assert!(create_comment(post.post_id, None, "a", None, &banned_user, &db_pool).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_is_user_forum_moderator() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let test_user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    // set user role in the DB, needed to test that global Moderators/Admin cannot be banned
    global_moderator.admin_role = AdminRole::Moderator;
    admin.admin_role = AdminRole::Admin;
    set_user_admin_role(global_moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await?;
    set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await?;
    let ordinary_user = create_user("user", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, &db_pool).await?;

    assert_eq!(is_user_forum_moderator(test_user.user_id, &forum.forum_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_forum_moderator(global_moderator.user_id, &forum.forum_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_forum_moderator(admin.user_id, &forum.forum_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_forum_moderator(ordinary_user.user_id, &forum.forum_name, &db_pool).await, Ok(false));
    assert!(is_user_forum_moderator(ordinary_user.user_id + 1, &forum.forum_name, &db_pool).await.is_err());

    Ok(())
}