use crate::common::{create_user, get_db_pool};
use app::errors::AppError;
use app::forum::ssr::create_forum;
use app::role::AdminRole;
use app::rule::ssr::{add_rule, get_forum_rule_vec, load_rule_by_id, remove_rule, update_rule};
use app::user::User;

mod common;
mod data_factory;

#[tokio::test]
async fn test_get_rule_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum_1 = create_forum("1", "a", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let expected_common_rule = add_rule(None, 0, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let expected_forum_rule = add_rule(Some(&forum_1.forum_name), 1, "forum_1_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");

    let common_rule = load_rule_by_id(expected_common_rule.rule_id, &db_pool).await?;
    let forum_rule = load_rule_by_id(expected_forum_rule.rule_id, &db_pool).await?;

    assert_eq!(common_rule, expected_common_rule);
    assert_eq!(forum_rule, expected_forum_rule);

    Ok(())
}

#[tokio::test]
async fn test_get_forum_rule_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum_1 = create_forum("1", "a", false, &user, &db_pool).await?;
    let forum_2 = create_forum("2", "b", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let common_rule_1 = add_rule(None, 0, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_2 = add_rule(None, 3, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let forum_1_rule_1 = add_rule(Some(&forum_1.forum_name), 1, "forum_1_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");
    let forum_1_rule_2 = add_rule(Some(&forum_1.forum_name), 2, "forum_1_rule_2", "test", &user, &db_pool).await.expect("Rule should be created.");
    let forum_2_rule_1 = add_rule(Some(&forum_2.forum_name), 1, "forum_2_rule_1", "test", &user, &db_pool).await.expect("Rule should be created.");

    let forum_1_rule_vec = get_forum_rule_vec(&forum_1.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_1_rule_vec.len(), 4);
    assert_eq!(forum_1_rule_vec.first(), Some(&common_rule_1));
    assert_eq!(forum_1_rule_vec.get(1), Some(&common_rule_2));
    assert_eq!(forum_1_rule_vec.get(2), Some(&forum_1_rule_1));
    assert_eq!(forum_1_rule_vec.get(3), Some(&forum_1_rule_2));

    let forum_2_rule_vec = get_forum_rule_vec(&forum_2.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_2_rule_vec.len(), 3);
    assert_eq!(forum_2_rule_vec.first(), Some(&common_rule_1));
    assert_eq!(forum_2_rule_vec.get(1), Some(&common_rule_2));
    assert_eq!(forum_2_rule_vec.get(2), Some(&forum_2_rule_1));

    Ok(())
}

#[tokio::test]
async fn test_add_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";

    assert_eq!(add_rule(None, 0, title, description, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(add_rule(None, 0, title, description, &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let common_rule_1 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(common_rule_1.forum_id, None);
    assert_eq!(common_rule_1.forum_name, None);
    assert_eq!(common_rule_1.priority, 0);
    assert_eq!(common_rule_1.title, title);
    assert_eq!(common_rule_1.description, description);
    assert_eq!(common_rule_1.user_id, admin.user_id);

    assert_eq!(add_rule(Some(&forum.forum_name), 1, title, description, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let rule_1 = add_rule(Some(&forum.forum_name), 1, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    // creating rule_2 should increment rule_1's priority
    let rule_2 = add_rule(Some(&forum.forum_name), 1, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(rule_1.forum_id, Some(forum.forum_id));
    assert_eq!(rule_1.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_1.priority, 1);
    assert_eq!(rule_1.title, title);
    assert_eq!(rule_1.description, description);
    assert_eq!(rule_1.user_id, lead.user_id);

    assert_eq!(rule_2.forum_id, Some(forum.forum_id));
    assert_eq!(rule_2.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_2.priority, 1);
    assert_eq!(rule_2.title, title);
    assert_eq!(rule_2.description, description);
    assert_eq!(rule_2.user_id, admin.user_id);

    let common_rule_2 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 4);
    assert_eq!(forum_rule_vec.first(), Some(&common_rule_2));
    assert_eq!(forum_rule_vec.get(1).unwrap().rule_id, common_rule_1.rule_id);
    assert_eq!(forum_rule_vec.get(2), Some(&rule_2));
    assert_eq!(forum_rule_vec.get(3).unwrap().rule_id, rule_1.rule_id);

    Ok(())
}

#[tokio::test]
async fn test_update_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";
    let updated_title = "updated";
    let updated_desc = "updated";

    let common_rule_1 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_2 = add_rule(None, 1, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_3 = add_rule(None, 2, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let rule_1 = add_rule(Some(&forum.forum_name), 0, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    let rule_2 = add_rule(Some(&forum.forum_name), 1, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let rule_3 = add_rule(Some(&forum.forum_name), 2, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(update_rule(None, 0, 1, updated_title, updated_desc, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(update_rule(None, 0, 1, updated_title, updated_desc,  &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let common_rule_1_updated = update_rule(None, 0, 1, updated_title, updated_desc, &admin, &db_pool).await.expect("Rule should be updated.");
    assert_eq!(common_rule_1_updated.rule_key, common_rule_1.rule_key);
    assert_eq!(common_rule_1_updated.priority, 1);
    assert_eq!(common_rule_1_updated.forum_id, None);
    assert_eq!(common_rule_1_updated.forum_name, None);
    assert_eq!(common_rule_1_updated.title, updated_title);
    assert_eq!(common_rule_1_updated.description, updated_desc);

    assert_eq!(update_rule(Some(&forum.forum_name), 1, 0, updated_title, updated_desc, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    let rule_2_updated = update_rule(Some(&forum.forum_name), 1, 0, updated_title, updated_desc,  &lead, &db_pool).await.expect("Rule should be updated.");
    assert_eq!(rule_2_updated.rule_key, rule_2.rule_key);
    assert_eq!(rule_2_updated.priority, 0);
    assert_eq!(rule_2_updated.forum_id, Some(forum.forum_id));
    assert_eq!(rule_2_updated.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_2_updated.title, updated_title);
    assert_eq!(rule_2_updated.description, updated_desc);
    let rule_3_updated = update_rule(Some(&forum.forum_name), 2, 1, updated_title, updated_desc, &admin, &db_pool).await.expect("Rule should be updated.");
    assert_eq!(rule_3_updated.rule_key, rule_3.rule_key);
    assert_eq!(rule_3_updated.priority, 1);
    assert_eq!(rule_3_updated.forum_id, Some(forum.forum_id));
    assert_eq!(rule_3_updated.forum_name, Some(forum.forum_name.clone()));
    assert_eq!(rule_3_updated.title, updated_title);
    assert_eq!(rule_3_updated.description, updated_desc);

    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 6);
    assert_eq!(forum_rule_vec.first().unwrap().rule_id, common_rule_2.rule_id);
    assert_eq!(forum_rule_vec.get(1), Some(&common_rule_1_updated));
    assert_eq!(forum_rule_vec.get(2), Some(&common_rule_3));
    assert_eq!(forum_rule_vec.get(3), Some(&rule_2_updated));
    assert_eq!(forum_rule_vec.get(4), Some(&rule_3_updated));
    assert_eq!(forum_rule_vec.get(5).unwrap().rule_id, rule_1.rule_id);

    Ok(())
}

#[tokio::test]
async fn test_remove_rule() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("user", &db_pool).await;
    let lead = create_user("lead", &db_pool).await;
    let mut admin = create_user("admin", &db_pool).await;
    admin.admin_role = AdminRole::Admin;
    let forum = create_forum("forum", "a", false, &lead, &db_pool).await?;
    let lead = User::get(lead.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let title = "title";
    let description = "description";

    let _common_rule_1 = add_rule(None, 0, title, description, &admin, &db_pool).await.expect("Rule should be created.");
    let common_rule_2 = add_rule(None, 1, "common", "0", &admin, &db_pool).await.expect("Rule should be created.");
    let _rule_1 = add_rule(Some(&forum.forum_name), 0, title, description, &lead, &db_pool).await.expect("Rule should be created.");
    let rule_2 = add_rule(Some(&forum.forum_name), 1, title, description, &admin, &db_pool).await.expect("Rule should be created.");

    assert_eq!(remove_rule(None, 0, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_rule(None, 0, &lead, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_rule(None, 0, &admin, &db_pool).await, Ok(()));

    assert_eq!(remove_rule(Some(&forum.forum_name), 0, &user, &db_pool).await, Err(AppError::InsufficientPrivileges));
    assert_eq!(remove_rule(Some(&forum.forum_name), 0, &lead, &db_pool).await, Ok(()));

    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 2);
    assert_eq!(forum_rule_vec.first().unwrap().rule_id, common_rule_2.rule_id);
    assert_eq!(forum_rule_vec.first().unwrap().priority, 0);
    assert_eq!(forum_rule_vec.get(1).unwrap().rule_id, rule_2.rule_id);
    assert_eq!(forum_rule_vec.get(1).unwrap().priority, 0);

    assert_eq!(remove_rule(Some(&forum.forum_name), 0, &admin, &db_pool).await, Ok(()));

    let forum_rule_vec = get_forum_rule_vec(&forum.forum_name, &db_pool).await.expect("Forum rules should be loaded");
    assert_eq!(forum_rule_vec.len(), 1);
    assert_eq!(forum_rule_vec.first().unwrap().rule_id, common_rule_2.rule_id);

    Ok(())
}