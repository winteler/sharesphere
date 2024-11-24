use chrono::Days;
use std::ops::Add;

use crate::common::*;
use crate::data_factory::{create_forum_with_post, create_forum_with_post_and_comment};
use crate::utils::*;
use app::comment::ssr::create_comment;
use app::errors::AppError;
use app::forum::ssr::{create_forum, get_forum_by_name};
use app::forum_management::ssr::{get_forum_ban_vec, is_user_forum_moderator, remove_user_ban, set_forum_banner_url, set_forum_icon_url, store_forum_image};
use app::forum_management::{BANNER_FILE_INFER_ERROR_STR, INCORRECT_BANNER_FILE_TYPE_STR, MISSING_BANNER_FILE_STR, MISSING_FORUM_STR};
use app::moderation::ssr::{ban_user_from_forum, moderate_comment, moderate_post};
use app::post::ssr::create_post;
use app::role::ssr::set_user_admin_role;
use app::role::AdminRole;
use app::rule::ssr::add_rule;
use app::user::User;
use app::widget::{FORUM_NAME_PARAM, IMAGE_FILE_PARAM};

mod common;
mod data_factory;
mod utils;

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
async fn test_store_forum_image() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;
    let store_path = "/tmp/";
    let image_category = "test_";
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    
    let (forum_name, image_file_name) = store_forum_image(
        store_path,
        image_category,
        get_multipart_image_with_string(IMAGE_FILE_PARAM, FORUM_NAME_PARAM, &forum.forum_name).await,
        &user,
    ).await?;
    assert_eq!(forum_name, forum.forum_name);
    assert_eq!(image_file_name, Some(format!("{forum_name}.png")));
    assert_eq!(
        store_forum_image(
            store_path,
            image_category,
            get_multipart_image(IMAGE_FILE_PARAM).await,
            &user,
        ).await,
        Err(AppError::new(MISSING_FORUM_STR))
    );
    assert_eq!(
        store_forum_image(
            store_path,
            image_category,
            get_multipart_string(FORUM_NAME_PARAM, &forum.forum_name).await,
            &user,
        ).await,
        Err(AppError::new(MISSING_BANNER_FILE_STR))
    );
    assert_eq!(
        store_forum_image(
            store_path,
            image_category,
            get_multipart_pdf_with_string(IMAGE_FILE_PARAM, FORUM_NAME_PARAM, &forum.forum_name).await,
            &user,
        ).await,
        Err(AppError::new(INCORRECT_BANNER_FILE_TYPE_STR))
    );
    assert_eq!(
        store_forum_image(
            store_path,
            image_category,
            get_invalid_multipart_image_with_string(IMAGE_FILE_PARAM, FORUM_NAME_PARAM, &forum.forum_name).await,
            &user,
        ).await,
        Err(AppError::new(BANNER_FILE_INFER_ERROR_STR))
    );
    Ok(())
}

#[tokio::test]
async fn test_set_forum_icon_url() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let icon_url = "a";
    assert_eq!(forum.icon_url, None);

    set_forum_icon_url(&forum.forum_name, Some(icon_url), &user, &db_pool).await?;
    let forum = get_forum_by_name(&forum.forum_name, &db_pool).await?;
    assert_eq!(forum.icon_url, Some(String::from(icon_url)));
    Ok(())
}

#[tokio::test]
async fn test_set_forum_banner_url() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let banner_url = "a";
    assert_eq!(forum.banner_url, None);

    set_forum_banner_url(&forum.forum_name, Some(banner_url), &user, &db_pool).await?;
    let forum = get_forum_by_name(&forum.forum_name, &db_pool).await?;
    assert_eq!(forum.banner_url, Some(String::from(banner_url)));
    Ok(())
}