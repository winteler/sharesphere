use chrono::Days;
use std::ops::Add;
use object_store::memory::InMemory;
use object_store::ObjectStore;
use crate::common::*;
use crate::data_factory::{create_sphere_with_post, create_sphere_with_post_and_comment};
use crate::utils::*;
use sharesphere_core::comment::ssr::create_comment;
use sharesphere_core::moderation::ssr::moderate_comment;
use sharesphere_core::post::ssr::create_post;
use sharesphere_core::sphere::ssr::{create_sphere, get_sphere_by_name};
use sharesphere_core::sphere_management::ssr::{delete_sphere_image, get_sphere_ban_vec, remove_user_ban, set_sphere_banner_url, set_sphere_icon_url, store_sphere_image, SphereImageType, MAX_ICON_SIZE};
use sharesphere_core::sphere_management::ssr::{BANNER_FILE_INFER_ERROR_STR, INCORRECT_BANNER_FILE_TYPE_STR, MISSING_BANNER_FILE_STR, MISSING_SPHERE_STR};
use sharesphere_auth::role::ssr::{is_user_sphere_moderator, set_user_admin_role};
use sharesphere_auth::role::AdminRole;
use sharesphere_auth::user::User;
use sharesphere_core::moderation::ssr::{ban_user_from_sphere, moderate_post};
use sharesphere_core::post::PostTags;
use sharesphere_core::rule::BaseRule;
use sharesphere_core::rule::ssr::add_rule;
use sharesphere_utils::embed::Link;
use sharesphere_utils::errors::AppError;
use sharesphere_utils::widget::{IMAGE_FILE_PARAM, SPHERE_NAME_PARAM};

mod common;
mod data_factory;
mod utils;

#[tokio::test]
async fn test_get_sphere_ban_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut lead = create_user("test", &db_pool).await;
    let banned_user_1 = create_user("1", &db_pool).await;
    let banned_user_2 = create_user("2", &db_pool).await;

    let (sphere, post) = create_sphere_with_post("sphere", &mut lead, &db_pool).await;

    let rule = add_rule(Some(&sphere.sphere_name), 0, "test", "test", None, &lead, &db_pool).await.expect("Rule should be added.");

    let ban_user_1 = ban_user_from_sphere(
        banned_user_1.user_id,&
        sphere.sphere_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "", &db_pool).await.expect("Should load sphere bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    let ban_user_2 = ban_user_from_sphere(
        banned_user_2.user_id,&
        sphere.sphere_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(7),
        &db_pool
    ).await.expect("User 2 should be banned").expect("User 2 ban should be Some.");
    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "", &db_pool).await.expect("Should load sphere bans");
    assert_eq!(banned_user_vec.len(), 2);
    assert!(banned_user_vec.contains(&ban_user_1));
    assert!(banned_user_vec.contains(&ban_user_2));

    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "1", &db_pool).await.expect("Should load sphere bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "x", &db_pool).await.expect("Should load sphere bans");
    assert_eq!(banned_user_vec.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_remove_user_ban() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut lead = create_user("test", &db_pool).await;
    let mut global_mod = create_user("global", &db_pool).await;
    global_mod.admin_role = AdminRole::Moderator;
    let banned_user_1 = create_user("1", &db_pool).await;

    let (sphere, post) = create_sphere_with_post("sphere", &mut lead, &db_pool).await;
    
    let rule = add_rule(Some(&sphere.sphere_name), 0, "test", "test", None, &lead, &db_pool).await.expect("Rule should be added.");

    let ban_user_1 = ban_user_from_sphere(
        banned_user_1.user_id,&
        sphere.sphere_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "", &db_pool).await.expect("Should load sphere bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    assert_eq!(remove_user_ban(ban_user_1.ban_id, &banned_user_1, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_user_ban(ban_user_1.ban_id, &lead, &db_pool).await, Ok(ban_user_1));

    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "", &db_pool).await.expect("Should load sphere bans");
    assert!(banned_user_vec.is_empty());

    let ban_user_1 = ban_user_from_sphere(
        banned_user_1.user_id,&
        sphere.sphere_name,
        post.post_id,
        None,
        rule.rule_id,
        &lead,
        Some(1),
        &db_pool
    ).await.expect("User 1 should be banned").expect("User 1 ban should be Some.");
    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "", &db_pool).await.expect("Should load sphere bans");
    assert_eq!(banned_user_vec.len(), 1);
    assert!(banned_user_vec.contains(&ban_user_1));

    assert_eq!(remove_user_ban(ban_user_1.ban_id, &global_mod, &db_pool).await, Ok(ban_user_1.clone()));

    let banned_user_vec = get_sphere_ban_vec(&sphere.sphere_name, "", &db_pool).await.expect("Should load sphere bans");
    assert!(banned_user_vec.is_empty());

    let removed_ban = get_user_ban_by_id(ban_user_1.ban_id, &db_pool).await?;
    assert_eq!(removed_ban.ban_id, ban_user_1.ban_id);
    assert_eq!(removed_ban.user_id, ban_user_1.user_id);
    assert_eq!(removed_ban.username, ban_user_1.username);
    assert_eq!(removed_ban.sphere_id, ban_user_1.sphere_id);
    assert_eq!(removed_ban.sphere_name, ban_user_1.sphere_name);
    assert_eq!(removed_ban.post_id, ban_user_1.post_id);
    assert_eq!(removed_ban.comment_id, ban_user_1.comment_id);
    assert_eq!(removed_ban.infringed_rule_id, ban_user_1.infringed_rule_id);
    assert_eq!(removed_ban.moderator_id, ban_user_1.moderator_id);
    assert_eq!(removed_ban.until_timestamp, ban_user_1.until_timestamp);
    assert!(removed_ban.delete_timestamp.is_some_and(|delete_timestamp| delete_timestamp > removed_ban.create_timestamp));

    // TODO add test to remove global ban when possible to create it

    Ok(())
}

#[tokio::test]
async fn test_moderate_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;

    let (sphere, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;
    let rule = add_rule(Some(&sphere.sphere_name), 0, "test", "test", None, &user, &db_pool).await.expect("Rule should be added.");

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
    let mut user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;
    
    let (sphere, _post, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;
    let rule = add_rule(Some(&sphere.sphere_name), 0, "test", "test", None, &user, &db_pool).await.expect("Rule should be added.");

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
async fn test_ban_user_from_sphere() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    // set user role in the DB, needed to test that global Moderators/Admin cannot be banned
    global_moderator.admin_role = AdminRole::Moderator;
    admin.admin_role = AdminRole::Admin;
    set_user_admin_role(global_moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await?;
    set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await?;
    let unauthorized_user = create_user("user", &db_pool).await;
    let banned_user = create_user("banned", &db_pool).await;

    let (sphere, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;
    let rule = add_rule(None, 0, BaseRule::BeRespectful.into(), "test", None, &admin, &db_pool).await.expect("Rule should be added.");

    // unauthorized used cannot ban
    assert!(ban_user_from_sphere(banned_user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &unauthorized_user, None, &db_pool).await.is_err());
    // ban with 0 days has no effect
    assert_eq!(ban_user_from_sphere(unauthorized_user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &user, Some(0), &db_pool).await?, None);
    let post = create_post(
        &sphere.sphere_name, None,"a", "b", None, Link::default(),PostTags::default(), &unauthorized_user, &db_pool
    ).await?;

    // cannot ban moderators
    assert!(ban_user_from_sphere(user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &global_moderator, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_sphere(global_moderator.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_sphere(admin.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_sphere(user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &admin, Some(1), &db_pool).await.is_err());

    // sphere moderator can ban ordinary users
    let user_ban = ban_user_from_sphere(unauthorized_user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await?.expect("User ban from sphere should be possible.");
    assert_eq!(user_ban.user_id, unauthorized_user.user_id);
    assert_eq!(user_ban.sphere_id, Some(sphere.sphere_id));
    assert_eq!(user_ban.sphere_name, Some(sphere.sphere_name.clone()));
    assert_eq!(user_ban.moderator_id, user.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(1))));

    // banned user cannot create new content
    let unauthorized_user = User::get(unauthorized_user.user_id, &db_pool).await.expect("Should be able to reload user.");
    assert!(
        matches!(
            create_post(
                &sphere.sphere_name, None,"c", "d", None, Link::default(), PostTags::default(), &unauthorized_user, &db_pool
            ).await,
            Err(AppError::SphereBanUntil(_)),
        )
    );
    assert!(
        matches!(
            create_comment(post.post_id, None, "a", None, false, &unauthorized_user, &db_pool).await,
            Err(AppError::SphereBanUntil(_)),
        )
    );

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_sphere(banned_user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &global_moderator, Some(2), &db_pool).await?.expect("User ban from sphere should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.sphere_id, Some(sphere.sphere_id));
    assert_eq!(user_ban.sphere_name, Some(sphere.sphere_name.clone()));
    assert_eq!(user_ban.moderator_id, global_moderator.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(2))));

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_sphere(banned_user.user_id, &sphere.sphere_name, post.post_id, None, rule.rule_id, &admin, None, &db_pool).await?.expect("User ban from sphere should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.sphere_id, Some(sphere.sphere_id));
    assert_eq!(user_ban.sphere_name, Some(sphere.sphere_name.clone()));
    assert_eq!(user_ban.moderator_id, admin.user_id);
    assert_eq!(user_ban.until_timestamp, None);

    // banned user cannot create new content
    let banned_user = User::get(banned_user.user_id, &db_pool).await.expect("Should be possible to reload banned user.");
    assert_eq!(
        create_post(&sphere.sphere_name, None,"c", "d", None, Link::default(), PostTags::default(), &banned_user, &db_pool).await,
        Err(AppError::PermanentSphereBan),
    );
    assert_eq!(
        create_comment(post.post_id, None, "a", None, false, &banned_user, &db_pool).await,
        Err(AppError::PermanentSphereBan),
    );

    Ok(())
}

#[tokio::test]
async fn test_is_user_sphere_moderator() -> Result<(), AppError> {
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

    let sphere = create_sphere("sphere", "a", false, &user, &db_pool).await?;

    assert_eq!(is_user_sphere_moderator(user.user_id, &sphere.sphere_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_sphere_moderator(global_moderator.user_id, &sphere.sphere_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_sphere_moderator(admin.user_id, &sphere.sphere_name, &db_pool).await, Ok(true));
    assert_eq!(is_user_sphere_moderator(ordinary_user.user_id, &sphere.sphere_name, &db_pool).await, Ok(false));
    assert!(is_user_sphere_moderator(ordinary_user.user_id + 1, &sphere.sphere_name, &db_pool).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_delete_sphere_image() {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let base_user = create_user("base", &db_pool).await;
    let sphere = create_sphere("sphere", "a", false, &user, &db_pool).await.expect("Should create sphere");
    let object_store = InMemory::new();
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");

    let (sphere_name, image_file_name) = store_sphere_image(
        get_multipart_image_with_string(IMAGE_FILE_PARAM, SPHERE_NAME_PARAM, &sphere.sphere_name).await,
        MAX_ICON_SIZE,
        &object_store,
        &user,
    ).await.expect("Should store image");

    set_sphere_icon_url(
        &sphere_name.clone(),
        image_file_name.clone().map(|file_name| format!("https://test.com/{file_name}")).as_deref(),
        &user,
        &db_pool
    ).await.expect("Should set sphere icon url");

    let image_file_name = image_file_name.expect("Should have file name.");
    assert!(object_store.get(&object_store::path::Path::from(image_file_name.clone())).await.is_ok());

    delete_sphere_image(
        &sphere_name,
        SphereImageType::ICON,
        &object_store,
        &base_user,
        &db_pool
    ).await.expect_err("Base user should not have permission to store sphere image");

    delete_sphere_image(
        &sphere_name,
        SphereImageType::ICON,
        &object_store,
        &user,
        &db_pool
    ).await.expect("Should delete Sphere icon");

    assert!(object_store.get(&object_store::path::Path::from(image_file_name)).await.is_err());
}

#[tokio::test]
async fn test_store_sphere_image() {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let base_user = create_user("base", &db_pool).await;
    let sphere = create_sphere("sphere", "a", false, &user, &db_pool).await.expect("Should create sphere");
    let object_store = InMemory::new();
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");

    // Test need manage permissions to store image

    store_sphere_image(
        get_multipart_image_with_string(IMAGE_FILE_PARAM, SPHERE_NAME_PARAM, &sphere.sphere_name).await,
        MAX_ICON_SIZE,
        &object_store,
        &base_user,
    ).await.expect_err("Base user should not have permission to store sphere image");

    let (sphere_name, image_file_name) = store_sphere_image(
        get_multipart_image_with_string(IMAGE_FILE_PARAM, SPHERE_NAME_PARAM, &sphere.sphere_name).await,
        MAX_ICON_SIZE,
        &object_store,
        &user,
    ).await.expect("Should store image");
    assert_eq!(sphere_name, sphere.sphere_name);
    assert!(image_file_name.clone().is_some_and(|file_name| file_name.starts_with(&sphere_name) && file_name.ends_with(".webp")));
    assert!(object_store.get(&object_store::path::Path::from(image_file_name.unwrap())).await.is_ok());
    assert_eq!(
        store_sphere_image(
            get_multipart_image(IMAGE_FILE_PARAM).await,
            MAX_ICON_SIZE,
            &object_store,
            &user,
        ).await,
        Err(AppError::new(MISSING_SPHERE_STR))
    );
    assert_eq!(
        store_sphere_image(
            get_multipart_string(SPHERE_NAME_PARAM, &sphere.sphere_name).await,
            MAX_ICON_SIZE,
            &object_store,
            &user,
        ).await,
        Err(AppError::new(MISSING_BANNER_FILE_STR))
    );
    assert_eq!(
        store_sphere_image(
            get_multipart_pdf_with_string(IMAGE_FILE_PARAM, SPHERE_NAME_PARAM, &sphere.sphere_name).await,
            MAX_ICON_SIZE,
            &object_store,
            &user,
        ).await,
        Err(AppError::new(INCORRECT_BANNER_FILE_TYPE_STR))
    );
    assert_eq!(
        store_sphere_image(
            get_invalid_multipart_image_with_string(IMAGE_FILE_PARAM, SPHERE_NAME_PARAM, &sphere.sphere_name).await,
            MAX_ICON_SIZE,
            &object_store,
            &user,
        ).await,
        Err(AppError::new(BANNER_FILE_INFER_ERROR_STR))
    );
}

#[tokio::test]
async fn test_set_sphere_icon_url() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let base_user = create_user("base", &db_pool).await;
    let sphere = create_sphere("sphere", "a", false, &user, &db_pool).await?;
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let icon_url = "a";
    assert_eq!(sphere.icon_url, None);

    set_sphere_icon_url(&sphere.sphere_name, Some(icon_url), &user, &db_pool).await?;
    let sphere = get_sphere_by_name(&sphere.sphere_name, &db_pool).await?;
    assert_eq!(sphere.icon_url, Some(String::from(icon_url)));

    set_sphere_icon_url(&sphere.sphere_name, Some(icon_url), &base_user, &db_pool).await.expect_err("Base user should not have permission to set sphere icon url");

    Ok(())
}

#[tokio::test]
async fn test_set_sphere_banner_url() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;
    let base_user = create_user("base", &db_pool).await;
    let sphere = create_sphere("sphere", "a", false, &user, &db_pool).await?;
    // reload user to have updated permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let banner_url = "a";
    assert_eq!(sphere.banner_url, None);

    set_sphere_banner_url(&sphere.sphere_name, Some(banner_url), &user, &db_pool).await?;
    let sphere = get_sphere_by_name(&sphere.sphere_name, &db_pool).await?;
    assert_eq!(sphere.banner_url, Some(String::from(banner_url)));

    set_sphere_banner_url(&sphere.sphere_name, Some(banner_url), &base_user, &db_pool).await.expect_err("Base user should not have permission to set sphere banner url");

    Ok(())
}