use std::ops::Add;

use chrono::Days;
use leptos::ServerFnError;

use app::auth::User;
use app::comment::ssr::create_comment;
use app::forum::ssr::create_forum;
use app::forum_management::ssr::{ban_user_from_forum, moderate_comment, moderate_post};
use app::post::ssr::create_post;
use app::role::AdminRole;

use crate::common::*;

mod common;

#[tokio::test]
async fn test_moderate_post() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;
    let mut global_moderator = create_user("mod", "mod", "mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", "user", "user", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, db_pool.clone()).await?;
    let post = create_post(&forum.forum_name, "a", "body", None, false, None, &test_user, db_pool.clone()).await?;

    assert!(moderate_post(post.post_id, "unauthorized", &unauthorized_user, db_pool.clone()).await.is_err());

    let moderated_post = moderate_post(post.post_id, "test", &test_user, db_pool.clone()).await?;
    assert_eq!(moderated_post.moderator_id, Some(test_user.user_id));
    assert_eq!(moderated_post.moderator_name, Some(test_user.username));
    assert_eq!(moderated_post.moderated_body, Some(String::from("test")));

    let remoderated_post = moderate_post(post.post_id, "global", &global_moderator, db_pool.clone()).await?;
    assert_eq!(remoderated_post.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_post.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_post.moderated_body, Some(String::from("global")));

    Ok(())
}

#[tokio::test]
async fn test_moderate_comment() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;
    let mut global_moderator = create_user("mod", "mod", "mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", "user", "user", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, db_pool.clone()).await?;
    let post = create_post(&forum.forum_name, "a", "body", None, false, None, &test_user, db_pool.clone()).await?;
    let comment = create_comment(post.post_id, None, "comment", None, &test_user, db_pool.clone()).await?;

    assert!(moderate_comment(comment.comment_id, "unauthorized", &unauthorized_user, db_pool.clone()).await.is_err());

    let moderated_comment = moderate_comment(comment.comment_id, "test", &test_user, db_pool.clone()).await?;
    assert_eq!(moderated_comment.moderator_id, Some(test_user.user_id));
    assert_eq!(moderated_comment.moderator_name, Some(test_user.username));
    assert_eq!(moderated_comment.moderated_body, Some(String::from("test")));

    let remoderated_comment = moderate_comment(comment.comment_id, "global", &global_moderator, db_pool.clone()).await?;
    assert_eq!(remoderated_comment.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_comment.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_comment.moderated_body, Some(String::from("global")));

    Ok(())
}

#[tokio::test]
async fn test_ban_user_from_forum() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;
    let mut global_moderator = create_user("mod", "mod", "mod", &db_pool).await;
    global_moderator.admin_role = AdminRole::Moderator;
    let unauthorized_user = create_user("user", "user", "user", &db_pool).await;
    let banned_user = create_user("banned", "banned", "banned", &db_pool).await;

    let forum = create_forum("forum", "a", false, &test_user, db_pool.clone()).await?;
    let test_user = User::get(test_user.user_id, &db_pool).await.expect("Could not reload test user to update roles.");

    assert!(ban_user_from_forum(global_moderator.user_id, &forum.forum_name, &unauthorized_user, None, db_pool.clone()).await.is_err());
    assert_eq!(ban_user_from_forum(unauthorized_user.user_id, &forum.forum_name, &test_user, Some(0), db_pool.clone()).await?, None);
    let post = create_post(&forum.forum_name, "a", "b", None, false, None, &unauthorized_user, db_pool.clone()).await?;

    let user_ban = ban_user_from_forum(unauthorized_user.user_id, &forum.forum_name, &global_moderator, Some(1), db_pool.clone()).await?.expect("Expected Some(user_ban).");
    assert_eq!(user_ban.user_id, unauthorized_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, global_moderator.user_id);
    assert_eq!(user_ban.until_timestamp, Some(user_ban.create_timestamp.add(Days::new(1))));

    let unauthorized_user = User::get(unauthorized_user.user_id, &db_pool).await.expect("Could not reload unauthorized user to update roles.");
    assert!(create_post(&forum.forum_name, "c", "d", None, false, None, &unauthorized_user, db_pool.clone()).await.is_err());
    assert!(create_comment(post.post_id, None, "a", None, &unauthorized_user, db_pool.clone()).await.is_err());

    assert!(ban_user_from_forum(test_user.user_id, &forum.forum_name, &global_moderator, Some(1), db_pool.clone()).await.is_err());
    // TODO: need to set global moderator in DB for next line to pass
    // assert!(ban_user_from_forum(global_moderator.user_id, &forum.forum_name, &test_user, Some(1), db_pool.clone()).await.is_err());

    let user_ban = ban_user_from_forum(banned_user.user_id, &forum.forum_name, &test_user, None, db_pool.clone()).await?.expect("Expected Some(user_ban).");
    assert_eq!(user_ban.user_id, banned_user.user_id);
    assert_eq!(user_ban.forum_id, Some(forum.forum_id));
    assert_eq!(user_ban.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(user_ban.moderator_id, test_user.user_id);
    assert_eq!(user_ban.until_timestamp, None);

    let banned_user = User::get(banned_user.user_id, &db_pool).await.expect("Could not reload banned user to update roles.");
    assert!(create_post(&forum.forum_name, "c", "d", None, false, None, &banned_user, db_pool.clone()).await.is_err());
    assert!(create_comment(post.post_id, None, "a", None, &banned_user, db_pool.clone()).await.is_err());

    Ok(())
}