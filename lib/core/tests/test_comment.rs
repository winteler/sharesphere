use rand::Rng;

pub use crate::common::*;
pub use crate::data_factory::*;
use crate::utils::{get_vote_from_comment_num, sort_comment_tree, COMMENT_SORT_TYPE_ARRAY};
use sharesphere_core::comment::ssr::{create_comment, delete_comment, get_comment_by_id, get_comment_sphere, get_comment_tree_by_id, get_post_comment_tree, update_comment};
use sharesphere_core::comment::{CommentWithChildren, COMMENT_BATCH_SIZE};
use sharesphere_core::post::ssr::get_post_by_id;
use sharesphere_core::ranking::{CommentSortType, SortType};
use sharesphere_auth::user::User;
use sharesphere_utils::editor::get_styled_html_from_markdown;
use sharesphere_utils::errors::AppError;

mod common;
mod data_factory;
mod utils;

#[tokio::test]
async fn test_get_comment_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, _, expected_comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let comment = get_comment_by_id(expected_comment.comment_id, &db_pool).await.expect("Should be able to get comment sphere.");

    assert_eq!(comment, expected_comment);
    
    Ok(())
}

#[tokio::test]
async fn test_get_comment_sphere() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (expected_sphere, _, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let sphere = get_comment_sphere(comment.comment_id, &db_pool).await.expect("Should be able to get comment sphere.");

    assert_eq!(sphere, expected_sphere);

    Ok(())
}

#[tokio::test]
async fn test_get_post_comment_tree() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let sphere_name = "sphere";
    sharesphere_core::sphere::ssr::create_sphere(
        sphere_name,
        "sphere",
        false,
        &user,
        &db_pool,
    ).await?;

    let num_comments = 200;
    let mut rng = rand::rng();

    let (post, mut expected_comment_tree) = create_post_with_comment_tree(
        sphere_name,
        "Post with comments",
        num_comments,
        (0..num_comments).map(|i| match i {
            i if i > 1 && (i % 2 == 0) => Some(rng.random_range(0..i-1)),
            _ => None,
        }).collect(),
        (0..num_comments).map(|i| (i as i32) - (num_comments as i32)/2).collect(),
        (0..num_comments).map(get_vote_from_comment_num).collect(),
        &user,
        &db_pool
    ).await;

    // reload user to refresh moderator permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let pinned_comment = create_comment(post.post_id, None, "1", None, true, &user, &db_pool).await?;

    expected_comment_tree.push(CommentWithChildren {
        comment: pinned_comment.clone(),
        vote: None,
        child_comments: Vec::new(),
    });

    for sort_type in COMMENT_SORT_TYPE_ARRAY {
        println!("Sort type: {}", sort_type.to_string());
        let comment_tree = get_post_comment_tree(
            post.post_id,
            SortType::Comment(sort_type),
            None,
            Some(user.user_id),
            COMMENT_BATCH_SIZE,
            0,
            &db_pool,
        ).await?;

        assert_eq!(comment_tree.is_empty(), false);
        assert_eq!(comment_tree[0].comment, pinned_comment);

        sort_comment_tree(&mut expected_comment_tree, sort_type, true);
        assert_eq!(comment_tree, expected_comment_tree[..(COMMENT_BATCH_SIZE as usize)]);
        let offset_comment_tree = get_post_comment_tree(
            post.post_id,
            SortType::Comment(sort_type),
            None,
            Some(user.user_id),
            COMMENT_BATCH_SIZE,
            COMMENT_BATCH_SIZE,
            &db_pool,
        ).await?;

        assert_eq!(offset_comment_tree, expected_comment_tree[(COMMENT_BATCH_SIZE as usize)..(2*COMMENT_BATCH_SIZE as usize)]);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_post_comment_tree_with_depth() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;

    let comment_1 = create_comment(
        post.post_id, None, "1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1");

    let comment_2 = create_comment(
        post.post_id, None, "2", None, false, &user, &db_pool
    ).await.expect("Should create comment 2");
    let comment_2 = set_comment_score(comment_2.comment_id, 1, &db_pool).await.expect("Should set comment 2 score");

    let comment_1_1 = create_comment(
        post.post_id, Some(comment_1.comment_id), "1_1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_1");

    let comment_1_2 = create_comment(
        post.post_id, Some(comment_1.comment_id), "1_2", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_2");
    let comment_1_2 = set_comment_score(comment_1_2.comment_id, 1, &db_pool).await.expect("Should set comment 1_2 score");

    let comment_1_2_1 = create_comment(
        post.post_id, Some(comment_1_2.comment_id), "1_2_1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_2_1");

    let _comment_1_2_1_1 = create_comment(
        post.post_id, Some(comment_1_2_1.comment_id), "1_2_1_1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_2_1_1");

    let depth_0_comment_tree = get_post_comment_tree(
        post.post_id,
        SortType::Comment(CommentSortType::Best),
        Some(0),
        Some(user.user_id),
        COMMENT_BATCH_SIZE,
        0,
        &db_pool,
    ).await.expect("Should get depth 1 comment tree");

    assert_eq!(depth_0_comment_tree.len(), 2);
    let depth_0_elem_1 = depth_0_comment_tree.first().expect("Should get depth_0_comment_tree 1st element");
    let depth_0_elem_2 = depth_0_comment_tree.get(1).expect("Should get depth_0_comment_tree 2nd element");
    assert_eq!(depth_0_elem_1.comment, comment_2);
    assert_eq!(depth_0_elem_2.comment, comment_1);
    assert!(depth_0_elem_1.child_comments.is_empty());
    assert_eq!(depth_0_elem_2.child_comments.len(), 2);

    let depth_1_elem_1 = depth_0_elem_2.child_comments.first().expect("Should get depth_0_elem_2 1st child");
    let depth_1_elem_2 = depth_0_elem_2.child_comments.get(1).expect("Should get depth_0_elem_2 2nd child");
    assert_eq!(depth_1_elem_1.comment, comment_1_2);
    assert_eq!(depth_1_elem_2.comment, comment_1_1);
    assert!(depth_1_elem_1.child_comments.is_empty());
    assert!(depth_1_elem_2.child_comments.is_empty());

    let depth_1_comment_tree = get_post_comment_tree(
        post.post_id,
        SortType::Comment(CommentSortType::Best),
        Some(1),
        Some(user.user_id),
        COMMENT_BATCH_SIZE,
        0,
        &db_pool,
    ).await.expect("Should get depth 1 comment tree");

    assert_eq!(depth_1_comment_tree.len(), 2);
    let depth_0_elem_1 = depth_1_comment_tree.first().expect("Should get depth_1_comment_tree 1st element");
    let depth_0_elem_2 = depth_1_comment_tree.get(1).expect("Should get depth_1_comment_tree 2nd element");
    assert_eq!(depth_0_elem_1.comment, comment_2);
    assert_eq!(depth_0_elem_2.comment, comment_1);
    assert!(depth_0_elem_1.child_comments.is_empty());
    assert_eq!(depth_0_elem_2.child_comments.len(), 2);

    let depth_1_elem_1 = depth_0_elem_2.child_comments.first().expect("Should get depth_0_elem_2 1st child");
    let depth_1_elem_2 = depth_0_elem_2.child_comments.get(1).expect("Should get depth_0_elem_2 2nd child");
    assert_eq!(depth_1_elem_1.comment, comment_1_2);
    assert_eq!(depth_1_elem_2.comment, comment_1_1);
    assert_eq!(depth_1_elem_1.child_comments.len(), 1);
    assert!(depth_1_elem_2.child_comments.is_empty());

    let depth_2_elem_1 = depth_1_elem_1.child_comments.first().expect("Should get depth_1_elem_1 1st child");
    assert_eq!(depth_2_elem_1.comment, comment_1_2_1);
    assert!(depth_2_elem_1.child_comments.is_empty());
}

#[tokio::test]
async fn test_get_comment_tree_by_id() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;

    let comment_1 = create_comment(
        post.post_id, None, "1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1");

    let comment_2 = create_comment(
        post.post_id, None, "2", None, false, &user, &db_pool
    ).await.expect("Should create comment 2");

    let comment_1_1 = create_comment(
        post.post_id, Some(comment_1.comment_id), "1_1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_1");

    let comment_1_2 = create_comment(
        post.post_id, Some(comment_1.comment_id), "1_2", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_2");
    let comment_1_2 = set_comment_score(comment_1_2.comment_id, 1, &db_pool).await.expect("Should set comment 1_2 score");

    let comment_1_2_1 = create_comment(
        post.post_id, Some(comment_1_2.comment_id), "1_2_1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_2_1");

    let _comment_1_2_1_1 = create_comment(
        post.post_id, Some(comment_1_2_1.comment_id), "1_2_1_1", None, false, &user, &db_pool
    ).await.expect("Should create comment 1_2_1_1");

    let full_comment_tree = get_post_comment_tree(
        post.post_id,
        SortType::Comment(CommentSortType::Best),
        None,
        Some(user.user_id),
        COMMENT_BATCH_SIZE,
        0,
        &db_pool
    ).await.expect("Should get full comment tree");

    let expected_comment_1_tree = full_comment_tree.iter().find(
        |comment| comment.comment == comment_1
    ).expect("Should find comment 1");

    let expected_comment_2_tree = full_comment_tree.iter().find(
        |comment| comment.comment == comment_2
    ).expect("Should find comment 2");

    let mut expected_comment_1_1_tree = expected_comment_1_tree.clone();
    expected_comment_1_1_tree.child_comments = expected_comment_1_1_tree.child_comments.into_iter().filter(|comment| comment.comment == comment_1_1).collect();

    let mut expected_comment_1_2_tree = expected_comment_1_tree.clone();
    expected_comment_1_2_tree.child_comments = expected_comment_1_2_tree.child_comments.into_iter().filter(|comment| comment.comment == comment_1_2).collect();

    let expected_comment_1_2_1_tree = expected_comment_1_tree.child_comments.iter().find(
        |comment | comment.comment == comment_1_2
    ).expect("Should find comment 1_2");

    for sort_type in COMMENT_SORT_TYPE_ARRAY {
        let comment_1_tree = get_comment_tree_by_id(
            comment_1.comment_id, SortType::Comment(sort_type), None, Some(user.user_id), &db_pool
        ).await.expect("Should get comment 1 tree");
        assert_eq!(comment_1_tree, *expected_comment_1_tree);

        let comment_2_tree = get_comment_tree_by_id(
            comment_2.comment_id, SortType::Comment(sort_type), None, Some(user.user_id), &db_pool
        ).await.expect("Should get comment 2 tree");
        assert_eq!(comment_2_tree, *expected_comment_2_tree);

        let comment_1_1_tree = get_comment_tree_by_id(
            comment_1_1.comment_id, SortType::Comment(sort_type), None, Some(user.user_id), &db_pool
        ).await.expect("Should get comment 1_1 tree");
        assert_eq!(comment_1_1_tree, expected_comment_1_1_tree);

        let comment_1_2_tree = get_comment_tree_by_id(
            comment_1_2.comment_id, SortType::Comment(sort_type), None, Some(user.user_id), &db_pool
        ).await.expect("Should get comment 1_2 tree");
        assert_eq!(comment_1_2_tree, expected_comment_1_2_tree);

        let comment_1_2_1_tree = get_comment_tree_by_id(
            comment_1_2_1.comment_id, SortType::Comment(sort_type), None, Some(user.user_id), &db_pool
        ).await.expect("Should get comment 1_2_1 tree");
        assert_eq!(comment_1_2_1_tree, *expected_comment_1_2_1_tree);
    }

    let depth_0_comment_1_tree = get_comment_tree_by_id(
        comment_1.comment_id,
        SortType::Comment(CommentSortType::Best),
        Some(0),
        Some(user.user_id),
        &db_pool
    ).await.expect("Should get depth 0 comment 1 tree");

    assert_eq!(depth_0_comment_1_tree.comment, comment_1);
    assert_eq!(depth_0_comment_1_tree.child_comments.len(), 2);
    assert_eq!(
        depth_0_comment_1_tree.child_comments.first().expect("Should get depth_0_comment_1_tree 1st child").comment,
        comment_1_2,
    );
    assert_eq!(
        depth_0_comment_1_tree.child_comments.get(1).expect("Should get depth_0_comment_1_tree 2nd child").comment,
        comment_1_1,
    );

    let depth_0_comment_1_tree = get_comment_tree_by_id(
        comment_1.comment_id,
        SortType::Comment(CommentSortType::Best),
        Some(0),
        Some(user.user_id),
        &db_pool
    ).await.expect("Should get depth 0 comment 1 tree");

    assert_eq!(depth_0_comment_1_tree.comment, comment_1);
    assert_eq!(depth_0_comment_1_tree.child_comments.len(), 2);
    let depth_1_comment_1_elem_1 = depth_0_comment_1_tree.child_comments.first().expect("Should get depth_0_comment_1_tree 1st child");
    let depth_1_comment_1_elem_2 = depth_0_comment_1_tree.child_comments.get(1).expect("Should get depth_0_comment_1_tree 2nd child");
    assert_eq!(
        depth_1_comment_1_elem_1.comment,
        comment_1_2,
    );
    assert_eq!(
        depth_1_comment_1_elem_2.comment,
        comment_1_1,
    );
    assert!(depth_1_comment_1_elem_1.child_comments.is_empty());
    assert!(depth_1_comment_1_elem_2.child_comments.is_empty());

    let depth_1_comment_1_tree = get_comment_tree_by_id(
        comment_1.comment_id,
        SortType::Comment(CommentSortType::Best),
        Some(1),
        Some(user.user_id),
        &db_pool
    ).await.expect("Should get depth 1 comment 1 tree");

    assert_eq!(depth_1_comment_1_tree.comment, comment_1);
    assert_eq!(depth_1_comment_1_tree.child_comments.len(), 2);

    let depth_1_comment_1_elem_1 = depth_1_comment_1_tree.child_comments.first().expect("Should get depth_1_comment_1_tree 1st child");
    let depth_1_comment_1_elem_2 = depth_1_comment_1_tree.child_comments.get(1).expect("Should get depth_1_comment_1_tree 2nd child");
    assert_eq!(depth_1_comment_1_elem_1.comment, comment_1_2);
    assert_eq!(depth_1_comment_1_elem_2.comment, comment_1_1);
    assert_eq!(depth_1_comment_1_elem_1.child_comments.len(), 1);
    assert!(depth_1_comment_1_elem_2.child_comments.is_empty());

    let depth_2_comment_1_elem_1 = depth_1_comment_1_elem_1.child_comments.first().expect("Should get depth_1_comment_1_elem_1 1st child");
    assert_eq!(depth_2_comment_1_elem_1.comment, comment_1_2_1);
    assert!(depth_2_comment_1_elem_1.child_comments.is_empty());
}

#[tokio::test]
async fn test_create_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;
    let base_user = create_user("base", &db_pool).await;

    let (_sphere, post) = create_sphere_with_post("sphere", &mut user, &db_pool).await;

    let comment_body = "a";
    let comment = create_comment(post.post_id, None, comment_body, None, false, &user, &db_pool).await.expect("Comment should be created.");

    assert_eq!(comment.body, comment_body);
    assert_eq!(comment.markdown_body, None);
    assert_eq!(comment.is_edited, false);
    assert_eq!(comment.moderator_message, None);
    assert_eq!(comment.infringed_rule_id, None);
    assert_eq!(comment.infringed_rule_title, None);
    assert_eq!(comment.parent_id, None);
    assert_eq!(comment.post_id, post.post_id);
    assert_eq!(comment.creator_id, user.user_id);
    assert_eq!(comment.creator_name, user.username);
    assert_eq!(comment.is_creator_moderator, true);
    assert_eq!(comment.moderator_id, None);
    assert_eq!(comment.moderator_name, None);
    assert_eq!(comment.is_pinned, false);
    assert_eq!(comment.score, 0);
    assert_eq!(comment.score_minus, 0);
    assert_eq!(comment.edit_timestamp, None);
    assert_eq!(comment.delete_timestamp, None);

    // cannot create pinned comment without moderator permissions (need to reload user to actualize them)
    assert_eq!(
        create_comment(post.post_id, Some(comment.comment_id), comment_body, None, true, &base_user, &db_pool).await,
        Err(AppError::InsufficientPrivileges),
    );

    let post = get_post_by_id(post.post_id, &db_pool).await.expect("Should get post.");
    assert_eq!(post.num_comments, 1);

    let user = User::get(user.user_id, &db_pool).await.expect("User should be reloaded.");
    let markdown_body = "# markdown";
    let child_comment = create_comment(post.post_id, Some(comment.comment_id), comment_body, Some(markdown_body), true, &user, &db_pool).await.expect("Comment should be created.");

    assert_eq!(child_comment.body, comment_body);
    assert_eq!(child_comment.markdown_body, Some(String::from(markdown_body)));
    assert_eq!(child_comment.is_edited, false);
    assert_eq!(child_comment.moderator_message, None);
    assert_eq!(child_comment.infringed_rule_id, None);
    assert_eq!(child_comment.infringed_rule_title, None);
    assert_eq!(child_comment.parent_id, Some(comment.comment_id));
    assert_eq!(child_comment.post_id, post.post_id);
    assert_eq!(child_comment.creator_id, user.user_id);
    assert_eq!(child_comment.creator_name, user.username);
    assert_eq!(child_comment.is_creator_moderator, true);
    assert_eq!(child_comment.moderator_id, None);
    assert_eq!(child_comment.moderator_name, None);
    assert_eq!(child_comment.is_pinned, true);
    assert_eq!(child_comment.score, 0);
    assert_eq!(child_comment.score_minus, 0);
    assert_eq!(child_comment.edit_timestamp, None);
    assert_eq!(child_comment.delete_timestamp, None);

    let post = get_post_by_id(post.post_id, &db_pool).await.expect("Should get post.");
    assert_eq!(post.num_comments, 2);

    Ok(())
}

#[tokio::test]
async fn test_update_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post, comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let updated_markdown_body = "# Here is a comment with markdown";
    let updated_html_body = get_styled_html_from_markdown(String::from(updated_markdown_body)).await.expect("Should get html from markdown.");
    let updated_comment = update_comment(
        comment.comment_id,
        &updated_html_body,
        Some(updated_markdown_body),
        false,
        &user,
        &db_pool
    ).await?;

    assert_eq!(updated_comment.body, updated_html_body);
    assert_eq!(updated_comment.markdown_body, Some(String::from(updated_markdown_body)));
    assert!(
        updated_comment.edit_timestamp.is_some() &&
            updated_comment.edit_timestamp.unwrap() > updated_comment.create_timestamp &&
            updated_comment.create_timestamp == comment.create_timestamp
    );
    assert_eq!(updated_comment.delete_timestamp, None);

    // Cannot update moderated comment
    let moderated_comment = get_moderated_comment(&post, &user, &db_pool).await;
    assert_eq!(
        update_comment(
            moderated_comment.comment_id,
            &updated_html_body,
            Some(updated_markdown_body),
            false,
            &user,
            &db_pool
        ).await,
        Err(AppError::NotFound),
    );

    // Cannot update deleted comment
    let comment = create_comment(
        post.post_id,
        None,
        "update",
        None,
        false,
        &user,
        &db_pool
    ).await.expect("Comment should be created.");
    delete_comment(comment.comment_id, &user, &db_pool).await.expect("Comment should be deleted.");
    assert_eq!(
        update_comment(
            comment.comment_id,
            &updated_html_body,
            Some(updated_markdown_body),
            false,
            &user,
            &db_pool
        ).await,
        Err(AppError::NotFound),
    );

    Ok(())
}

#[tokio::test]
async fn test_delete_comment() {
    let db_pool = get_db_pool().await;
    let mut user = create_test_user(&db_pool).await;

    let (_, post, parent_comment) = create_sphere_with_post_and_comment("sphere", &mut user, &db_pool).await;

    let comment = create_comment(
        post.post_id,
        Some(parent_comment.comment_id),
        "comment",
        Some("markdown_comment"),
        true,
        &user,
        &db_pool
    ).await.expect("Comment should be created.");

    let deleted_comment = delete_comment(
        comment.comment_id,
        &user,
        &db_pool
    ).await.expect("Should deleted comment.");

    assert_eq!(deleted_comment.comment_id, comment.comment_id);
    assert_eq!(deleted_comment.parent_id, comment.parent_id);
    assert_eq!(deleted_comment.post_id, comment.post_id);
    assert_eq!(deleted_comment.body, "");
    assert_eq!(deleted_comment.markdown_body, None);
    assert_eq!(deleted_comment.creator_id, user.user_id);
    assert_eq!(deleted_comment.creator_name, "");
    assert_eq!(deleted_comment.is_pinned, false);
    assert!(
        deleted_comment.edit_timestamp.is_some() &&
            deleted_comment.edit_timestamp.unwrap() > deleted_comment.create_timestamp &&
            deleted_comment.create_timestamp == comment.create_timestamp
    );
    assert!(
        deleted_comment.delete_timestamp.is_some() &&
            deleted_comment.delete_timestamp.unwrap() > deleted_comment.create_timestamp
    );

    let deleted_parent_comment = delete_comment(
        parent_comment.comment_id,
        &user,
        &db_pool
    ).await.expect("Should delete parent comment.");

    assert_eq!(deleted_parent_comment.comment_id, parent_comment.comment_id);
    assert_eq!(deleted_parent_comment.parent_id, None);
    assert_eq!(deleted_parent_comment.post_id, parent_comment.post_id);
    assert_eq!(deleted_parent_comment.body, "");
    assert_eq!(deleted_parent_comment.markdown_body, None);
    assert_eq!(deleted_comment.creator_id, user.user_id);
    assert_eq!(deleted_comment.creator_name, "");
    assert_eq!(deleted_parent_comment.is_pinned, false);
    assert!(
        deleted_parent_comment.edit_timestamp.is_some() &&
            deleted_parent_comment.edit_timestamp.unwrap() > deleted_parent_comment.create_timestamp &&
            deleted_parent_comment.create_timestamp == parent_comment.create_timestamp
    );
    assert!(
        deleted_parent_comment.delete_timestamp.is_some() &&
            deleted_parent_comment.delete_timestamp.unwrap() > deleted_parent_comment.create_timestamp
    );

    let moderated_comment = get_moderated_comment(&post, &user, &db_pool).await;
    assert_eq!(
        delete_comment(
            moderated_comment.comment_id,
            &user,
            &db_pool
        ).await,
        Err(AppError::NotFound),
    );
}
