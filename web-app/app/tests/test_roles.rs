use leptos::ServerFnError;

use app::auth::User;
use app::forum;
use app::role::PermissionLevel;
use app::role::ssr::set_user_forum_role;

use crate::common::{create_test_user, create_user, get_db_pool};

mod common;
#[tokio::test]
async fn test_set_user_forum_role() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let creator_user = create_user("user", "user", "user@user.com", &db_pool).await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let forum = forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &creator_user,
        db_pool.clone(),
    ).await?;
    let creator_user = User::get(creator_user.user_id, &db_pool).await.expect("Could not reload user.");

    set_user_forum_role(forum.forum_id, forum_name, test_user.user_id, PermissionLevel::Moderate, &creator_user, &db_pool).await.expect("Could not assign role.");
    let test_user = User::get(test_user.user_id, &db_pool).await.expect("Could not reload user.");
    assert_eq!(*test_user.permission_by_forum_map.get(forum_name).expect("Has permission for forum."), PermissionLevel::Moderate);
    set_user_forum_role(forum.forum_id, forum_name, test_user.user_id, PermissionLevel::Ban, &creator_user, &db_pool).await.expect("Could not update role.");
    let test_user = User::get(test_user.user_id, &db_pool).await.expect("Could not reload user after role update.");
    assert_eq!(*test_user.permission_by_forum_map.get(forum_name).expect("Has permission for forum."), PermissionLevel::Ban);

    Ok(())
}