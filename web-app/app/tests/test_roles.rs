use leptos::ServerFnError;

use app::auth::User;
use app::errors::AppError;
use app::forum;
use app::role::{AdminRole, PermissionLevel};
use app::role::ssr::{set_user_admin_role, set_user_forum_role};

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

#[tokio::test]
async fn test_set_user_admin_role() -> Result<(), ServerFnError> {

    let db_pool = get_db_pool().await;
    let ordinary_user = create_test_user(&db_pool).await;
    let moderator = create_user("a", "a", "a", &db_pool).await;
    let mut admin = create_user("b", "b", "b", &db_pool).await;

    // ordinary user cannot set admin role
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Admin, &ordinary_user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Moderator, &ordinary_user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::None, &ordinary_user, &db_pool).await, Err(AppError::InsufficientPrivileges));

    // admin can set admin roles
    admin.admin_role = AdminRole::Admin;
    let sql_admin = set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await.expect("Could not set admin role.");
    assert_eq!(sql_admin.user_id, admin.user_id);
    assert_eq!(sql_admin.admin_role, AdminRole::Admin);
    let admin = User::get(admin.user_id, &db_pool).await.expect("Cannot reload admin user");
    assert_eq!(admin.admin_role, AdminRole::Admin);

    let sql_moderator = set_user_admin_role(moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await.expect("Could not set admin role.");
    assert_eq!(sql_moderator.user_id, moderator.user_id);
    assert_eq!(sql_moderator.admin_role, AdminRole::Moderator);
    let moderator = User::get(moderator.user_id, &db_pool).await.expect("Cannot reload moderator user");
    assert_eq!(moderator.admin_role, AdminRole::Moderator);

    // moderator cannot set admin roles
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Admin, &moderator, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Moderator, &moderator, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::None, &moderator, &db_pool).await, Err(AppError::InsufficientPrivileges));

    Ok(())
}
