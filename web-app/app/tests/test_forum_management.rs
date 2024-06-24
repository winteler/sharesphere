use leptos::ServerFnError;

use app::forum::ssr::create_forum;
use app::forum_management::ssr::moderate_post;
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

    println!("test user");

    let moderated_post = moderate_post(post.post_id, "test", &test_user, db_pool.clone()).await?;
    assert_eq!(moderated_post.moderator_id, Some(test_user.user_id));
    assert_eq!(moderated_post.moderator_name, Some(test_user.username));
    assert_eq!(moderated_post.moderated_body, Some(String::from("test")));

    println!("test global mod");

    let remoderated_post = moderate_post(post.post_id, "global", &global_moderator, db_pool.clone()).await?;
    assert_eq!(remoderated_post.moderator_id, Some(global_moderator.user_id));
    assert_eq!(remoderated_post.moderator_name, Some(global_moderator.username));
    assert_eq!(remoderated_post.moderated_body, Some(String::from("global")));

    Ok(())
}