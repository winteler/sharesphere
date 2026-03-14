use chrono::Days;
use std::ops::Add;

use sharesphere_core::comment::ssr::create_comment;
use sharesphere_core::moderation::ssr::{get_sphere_contents, moderate_comment};
use sharesphere_core::post::ssr::create_post;
use sharesphere_auth::role::ssr::{set_user_admin_role};
use sharesphere_auth::role::AdminRole;
use sharesphere_auth::user::User;
use sharesphere_core::moderation::Content;
use sharesphere_core::moderation::ssr::{ban_user_from_sphere, moderate_post};
use sharesphere_core::post::PostTags;
use sharesphere_core::rule::{BaseRule};
use sharesphere_core::rule::ssr::add_rule;
use sharesphere_utils::embed::Link;
use sharesphere_utils::errors::AppError;

use crate::common::*;
use crate::data_factory::{add_base_rule, create_sphere_with_post, create_sphere_with_post_and_comment, get_moderated_and_deleted_comments, get_moderated_and_deleted_posts};

mod common;
mod data_factory;
mod utils;

#[tokio::test]
async fn test_get_sphere_contents() {
    let db_pool = get_db_pool().await;
    let mut user = create_user("test", &db_pool).await;
    let (sphere, mut post_1, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;
    let post_2 = create_post(
        &sphere.sphere_name,
        None,
        "post_2",
        "post_2_body",
        None,
        Link::default(),
        PostTags::default(),
        &user,
        &db_pool
    ).await.expect("Should create post");
    let _ = get_moderated_and_deleted_posts(&sphere.sphere_name, &user, &db_pool).await;
    let _ = get_moderated_and_deleted_comments(&post_1, &sphere.sphere_name, &user, &db_pool).await;

    post_1.num_comments = 3;
    let expected_content_vec = vec![
        Content::Post(post_2),
        Content::Comment(comment),
        Content::Post(post_1),
    ];

    assert_eq!(
        get_sphere_contents(&sphere.sphere_name, 3, 0, &db_pool).await.expect("Should get full content vec"),
        expected_content_vec,
    );
    assert_eq!(
        get_sphere_contents(&sphere.sphere_name, 2, 0, &db_pool).await.expect("Should get first two contents"),
        expected_content_vec[0..2],
    );
    assert_eq!(
        get_sphere_contents(&sphere.sphere_name, 2, 1, &db_pool).await.expect("Should get last two contents"),
        expected_content_vec[1..],
    );
}

#[tokio::test]
async fn test_moderate_post() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_user("test", &db_pool).await;
    let mut global_moderator = create_user("mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", &db_pool).await;

    let (sphere, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;
    let rule = add_rule(&sphere.sphere_name, 0, "test", "test", None, &user, &db_pool).await.expect("Rule should be added.");

    assert!(moderate_post(post.post_id, rule.rule_id, "unauthorized", &unauthorized_user, &db_pool).await.is_err());

    let moderated_post = moderate_post(post.post_id, rule.rule_id, "test", &user, &db_pool).await?;
    assert_eq!(moderated_post.moderation_info.moderator_id, Some(user.user_id));
    assert_eq!(moderated_post.moderation_info.moderator_name, Some(user.username));
    assert_eq!(moderated_post.moderation_info.moderator_message, Some(String::from("test")));
    assert_eq!(moderated_post.moderation_info.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(moderated_post.moderation_info.infringed_rule_title, Some(rule.title.clone()));

    let remoderated_post = moderate_post(post.post_id, rule.rule_id, "global", &global_moderator, &db_pool).await?;
    assert_eq!(remoderated_post.moderation_info.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_post.moderation_info.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_post.moderation_info.moderator_message, Some(String::from("global")));
    assert_eq!(moderated_post.moderation_info.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(moderated_post.moderation_info.infringed_rule_title, Some(rule.title));

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
    let rule = add_rule(&sphere.sphere_name, 0, "test", "test", None, &user, &db_pool).await.expect("Rule should be added.");

    assert!(moderate_comment(comment.comment_id, rule.rule_id, "unauthorized", &unauthorized_user, &db_pool).await.is_err());

    let moderated_comment = moderate_comment(comment.comment_id, rule.rule_id, "test", &user, &db_pool).await?;
    assert_eq!(moderated_comment.moderation_info.moderator_id, Some(user.user_id));
    assert_eq!(moderated_comment.moderation_info.moderator_name, Some(user.username));
    assert_eq!(moderated_comment.moderation_info.moderator_message, Some(String::from("test")));
    assert_eq!(moderated_comment.moderation_info.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(moderated_comment.moderation_info.infringed_rule_title, Some(rule.title.clone()));

    let remoderated_comment = moderate_comment(comment.comment_id, rule.rule_id, "global", &global_moderator, &db_pool).await?;
    assert_eq!(remoderated_comment.moderation_info.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_comment.moderation_info.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_comment.moderation_info.moderator_message, Some(String::from("global")));
    assert_eq!(remoderated_comment.moderation_info.infringed_rule_id, Some(rule.rule_id));
    assert_eq!(remoderated_comment.moderation_info.infringed_rule_title, Some(rule.title));

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
    let rule = add_base_rule(0, BaseRule::BeRespectful.into(), "test", None, &admin, &db_pool).await.expect("Rule should be added.");

    // unauthorized used cannot ban
    assert!(ban_user_from_sphere(banned_user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &unauthorized_user, None, &db_pool).await.is_err());
    // ban with 0 days has no effect
    assert_eq!(ban_user_from_sphere(unauthorized_user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &user, Some(0), &db_pool).await?, None);
    let post = create_post(
        &sphere.sphere_name, None,"a", "b", None, Link::default(),PostTags::default(), &unauthorized_user, &db_pool
    ).await?;

    // cannot ban moderators
    assert!(ban_user_from_sphere(user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &global_moderator, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_sphere(global_moderator.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_sphere(admin.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool).await.is_err());
    assert!(ban_user_from_sphere(user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &admin, Some(1), &db_pool).await.is_err());

    // sphere moderator can ban ordinary users
    let user_ban = ban_user_from_sphere(
        unauthorized_user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &user, Some(1), &db_pool
    ).await?.expect("User ban from sphere should be possible.");
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
    let user_ban = ban_user_from_sphere(banned_user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &global_moderator, Some(2), &db_pool).await?.expect("User ban from sphere should be possible.");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.sphere_id, Some(sphere.sphere_id));
    assert_eq!(user_ban.sphere_name, Some(sphere.sphere_name.clone()));
    assert_eq!(user_ban.moderator_id, global_moderator.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(2))));

    // global moderator can ban ordinary users
    let user_ban = ban_user_from_sphere(banned_user.user_id, sphere.sphere_id, post.post_id, None, rule.rule_id, &admin, None, &db_pool).await?.expect("User ban from sphere should be possible.");
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