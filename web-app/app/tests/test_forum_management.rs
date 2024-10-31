use std::ops::Add;

use chrono::Days;

use app::comment::ssr::create_comment;
use app::errors::AppError;
use app::forum::ssr::{create_forum, get_forum_by_name};
use app::forum_management::ssr::{add_rule, get_forum_ban_vec, get_forum_rule_vec, is_user_forum_moderator, load_rule_by_id, remove_rule, remove_user_ban, set_banner_url, update_rule};
use app::moderation::ssr::{ban_user_from_forum, moderate_comment, moderate_post};
use app::post::ssr::create_post;
use app::role::ssr::set_user_admin_role;
use app::role::AdminRole;
use app::user::User;

use crate::common::*;
use crate::data_factory::{create_forum_with_post, create_forum_with_post_and_comment};

mod common;
mod data_factory;

#[tokio::test]
async fn test_get_rule_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum_1 = create_forum("1", "a", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let expected_common_rule = add_rule(None, 0, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let expected_forum_rule = add_rule(Some(&forum_1.forum_name), 1, "forum_1_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");
    
    let common_rule = load_rule_by_id(expected_common_rule.rule_id, &db_pool).await?;
    let forum_rule = load_rule_by_id(expected_forum_rule.rule_id, &db_pool).await?;

    assert_eq!(common_rule, expected_common_rule);
    assert_eq!(forum_rule, expected_forum_rule);
    
    Ok(())
}

#[tokio::test]
async fn test_get_forum_rule_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum_1 = create_forum("1", "a", false, &user, &db_pool).await?;
    let forum_2 = create_forum("2", "b", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let common_rule_1 = add_rule(None, 0, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_2 = add_rule(None, 3, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let forum_1_rule_1 = add_rule(Some(&forum_1.forum_name), 1, "forum_1_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");
    let forum_1_rule_2 = add_rule(Some(&forum_1.forum_name), 2, "forum_1_rule_2", "test", &user, &db_pool).await.expect("Rule should be created.");
    let forum_2_rule_1 = add_rule(Some(&forum_2.forum_name), 1, "forum_2_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");

    let forum_1_rule_vec = get_forum_rule_vec(&forum_1.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_1_rule_vec.len(), 4);
    assert_eq!(forum_1_rule_vec.get(0), Some(&common_rule_1));
    assert_eq!(forum_1_rule_vec.get(1), Some(&common_rule_2));
    assert_eq!(forum_1_rule_vec.get(2), Some(&forum_1_rule_1));
    assert_eq!(forum_1_rule_vec.get(3), Some(&forum_1_rule_2));

    let forum_2_rule_vec = get_forum_rule_vec(&forum_2.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_2_rule_vec.len(), 3);
    assert_eq!(forum_2_rule_vec.get(0), Some(&common_rule_1));
    assert_eq!(forum_2_rule_vec.get(1), Some(&common_rule_2));
    assert_eq!(forum_2_rule_vec.get(2), Some(&forum_2_rule_1));

    Ok(())
}

#[tokio::test]
async fn test_add_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";

    assert_eq!(add_rule(None, 0, title, description, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(add_rule(None, 0, title, description, &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let common_rule_1 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(common_rule_1.forum_id, None);
    assert_eq!(common_rule_1.forum_name, None);
    assert_eq!(common_rule_1.priority, 0);
    assert_eq!(common_rule_1.title, title);
    assert_eq!(common_rule_1.description, description);
    assert_eq!(common_rule_1.user_id, admin.user_id);

    assert_eq!(add_rule(Some(&forum.forum_name), 1, title, description, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let rule_1 = add_rule(Some(&forum.forum_name), 1, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    // creating rule_2 should increment rule_1's priority
    let rule_2 = add_rule(Some(&forum.forum_name), 1, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(rule_1.forum_id, Some(forum.forum_id));
    assert_eq!(rule_1.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_1.priority, 1);
    assert_eq!(rule_1.title, title);
    assert_eq!(rule_1.description, description);
    assert_eq!(rule_1.user_id, lead.user_id);

    assert_eq!(rule_2.forum_id, Some(forum.forum_id));
    assert_eq!(rule_2.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_2.priority, 1);
    assert_eq!(rule_2.title, title);
    assert_eq!(rule_2.description, description);
    assert_eq!(rule_2.user_id, admin.user_id);

    let common_rule_2 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 4);
    assert_eq!(forum_rule_vec.get(0), Some(&common_rule_2));
    assert_eq!(forum_rule_vec.get(1).unwrap().rule_id, common_rule_1.rule_id);
    assert_eq!(forum_rule_vec.get(2), Some(&rule_2));
    assert_eq!(forum_rule_vec.get(3).unwrap().rule_id, rule_1.rule_id);

    Ok(())
}

#[tokio::test]
async fn test_update_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";
    let updated_title = "updated";
    let updated_desc = "updated";

    let common_rule_1 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_2 = add_rule(None, 1, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_3 = add_rule(None, 2, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let rule_1 = add_rule(Some(&forum.forum_name), 0, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    let rule_2 = add_rule(Some(&forum.forum_name), 1, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let rule_3 = add_rule(Some(&forum.forum_name), 2, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(update_rule(None, 0, 1, updated_title, updated_desc, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(update_rule(None, 0, 1, updated_title, updated_desc,  &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let common_rule_1_updated = update_rule(None, 0, 1, updated_title, updated_desc, &admin, &db_pool).await.expect("Rule should be updated.");
    assert_eq!(common_rule_1_updated.rule_key, common_rule_1.rule_key);
    assert_eq!(common_rule_1_updated.priority, 1);
    assert_eq!(common_rule_1_updated.forum_id, None);
    assert_eq!(common_rule_1_updated.forum_name, None);
    assert_eq!(common_rule_1_updated.title, updated_title);
    assert_eq!(common_rule_1_updated.description, updated_desc);

    assert_eq!(update_rule(Some(&forum.forum_name), 1, 0, updated_title, updated_desc, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let rule_2_updated = update_rule(Some(&forum.forum_name), 1, 0, updated_title, updated_desc,  &lead, &db_pool).await.expect("Rule should be updated.");
    assert_eq!(rule_2_updated.rule_key, rule_2.rule_key);
    assert_eq!(rule_2_updated.priority, 0);
    assert_eq!(rule_2_updated.forum_id, Some(forum.forum_id));
    assert_eq!(rule_2_updated.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_2_updated.title, updated_title);
    assert_eq!(rule_2_updated.description, updated_desc);
    let rule_3_updated = update_rule(Some(&forum.forum_name), 2, 1, updated_title, updated_desc, &admin, &db_pool).await.expect("Rule should be updated.");
    assert_eq!(rule_3_updated.rule_key, rule_3.rule_key);
    assert_eq!(rule_3_updated.priority, 1);
    assert_eq!(rule_3_updated.forum_id, Some(forum.forum_id));
    assert_eq!(rule_3_updated.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_3_updated.title, updated_title);
    assert_eq!(rule_3_updated.description, updated_desc);

    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 6);
    assert_eq!(forum_rule_vec.get(0).unwrap().rule_id, common_rule_2.rule_id);
    assert_eq!(forum_rule_vec.get(1), Some(&common_rule_1_updated));
    assert_eq!(forum_rule_vec.get(2), Some(&common_rule_3));
    assert_eq!(forum_rule_vec.get(3), Some(&rule_2_updated));
    assert_eq!(forum_rule_vec.get(4), Some(&rule_3_updated));
    assert_eq!(forum_rule_vec.get(5).unwrap().rule_id, rule_1.rule_id);

    Ok(())
}

#[tokio::test]
async fn test_remove_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";

    let _common_rule_1 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_2 = add_rule(None, 1, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let _rule_1 = add_rule(Some(&forum.forum_name), 0, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    let rule_2 = add_rule(Some(&forum.forum_name), 1, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(remove_rule(None, 0, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_rule(None, 0, &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_rule(None, 0, &admin, &db_pool).await, Ok(()));

    assert_eq!(remove_rule(Some(&forum.forum_name), 0, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_rule(Some(&forum.forum_name), 0, &lead, &db_pool).await, Ok(()));

    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 2);
    assert_eq!(forum_rule_vec.get(0).unwrap().rule_id, common_rule_2.rule_id);
    assert_eq!(forum_rule_vec.get(0).unwrap().priority, 0);
    assert_eq!(forum_rule_vec.get(1).unwrap().rule_id, rule_2.rule_id);
    assert_eq!(forum_rule_vec.get(1).unwrap().priority, 0);

    assert_eq!(remove_rule(Some(&forum.forum_name), 0, &admin, &db_pool).await, Ok(()));

    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 1);
    assert_eq!(forum_rule_vec.get(0).unwrap().rule_id, common_rule_2.rule_id);

    Ok(())
}

#[tokio::test]
async fn test_get_forum_ban_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead = create_user("test", &db_pool).await;
    let banned_user_1 = create_user("1", &db_pool).await;
    let banned_user_2 = create_user("2", &db_pool).await;

    let (forum, post) = create_forum_with_post("forum", &lead, &db_pool).await;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let rule = add_rule(Some(&forum.forum_name), 0, "test", "test", &lead, &db_pool).await.expect("Rule should be added.");

    let ban_user_1 = ban_user_from_forum(
        banned_user_1.user_id,&
        forum.forum_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "", &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    let ban_user_2 = ban_user_from_forum(
        banned_user_2.user_id,&
        forum.forum_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(7),
        &db_pool
    ).await.expect("User 2 should be banned").expect("User 2 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "", &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 2);
    assert!(banned_user_vec.contains(&ban_user_1));
    assert!(banned_user_vec.contains(&ban_user_2));

    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "1", &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "x", &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_remove_user_ban() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead = create_user("test", &db_pool).await;
    let mut global_mod = create_user("global", &db_pool).await;
    global_mod.admin_role = AdminRole::Moderator;
    let banned_user_1 = create_user("1", &db_pool).await;

    let (forum, post) = create_forum_with_post("forum", &lead, &db_pool).await;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");
    
    let rule = add_rule(Some(&forum.forum_name), 0, "test", "test", &lead, &db_pool).await.expect("Rule should be added.");

    let ban_user_1 = ban_user_from_forum(
        banned_user_1.user_id,&
        forum.forum_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "", &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    assert_eq!(remove_user_ban(ban_user_1.ban_id, &banned_user_1, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_user_ban(ban_user_1.ban_id, &lead, &db_pool).await, Ok(ban_user_1));

    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "", &db_pool).await.expect("Should load forum bans");
    assert!(banned_user_vec.is_empty());

    let ban_user_1 = ban_user_from_forum(
        banned_user_1.user_id,&
        forum.forum_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "", &db_pool).await.expect("Should load forum bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    assert_eq!(remove_user_ban(ban_user_1.ban_id, &global_mod, &db_pool).await, Ok(ban_user_1));

    let banned_user_vec = get_forum_ban_vec(&forum.forum_name, "", &db_pool).await.expect("Should load forum bans");
    assert!(banned_user_vec.is_empty());

    // TODO add test to remove global ban when possible to create it

    Ok(())
}

#[tokio::test]
async fn test_moderate_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;

    let (forum, post) = create_forum_with_post("forum", &user, &db_pool).await;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded after forum creation");
    let rule = add_rule(Some(&forum.forum_name), 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");

    assert!(moderate_post(post.post_id, rule.rule_id, "unauthorized", &unauthorized_user, &db_pool).await.is_err());

    let moderated_post = moderate_post(post.post_id, rule.rule_id, "test", &user, &db_pool).await?;
    assert_eq!(moderated_post.moderator_id, Some(user.user_id));
    assert_eq!(moderated_post.moderator_name, Some(user.username));
    assert_eq!(moderated_post.moderator_message, Some(String::from("test")));
    assert_eq!(moderated_post.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(moderated_post.infringed_rule_title, Some(rule.title.clone()));

    let remoderated_post = moderate_post(post.post_id, rule.rule_id, "global", &global_moderator, &db_pool).await?;
    assert_eq!(remoderated_post.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_post.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_post.moderator_message, Some(String::from("global")));
    assert_eq!(moderated_post.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(moderated_post.infringed_rule_title, Some(rule.title));

    Ok(())
}

#[tokio::test]
async fn test_moderate_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;
    
    let (forum, _post, comment) = create_forum_with_post_and_comment("forum", &user, &db_pool).await;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded after forum creation");
    let rule = add_rule(Some(&forum.forum_name), 0, "test", "test", &user, &db_pool).await.expect("Rule should be added.");

    assert!(moderate_comment(comment.comment_id, rule.rule_id, "unauthorized", &unauthorized_user, &db_pool).await.is_err());

    let moderated_comment = moderate_comment(comment.comment_id, rule.rule_id, "test", &user, &db_pool).await?;
    assert_eq!(moderated_comment.moderator_id, Some(user.user_id));
    assert_eq!(moderated_comment.moderator_name, Some(user.username));
    assert_eq!(moderated_comment.moderator_message, Some(String::from("test")));
    assert_eq!(moderated_comment.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(moderated_comment.infringed_rule_title, Some(rule.title.clone()));

    let remoderated_comment = moderate_comment(comment.comment_id, rule.rule_id, "global", &global_moderator, &db_pool).await?;
    assert_eq!(remoderated_comment.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_comment.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_comment.moderator_message, Some(String::from("global")));
    assert_eq!(remoderated_comment.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(remoderated_comment.infringed_rule_title, Some(rule.title));

    Ok(())
}

#[tokio::test]
async fn test_ban_user_from_forum() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    // set user role in the DB, needed to test that global Moderators/Admin cannot be banned
    global_moderator.admin_role = AdminRole::Moderator;
    admin.admin_role = AdminRole::Admin;
    set_user_admin_role(global_moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await?;
    set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await?;
    let unauthorized_user = create_user("user", &db_pool).await;
    let banned_user = create_user("banned", &db_pool).await;

    let (forum, post) = create_forum_with_post("forum", &user, &db_pool).await;
    let rule = add_rule(None, 0, "test", "test", &admin, &db_pool).await.expect("Rule should be added.");
    let user = User::get(user.user_id, &db_pool).await.expect("Should be able to reload user.");

    // unauthorized used cannot ban
    assert!(ban_user_from_forum(banned_user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &unauthorized_user, None, &db_pool).await.is_err());
    // ban with 0 days has no effect
    assert_eq!(ban_user_from_forum(unauthorized_user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &user, Some(0), &db_pool).await?, None);
    let post = create_post(&forum.forum_name, "a", "b", None, false, false, false, None, &unauthorized_user, &db_pool).await?;

    // cannot ban moderators
    assert!(ban_user_from_forum(user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &global_moderator, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_forum(global_moderator.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_forum(admin.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_forum(user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &admin, Some(1), &db_pool).await.is_err());

    // forum moderator can ban ordinary users
    let user_ban = ban_user_from_forum(unauthorized_user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await?.expect("User ban from forum should be possible.");
    assert_eq!(user_ban.user_id, unauthorized_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, user.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(1))));

    // banned user cannot create new content
    let unauthorized_user = User::get(unauthorized_user.user_id, &db_pool).await.expect("Should be able to reload user.");
    assert!(create_post(&forum.forum_name, "c", "d", None, false, false, false, None, &unauthorized_user, &db_pool).await.is_err());
    assert!(create_comment(post.post_id, None, "a", None, false, &unauthorized_user, &db_pool).await.is_err());

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_forum(banned_user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &global_moderator, Some(2), &db_pool).await?.expect("User ban from forum should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, global_moderator.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(2))));

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_forum(banned_user.user_id, &forum.forum_name, post.post_id, None, rule.rule_id, &admin, None, &db_pool).await?.expect("User ban from forum should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, admin.user_id);
    assert_eq!(user_ban.until_timestamp, None);

    // banned user cannot create new content
    let banned_user = User::get(banned_user.user_id, &db_pool).await.expect("Should be possible to reload banned user.");
    assert!(create_post(&forum.forum_name, "c", "d", None, false, false, false, None, &banned_user, &db_pool).await.is_err());
    assert!(create_comment(post.post_id, None, "a", None, false, &banned_user, &db_pool).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_is_user_forum_moderator() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    // set user role in the DB, needed to test that global Moderators/Admin cannot be banned
    global_moderator.admin_role = AdminRole::Moderator;
    admin.admin_role = AdminRole::Admin;
    set_user_admin_role(global_moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await?;
    set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await?;
    let ordinary_user = create_user("user", &db_pool).await;

    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;

    assert_eq!(is_user_forum_moderator(user.user_id, &forum.forum_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_forum_moderator(global_moderator.user_id, &forum.forum_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_forum_moderator(admin.user_id, &forum.forum_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_forum_moderator(ordinary_user.user_id, &forum.forum_name, &db_pool).await, Ok(false));
    assert!(is_user_forum_moderator(ordinary_user.user_id + 1, &forum.forum_name, &db_pool).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_set_bannerl() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let banner_url = "a";
    assert_eq!(forum.banner_url, None);
    set_banner_url(&forum.forum_name, Some(banner_url), &user, &db_pool).await?;
    let forum = get_forum_by_name(&forum.forum_name, &db_pool).await?;
    assert_eq!(forum.banner_url, Some(String::from(banner_url)));
    Ok(())
}