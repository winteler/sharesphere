use app::errors::AppError;
use app::forum;
use app::role::{AdminRole, PermissionLevel};
use app::role::ssr::{get_forum_role_vec, get_user_forum_role, set_user_admin_role, set_user_forum_role};
use app::user::User;

use crate::common::{create_user, get_db_pool};

mod common;

#[tokio::test]
async fn test_get_user_forum_role() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user_a = create_user("a", &db_pool).await;
    let user_b = create_user("b", &db_pool).await;
    let user_c = create_user("c", &db_pool).await;

    let forum_1 = forum::ssr::create_forum("1", "forum", false, &user_a, &db_pool).await?;
    let forum_2 = forum::ssr::create_forum("2", "forum", false, &user_a, &db_pool).await?;
    let forum_3 = forum::ssr::create_forum("3", "forum", false, &user_b, &db_pool).await?;
    let user_a = User::get(user_a.user_id, &db_pool).await.expect("Should be able to reload user.");
    let user_b = User::get(user_b.user_id, &db_pool).await.expect("Should be able to reload user.");

    set_user_forum_role(user_b.user_id, &forum_1.forum_name, PermissionLevel::Manage, &user_a, &db_pool).await.expect("User should be able to grant Manage permissions.");
    set_user_forum_role(user_c.user_id, &forum_1.forum_name, PermissionLevel::Moderate, &user_a, &db_pool).await.expect("User should be able to grant Moderate permissions.");
    set_user_forum_role(user_b.user_id, &forum_2.forum_name, PermissionLevel::Ban, &user_a, &db_pool).await.expect("User should be able to grant Ban permissions.");
    set_user_forum_role(user_c.user_id, &forum_2.forum_name, PermissionLevel::Moderate, &user_a, &db_pool).await.expect("User should be able to grant Moderate permissions.");
    set_user_forum_role(user_a.user_id, &forum_3.forum_name, PermissionLevel::None, &user_b, &db_pool).await.expect("User should be able to grant Moderate permissions.");

    let user_a_forum_1_role = get_user_forum_role(user_a.user_id, &forum_1.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_a_forum_1_role.user_id, user_a.user_id);
    assert_eq!(user_a_forum_1_role.forum_id, forum_1.forum_id);
    assert_eq!(user_a_forum_1_role.forum_name, forum_1.forum_name);
    assert_eq!(user_a_forum_1_role.grantor_id, user_a.user_id);
    assert_eq!(user_a_forum_1_role.permission_level, PermissionLevel::Lead);

    let user_b_forum_1_role = get_user_forum_role(user_b.user_id, &forum_1.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_b_forum_1_role.user_id, user_b.user_id);
    assert_eq!(user_b_forum_1_role.forum_id, forum_1.forum_id);
    assert_eq!(user_b_forum_1_role.forum_name, forum_1.forum_name);
    assert_eq!(user_b_forum_1_role.grantor_id, user_a.user_id);
    assert_eq!(user_b_forum_1_role.permission_level, PermissionLevel::Manage);

    let user_c_forum_1_role = get_user_forum_role(user_c.user_id, &forum_1.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_c_forum_1_role.user_id, user_c.user_id);
    assert_eq!(user_c_forum_1_role.forum_id, forum_1.forum_id);
    assert_eq!(user_c_forum_1_role.forum_name, forum_1.forum_name);
    assert_eq!(user_c_forum_1_role.grantor_id, user_a.user_id);
    assert_eq!(user_c_forum_1_role.permission_level, PermissionLevel::Moderate);

    let user_a_forum_2_role = get_user_forum_role(user_a.user_id, &forum_2.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_a_forum_2_role.user_id, user_a.user_id);
    assert_eq!(user_a_forum_2_role.forum_id, forum_2.forum_id);
    assert_eq!(user_a_forum_2_role.forum_name, forum_2.forum_name);
    assert_eq!(user_a_forum_2_role.grantor_id, user_a.user_id);
    assert_eq!(user_a_forum_2_role.permission_level, PermissionLevel::Lead);

    let user_b_forum_2_role = get_user_forum_role(user_b.user_id, &forum_2.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_b_forum_2_role.user_id, user_b.user_id);
    assert_eq!(user_b_forum_2_role.forum_id, forum_2.forum_id);
    assert_eq!(user_b_forum_2_role.forum_name, forum_2.forum_name);
    assert_eq!(user_b_forum_2_role.grantor_id, user_a.user_id);
    assert_eq!(user_b_forum_2_role.permission_level, PermissionLevel::Ban);

    let user_c_forum_2_role = get_user_forum_role(user_c.user_id, &forum_2.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_c_forum_2_role.user_id, user_c.user_id);
    assert_eq!(user_c_forum_2_role.forum_id, forum_2.forum_id);
    assert_eq!(user_c_forum_2_role.forum_name, forum_2.forum_name);
    assert_eq!(user_c_forum_2_role.grantor_id, user_a.user_id);
    assert_eq!(user_c_forum_2_role.permission_level, PermissionLevel::Moderate);

    let user_a_forum_3_role = get_user_forum_role(user_a.user_id, &forum_3.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_a_forum_3_role.user_id, user_a.user_id);
    assert_eq!(user_a_forum_3_role.forum_id, forum_3.forum_id);
    assert_eq!(user_a_forum_3_role.forum_name, forum_3.forum_name);
    assert_eq!(user_a_forum_3_role.grantor_id, user_b.user_id);
    assert_eq!(user_a_forum_3_role.permission_level, PermissionLevel::None);

    let user_b_forum_3_role = get_user_forum_role(user_b.user_id, &forum_3.forum_name, &db_pool).await.expect("get_user_forum_role should return user role.");
    assert_eq!(user_b_forum_3_role.user_id, user_b.user_id);
    assert_eq!(user_b_forum_3_role.forum_id, forum_3.forum_id);
    assert_eq!(user_b_forum_3_role.forum_name, forum_3.forum_name);
    assert_eq!(user_b_forum_3_role.grantor_id, user_b.user_id);
    assert_eq!(user_b_forum_3_role.permission_level, PermissionLevel::Lead);

    assert_eq!(get_user_forum_role(user_c.user_id, &forum_3.forum_name, &db_pool).await, Err(AppError::NotFound));

    Ok(())
}

#[tokio::test]
async fn test_get_forum_role_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user_a = create_user("a", &db_pool).await;
    let user_b = create_user("b", &db_pool).await;
    let user_c = create_user("c", &db_pool).await;

    let forum = forum::ssr::create_forum("1", "forum", false, &user_a, &db_pool).await?;
    let user_a = User::get(user_a.user_id, &db_pool).await.expect("Should be able to reload user.");

    let user_a_forum_role = get_user_forum_role(user_a.user_id, &forum.forum_name, &db_pool).await.expect("User a should have lead role.");
    let (user_b_forum_role, _) = set_user_forum_role(
        user_b.user_id,
        &forum.forum_name,
        PermissionLevel::Manage,
        &user_a,
        &db_pool
    ).await.expect("User should be able to grant Manage permissions.");
    let (user_c_forum_role, _) = set_user_forum_role(
        user_c.user_id,
        &forum.forum_name,
        PermissionLevel::None,
        &user_a,
        &db_pool
    ).await.expect("User should be able to grant None permissions.");

    let forum_role_vec = get_forum_role_vec(&forum.forum_name, &db_pool).await.expect("Should load forum role vec");

    assert_eq!(forum_role_vec.len(), 3);
    assert!(forum_role_vec.contains(&user_a_forum_role));
    assert!(forum_role_vec.contains(&user_b_forum_role));
    assert!(forum_role_vec.contains(&user_c_forum_role));

    Ok(())
}

#[tokio::test]
async fn test_set_user_forum_role() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let lead_user = create_user("lead", &db_pool).await;
    let ordinary_user = create_user("a", &db_pool).await;
    let moderator = create_user("mod", &db_pool).await;

    let forum_name = "forum";
    let forum = forum::ssr::create_forum(forum_name, "forum", false, &lead_user, &db_pool).await?;
    let lead_user = User::get(lead_user.user_id, &db_pool)
        .await
        .expect("Should be able to reload lead_user.");

    // test elect moderator
    let (moderate_role, prev_leader_id) = set_user_forum_role(
        moderator.user_id,
        forum_name,
        PermissionLevel::Moderate,
        &lead_user,
        &db_pool,
    )
        .await
        .expect("Moderate role should be assignable by lead_user.");
    assert_eq!(moderate_role.user_id, moderator.user_id);
    assert_eq!(moderate_role.forum_id, forum.forum_id);
    assert_eq!(moderate_role.forum_name, forum.forum_name);
    assert_eq!(moderate_role.grantor_id, lead_user.user_id);
    assert_eq!(moderate_role.permission_level, PermissionLevel::Moderate);
    assert_eq!(prev_leader_id, None);
    let moderator = User::get(moderator.user_id, &db_pool)
        .await
        .expect("Should be able to reload moderator.");
    assert_eq!(
        moderator.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Moderate)
    );

    // test need elect permissions to add moderators
    assert_eq!(
        set_user_forum_role(
            ordinary_user.user_id,
            forum_name,
            PermissionLevel::Moderate,
            &moderator,
            &db_pool
        ).await,
        Err(AppError::InsufficientPrivileges)
    );

    // test change permission level to Elect
    let (moderate_role, prev_leader_id) = set_user_forum_role(
        moderator.user_id,
        forum_name,
        PermissionLevel::Manage,
        &lead_user,
        &db_pool,
    )
        .await
        .expect("lead_user should be able to update role to Manage.");
    assert_eq!(moderate_role.user_id, moderator.user_id);
    assert_eq!(moderate_role.forum_id, forum.forum_id);
    assert_eq!(moderate_role.forum_name, forum.forum_name);
    assert_eq!(moderate_role.grantor_id, lead_user.user_id);
    assert_eq!(moderate_role.permission_level, PermissionLevel::Manage);
    assert_eq!(prev_leader_id, None);
    let moderator = User::get(moderator.user_id, &db_pool)
        .await
        .expect("Should be able to reload moderator after role update.");
    assert_eq!(
        moderator.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Manage)
    );

    // test can now elect other moderators
    let (moderate_role, prev_leader_id) = set_user_forum_role(
        ordinary_user.user_id,
        forum_name,
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
        .expect("Should be able to reload ordinary_user.");
    assert_eq!(
        ordinary_user.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Moderate)
    );

    // test moderator cannot set leader or downgrade higher up moderator
    assert!(
        set_user_forum_role(
            ordinary_user.user_id,
            forum_name,
            PermissionLevel::Lead,
            &moderator,
            &db_pool
        ).await.is_err()
    );
    assert_eq!(
        set_user_forum_role(
            lead_user.user_id,
            forum_name,
            PermissionLevel::Moderate,
            &moderator,
            &db_pool
        ).await,
        Err(AppError::InsufficientPrivileges)
    );

    // test leader can choose another leader
    let (new_lead_role, prev_leader_id) = set_user_forum_role(
        ordinary_user.user_id,
        forum_name,
        PermissionLevel::Lead,
        &lead_user,
        &db_pool,
    )
        .await
        .expect("lead_user should be able to elect new leader.");

    assert_eq!(new_lead_role.user_id, ordinary_user.user_id);
    assert_eq!(new_lead_role.forum_id, forum.forum_id);
    assert_eq!(new_lead_role.forum_name, forum.forum_name);
    assert_eq!(new_lead_role.grantor_id, moderator.user_id);
    assert_eq!(new_lead_role.permission_level, PermissionLevel::Lead);
    assert_eq!(prev_leader_id, Some(lead_user.user_id));
    let ordinary_user = User::get(ordinary_user.user_id, &db_pool)
        .await
        .expect("Should be able to reload ordinary_user after lead update.");
    assert_eq!(
        ordinary_user.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Lead)
    );
    let prev_lead_user = User::get(lead_user.user_id, &db_pool)
        .await
        .expect("Should be able to reload lead_use after lead update.");
    assert_eq!(
        prev_lead_user.permission_by_forum_map.get(forum_name),
        Some(&PermissionLevel::Manage)
    );

    Ok(())
}

#[tokio::test]
async fn test_set_user_admin_role() -> Result<(), AppError> {

    let db_pool = get_db_pool().await;
    let ordinary_user = create_user("user", &db_pool).await;
    let moderator = create_user("mod", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;

    // ordinary user cannot set admin role
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Admin, &ordinary_user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Moderator, &ordinary_user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::None, &ordinary_user, &db_pool).await, Err(AppError::InsufficientPrivileges));

    // admin can set admin roles
    admin.admin_role = AdminRole::Admin;
    let sql_admin = set_user_admin_role(admin.user_id, AdminRole::Admin, &admin, &db_pool).await.expect("Admin should be able to grant admin role.");
    assert_eq!(sql_admin.user_id, admin.user_id);
    assert_eq!(sql_admin.admin_role, AdminRole::Admin);
    let admin = User::get(admin.user_id, &db_pool).await.expect("Should be able to reload admin.");
    assert_eq!(admin.admin_role, AdminRole::Admin);

    let sql_moderator = set_user_admin_role(moderator.user_id, AdminRole::Moderator, &admin, &db_pool).await.expect("Admin should be able to grant moderator role.");
    assert_eq!(sql_moderator.user_id, moderator.user_id);
    assert_eq!(sql_moderator.admin_role, AdminRole::Moderator);
    let moderator = User::get(moderator.user_id, &db_pool).await.expect("Should be able to reload moderator.");
    assert_eq!(moderator.admin_role, AdminRole::Moderator);

    // moderator cannot set admin roles
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Admin, &moderator, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::Moderator, &moderator, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(set_user_admin_role(ordinary_user.user_id, AdminRole::None, &moderator, &db_pool).await, Err(AppError::InsufficientPrivileges));

    Ok(())
}
