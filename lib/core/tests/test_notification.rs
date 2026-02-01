use sharesphere_core::notification::NotificationType;
use sharesphere_core::notification::ssr::{create_notification, get_notifications, read_all_notifications, read_notification};

use crate::common::*;
use crate::data_factory::*;
use crate::utils::get_notification;

mod common;
mod data_factory;
mod utils;

#[tokio::test]
async fn test_create_notification() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;
    let trigger_user = create_user("trigger", &db_pool).await;

    let (_, post, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let post_comment_notif = create_notification(
        post.post_id,
        None,
        trigger_user.user_id,
        NotificationType::Comment,
        &db_pool
    ).await.expect("Should create post comment notification");

    assert_eq!(post_comment_notif.post_id, post.post_id);
    assert_eq!(post_comment_notif.comment_id, None);
    assert_eq!(post_comment_notif.user_id, user.user_id);
    assert_eq!(post_comment_notif.trigger_user_id, trigger_user.user_id);
    assert_eq!(post_comment_notif.trigger_username, trigger_user.username);
    assert_eq!(post_comment_notif.notification_type, NotificationType::Comment);
    assert_eq!(post_comment_notif.is_read, false);

    let comment_comment_notif = create_notification(
        comment.post_id,
        Some(comment.comment_id),
        trigger_user.user_id,
        NotificationType::Moderation,
        &db_pool
    ).await.expect("Should create post comment notification");

    assert_eq!(comment_comment_notif.post_id, comment.post_id);
    assert_eq!(comment_comment_notif.comment_id, Some(comment.comment_id));
    assert_eq!(comment_comment_notif.user_id, user.user_id);
    assert_eq!(comment_comment_notif.trigger_user_id, trigger_user.user_id);
    assert_eq!(comment_comment_notif.trigger_username, trigger_user.username);
    assert_eq!(comment_comment_notif.notification_type, NotificationType::Moderation);
    assert_eq!(comment_comment_notif.is_read, false);
}

#[tokio::test]
async fn test_get_notifications() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;
    let trigger_user = create_user("trigger", &db_pool).await;

    let (_, post, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let post_comment_notif = create_notification(
        post.post_id,
        None,
        trigger_user.user_id,
        NotificationType::Moderation,
        &db_pool
    ).await.expect("Should create post comment notification");

    let comment_comment_notif = create_notification(
        comment.post_id,
        Some(comment.comment_id),
        trigger_user.user_id,
        NotificationType::Comment,
        &db_pool
    ).await.expect("Should create post comment notification");

    let expected_notif_vec = vec![comment_comment_notif, post_comment_notif];

    let notif_vec = get_notifications(user.user_id, &db_pool).await.expect("Should get notification vec");
    assert_eq!(notif_vec, expected_notif_vec);
}

#[tokio::test]
async fn test_read_notification() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;
    let trigger_user = create_user("trigger", &db_pool).await;

    let (_, post, _) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let mut notification = create_notification(
        post.post_id,
        None,
        trigger_user.user_id,
        NotificationType::Comment,
        &db_pool
    ).await.expect("Should create post comment notification");

    read_notification(notification.notification_id, user.user_id, &db_pool).await.expect("Should read notification");

    notification.is_read = true;

    let read_notif = get_notification(notification.notification_id, &db_pool).await.expect("Should get notification");
    assert_eq!(read_notif, notification);
}

#[tokio::test]
async fn test_read_all_notifications() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;
    let trigger_user = create_user("trigger", &db_pool).await;

    let (_, post, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let mut post_comment_notif = create_notification(
        post.post_id,
        None,
        trigger_user.user_id,
        NotificationType::Moderation,
        &db_pool
    ).await.expect("Should create post comment notification");

    let mut comment_comment_notif = create_notification(
        comment.post_id,
        Some(comment.comment_id),
        trigger_user.user_id,
        NotificationType::Comment,
        &db_pool
    ).await.expect("Should create post comment notification");

    read_all_notifications(user.user_id, &db_pool).await.expect("Should read all notification");

    post_comment_notif.is_read = true;
    comment_comment_notif.is_read = true;

    let expected_notif_vec = vec![comment_comment_notif, post_comment_notif];

    let notif_vec = get_notifications(user.user_id, &db_pool).await.expect("Should get notification vec");
    assert_eq!(notif_vec, expected_notif_vec);
}