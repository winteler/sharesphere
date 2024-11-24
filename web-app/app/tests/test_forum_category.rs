use crate::common::{create_user, get_db_pool};
use app::errors::AppError;
use app::forum::ssr::create_forum;
use app::forum_category::ssr::{delete_forum_category, get_forum_category_vec, set_forum_category, CATEGORY_NOT_DELETED_STR};
use app::post::ssr::create_post;
use app::user::User;

mod common;
mod data_factory;
#[tokio::test]
async fn test_get_forum_category_vec() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let forum_1 = create_forum("1", "1", false, &user, &db_pool).await?;
    let forum_2 = create_forum("2", "2", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let forum_1_category_1 = set_forum_category(
        &forum_1.forum_name,
        "1",
        "1",
        false,
        &user,
        &db_pool
    ).await.expect("Category 1 should be added.");

    let forum_1_category_1_updated = set_forum_category(
        &forum_1_category_1.forum_name,
        &forum_1_category_1.category_name,
        "updated",
        true,
        &user,
        &db_pool
    ).await.expect("Category 1 should be added.");

    let forum_1_category_2 = set_forum_category(
        &forum_1.forum_name,
        "2",
        "2",
        true,
        &user,
        &db_pool
    ).await.expect("Category 2 should be added.");

    let forum_1_category_off = set_forum_category(
        &forum_1.forum_name,
        "0",
        "0",
        false,
        &user,
        &db_pool
    ).await.expect("Category off should be added.");

    let forum_2_category_1 = set_forum_category(
        &forum_2.forum_name,
        "1",
        "1",
        true,
        &user,
        &db_pool
    ).await.expect("Category 1 should be added.");

    let forum_1_category_vec = get_forum_category_vec(
        &forum_1.forum_name,
        &db_pool
    ).await.expect("Should load forum categories");
    let forum_2_category_vec = get_forum_category_vec(
        &forum_2.forum_name,
        &db_pool
    ).await?;

    assert_eq!(forum_1_category_vec.len(), 3);
    assert_eq!(forum_1_category_vec.first(), Some(&forum_1_category_1_updated));
    assert_eq!(forum_1_category_vec.get(1), Some(&forum_1_category_2));
    assert_eq!(forum_1_category_vec.get(2), Some(&forum_1_category_off));
    assert_eq!(forum_2_category_vec.len(), 1);
    assert_eq!(forum_2_category_vec.first(), Some(&forum_2_category_1));

    Ok(())
}

#[tokio::test]
async fn test_set_forum_category() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;
    let forum_2 = create_forum("forum_2", "a", false, &user, &db_pool).await?;

    let category_name = "a";
    let description = "b";

    // Cannot create two categories with the same name in one forum
    assert_eq!(
        set_forum_category(
            &forum.forum_name,
            category_name,
            description,
            true,
            &user,
            &db_pool
        ).await,
        Err(AppError::InsufficientPrivileges)
    );

    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let forum_category = set_forum_category(
        &forum.forum_name,
        category_name,
        description,
        true,
        &user,
        &db_pool
    ).await.expect("Category should be added.");

    assert_eq!(forum_category.forum_id, forum.forum_id);
    assert_eq!(forum_category.forum_name, forum.forum_name);
    assert_eq!(forum_category.category_name, category_name);
    assert_eq!(forum_category.description, description);
    assert_eq!(forum_category.creator_id, user.user_id);
    assert!(forum_category.is_active);
    assert_eq!(forum_category.delete_timestamp, None);

    let updated_description = "c";
    let updated_category = set_forum_category(
        &forum.forum_name,
        category_name,
        updated_description,
        false,
        &user,
        &db_pool
    ).await.expect("Category should be updated.");

    assert_eq!(updated_category.forum_id, forum.forum_id);
    assert_eq!(updated_category.forum_name, forum.forum_name);
    assert_eq!(updated_category.category_name, category_name);
    assert_eq!(updated_category.description, updated_description);
    assert_eq!(updated_category.creator_id, user.user_id);
    assert!(!updated_category.is_active);
    assert_eq!(updated_category.delete_timestamp, None);

    // Can create a category with the same name for a different forum
    let forum_2_category = set_forum_category(
        &forum_2.forum_name,
        category_name,
        description,
        false,
        &user,
        &db_pool
    ).await.expect("Category should be added.");

    assert_eq!(forum_2_category.forum_id, forum_2.forum_id);
    assert_eq!(forum_2_category.forum_name, forum_2.forum_name);
    assert_eq!(forum_2_category.category_name, category_name);
    assert_eq!(forum_2_category.description, description);
    assert_eq!(forum_2_category.creator_id, user.user_id);
    assert!(!forum_2_category.is_active);
    assert_eq!(forum_2_category.delete_timestamp, None);

    Ok(())
}

#[tokio::test]
async fn test_delete_forum_category() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_user("test", &db_pool).await;
    let forum = create_forum("forum", "a", false, &user, &db_pool).await?;
    let user = User::get(user.user_id, &db_pool).await.expect("User should be loaded after forum creation");

    let category_name = "a";
    let forum_category = set_forum_category(
        &forum.forum_name,
        category_name,
        "b",
        true,
        &user,
        &db_pool
    ).await.expect("Category should be added.");

    delete_forum_category(&forum.forum_name, &forum_category.category_name, &user, &db_pool).await.expect("Forum category should be deleted.");

    assert!(get_forum_category_vec(&forum.forum_name, &db_pool).await.expect("Forum category should be deleted.").is_empty());

    let forum_category = set_forum_category(
        &forum.forum_name,
        category_name,
        "b",
        true,
        &user,
        &db_pool
    ).await.expect("Category should be added.");

    create_post(
        &forum.forum_name,
        "a",
        "b",
        None,
        false,
        false,
        false,
        Some(forum_category.category_id),
        &user,
        &db_pool
    ).await.expect("Post should be created.");

    assert_eq!(
        delete_forum_category(&forum.forum_name, &forum_category.category_name, &user, &db_pool).await,
        Err(AppError::InternalServerError(String::from(CATEGORY_NOT_DELETED_STR))),
    );

    Ok(())
}