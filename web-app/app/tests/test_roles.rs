use leptos::ServerFnError;

use app::auth::User;
use app::errors::AppError;
use app::forum;
use app::role::PermissionLevel;
use app::role::ssr::set_user_forum_role;

use crate::common::{create_test_user, create_user, get_db_pool};

mod common;
#[tokio::test]
async fn test_set_user_forum_role() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let lead_user = create_user("user", "user", "user@user.com", &db_pool).await;
    let ordinary_user = create_user("a", "a", "a", &db_pool).await;
    let moderator = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let forum = forum::ssr::create_forum(forum_name, "forum", false, &lead_user, db_pool.clone()).await?;
    let lead_user = User::get(lead_user.user_id, &db_pool)
        .await
        .expect("Could not reload user.");

    // test elect moderator
    let (moderate_role, prev_leader_id) = set_user_forum_role(
        forum.forum_id,
        forum_name,
        moderator.user_id,
        PermissionLevel::Moderate,
        &lead_user,
        &db_pool,
    )
        .await
        .expect("Could not assign role.");
    assert_eq!(moderate_role.user_id, moderator.user_id);
    assert_eq!(moderate_role.forum_id, forum.forum_id);
    assert_eq!(moderate_role.forum_name, forum.forum_name);
    assert_eq!(moderate_role.grantor_id, lead_user.user_id);
    assert_eq!(moderate_role.permission_level, PermissionLevel::Moderate);
    assert_eq!(prev_leader_id, None);
    let moderator = User::get(moderator.user_id, &db_pool)
        .await
        .expect("Could not reload user.");
    assert_eq!(
        moderator.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Moderate)
    );

    // test need elect permissions to add moderators
    assert_eq!(
        set_user_forum_role(
            forum.forum_id,
            forum_name,
            ordinary_user.user_id,
            PermissionLevel::Moderate,
            &moderator,
            &db_pool
        ).await,
        Err(AppError::InsufficientPrivileges)
    );

    // test change permission level to Elect
    let (moderate_role, prev_leader_id) = set_user_forum_role(
        forum.forum_id,
        forum_name,
        moderator.user_id,
        PermissionLevel::Elect,
        &lead_user,
        &db_pool,
    )
        .await
        .expect("Could not update role.");
    assert_eq!(moderate_role.user_id, moderator.user_id);
    assert_eq!(moderate_role.forum_id, forum.forum_id);
    assert_eq!(moderate_role.forum_name, forum.forum_name);
    assert_eq!(moderate_role.grantor_id, lead_user.user_id);
    assert_eq!(moderate_role.permission_level, PermissionLevel::Elect);
    assert_eq!(prev_leader_id, None);
    let moderator = User::get(moderator.user_id, &db_pool)
        .await
        .expect("Could not reload user after role update.");
    assert_eq!(
        moderator.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Elect)
    );

    // test can now elect other moderators
    let (moderate_role, prev_leader_id) = set_user_forum_role(
        forum.forum_id,
        forum_name,
        ordinary_user.user_id,
        PermissionLevel::Moderate,
        &moderator,
        &db_pool,
    )
        .await?;
    assert_eq!(moderate_role.user_id, ordinary_user.user_id);
    assert_eq!(moderate_role.forum_id, forum.forum_id);
    assert_eq!(moderate_role.forum_name, forum.forum_name);
    assert_eq!(moderate_role.grantor_id, moderator.user_id);
    assert_eq!(moderate_role.permission_level, PermissionLevel::Moderate);
    assert_eq!(prev_leader_id, None);
    let ordinary_user = User::get(ordinary_user.user_id, &db_pool)
        .await
        .expect("Could not reload user after role update.");
    assert_eq!(
        ordinary_user.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Moderate)
    );

    // test moderator cannot set leader or downgrade higher up moderator
    assert!(
        set_user_forum_role(
            forum.forum_id,
            forum_name,
            ordinary_user.user_id,
            PermissionLevel::Lead,
            &moderator,
            &db_pool
        ).await.is_err()
    );
    assert_eq!(
        set_user_forum_role(
            forum.forum_id,
            forum_name,
            lead_user.user_id,
            PermissionLevel::Moderate,
            &moderator,
            &db_pool
        ).await,
        Err(AppError::InsufficientPrivileges)
    );

    // test leader can choose another leader
    let (new_lead_role, prev_leader_id) = set_user_forum_role(
        forum.forum_id,
        forum_name,
        ordinary_user.user_id,
        PermissionLevel::Lead,
        &lead_user,
        &db_pool,
    )
        .await
        .expect("Could not change leader.");

    assert_eq!(new_lead_role.user_id, ordinary_user.user_id);
    assert_eq!(new_lead_role.forum_id, forum.forum_id);
    assert_eq!(new_lead_role.forum_name, forum.forum_name);
    assert_eq!(new_lead_role.grantor_id, moderator.user_id);
    assert_eq!(new_lead_role.permission_level, PermissionLevel::Lead);
    assert_eq!(prev_leader_id, Some(lead_user.user_id));
    let ordinary_user = User::get(ordinary_user.user_id, &db_pool)
        .await
        .expect("Could not reload user after lead update.");
    assert_eq!(
        ordinary_user.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Lead)
    );
    let prev_lead_user = User::get(lead_user.user_id, &db_pool)
        .await
        .expect("Could not reload user after lead update.");
    assert_eq!(
        prev_lead_user.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Elect)
    );

    Ok(())
}
