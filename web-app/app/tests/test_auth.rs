use leptos::ServerFnError;

use app::auth::User;
use app::forum::ssr::create_forum;
use app::forum_management::ssr::ban_user_from_forum;
use app::role::PermissionLevel;
use app::role::ssr::set_user_forum_role;

pub use crate::common::*;

mod common;
mod data_factory;

#[tokio::test]
async fn test_user_get() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let creator_user = create_user("user", "user", "user@user.com", &db_pool).await;
    let test_user = create_test_user(&db_pool).await;

    let forum_a = create_forum("a", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_b = create_forum("b", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_c = create_forum("c", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_d = create_forum("d", "test", false, &creator_user, db_pool.clone()).await?;
    let forum_e = create_forum("e", "test", false, &creator_user, db_pool.clone()).await?;

    // reload creator_user so that it has the updated roles after creating forums.
    let creator_user = User::get(creator_user.user_id, &db_pool).await.expect("Could not get creator user.");

    set_user_forum_role(forum_a.forum_id, &forum_a.forum_name, test_user.user_id, PermissionLevel::Moderate, &creator_user, db_pool.clone()).await?;
    set_user_forum_role(forum_b.forum_id, &forum_b.forum_name, test_user.user_id, PermissionLevel::Elect, &creator_user, db_pool.clone()).await?;

    ban_user_from_forum(test_user.user_id, &forum_c.forum_name, &creator_user, Some(0), db_pool.clone()).await?;
    ban_user_from_forum(test_user.user_id, &forum_d.forum_name, &creator_user, Some(1), db_pool.clone()).await?;
    ban_user_from_forum(test_user.user_id, &forum_e.forum_name, &creator_user, None, db_pool.clone()).await?;

    let result_user = User::get(test_user.user_id, &db_pool).await.expect("Could not get result user.");

    assert_eq!(result_user.can_moderate_forum(&forum_a.forum_name), true);
    assert_eq!(result_user.can_moderate_forum(&forum_b.forum_name), true);
    assert_eq!(result_user.can_moderate_forum(&forum_c.forum_name), false);
    assert_eq!(result_user.can_moderate_forum(&forum_d.forum_name), false);
    assert_eq!(result_user.can_moderate_forum(&forum_e.forum_name), false);

    assert_eq!(result_user.is_banned_from_forum(&forum_a.forum_name), false);
    assert_eq!(result_user.is_banned_from_forum(&forum_b.forum_name), false);
    assert_eq!(result_user.is_banned_from_forum(&forum_c.forum_name), false);
    assert_eq!(result_user.is_banned_from_forum(&forum_d.forum_name), true);
    assert_eq!(result_user.is_banned_from_forum(&forum_e.forum_name), true);

    // TODO test global ban when ssr function is implemented

    Ok(())
}