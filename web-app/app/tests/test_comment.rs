use rand::Rng;

pub use crate::common::*;
pub use crate::data_factory::*;
use app::comment;
use app::comment::ssr::{create_comment, get_comment_by_id, get_comment_forum};
use app::comment::{CommentSortType, CommentWithChildren, COMMENT_BATCH_SIZE};
use app::editor::get_styled_html_from_markdown;
use app::errors::AppError;
use app::post::ssr::get_post_by_id;
use app::ranking::{SortType, Vote, VoteValue};
use app::user::User;

mod common;
mod data_factory;

fn get_vote_from_comment_num(comment_num: usize) -> Option<VoteValue> {
    match comment_num % 3 {
        0 => Some(VoteValue::Down),
        1 => None,
        _ => Some(VoteValue::Up),
    }
}

fn test_comment_and_vote(
    comment_with_children: &CommentWithChildren,
    expected_user_id: i64,
    expected_post_id: i64,
) {
    // Test current comment
    assert_eq!(comment_with_children.comment.creator_id, expected_user_id);
    assert_eq!(comment_with_children.comment.post_id, expected_post_id);

    // Test associated vote
    let comment_num = comment_with_children
        .comment
        .body
        .parse::<usize>()
        .expect("Comment number in body should be parsable.");
    let expected_vote_value = get_vote_from_comment_num(comment_num);
    if let Some(expected_vote_value) = expected_vote_value {
        let vote: Vote = comment_with_children
            .vote
            .clone()
            .expect(format!("Comment {comment_num} should have a vote.").as_str());
        assert_eq!(vote.value, expected_vote_value);
        assert_eq!(vote.user_id, expected_user_id);
        assert_eq!(vote.post_id, expected_post_id);
        assert_eq!(vote.comment_id, Some(comment_with_children.comment.comment_id));
    } else {
        assert!(comment_with_children.vote.is_none());
    }
}

fn test_comment_vec(
    comment_vec: &[CommentWithChildren],
    sort_type: CommentSortType,
    expected_parent_id: Option<i64>,
    expected_user_id: i64,
    expected_post_id: i64,
) {
    for (index, child_comment) in comment_vec.iter().enumerate() {
        // Test that parent id is correct
        assert_eq!(
            child_comment.comment.parent_id,
            expected_parent_id,
        );
        // Test that the child comments are correctly sorted
        if index > 0 {
            let previous_child_comment = comment_vec.get(index - 1);
            assert!(previous_child_comment.is_some());
            let previous_child_comment = previous_child_comment.unwrap();
            assert!(previous_child_comment.comment.is_pinned || !child_comment.comment.is_pinned);
            assert!(
                (
                    previous_child_comment.comment.is_pinned && !child_comment.comment.is_pinned
                ) || match sort_type {
                    CommentSortType::Best =>
                        child_comment.comment.score <= previous_child_comment.comment.score,
                    CommentSortType::Recent =>
                        child_comment.comment.create_timestamp
                            <= previous_child_comment.comment.create_timestamp,
                }
            );
        }
        test_comment_with_children(child_comment, sort_type, expected_user_id, expected_post_id);
    }
}

fn test_comment_with_children(
    comment_with_children: &CommentWithChildren,
    sort_type: CommentSortType,
    expected_user_id: i64,
    expected_post_id: i64,
) {
    test_comment_and_vote(comment_with_children, expected_user_id, expected_post_id);
    // Test child comments
    test_comment_vec(&comment_with_children.child_comments, sort_type, Some(comment_with_children.comment.comment_id), expected_user_id, expected_post_id);
}

#[tokio::test]
async fn test_get_comment_by_id() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_, _, expected_comment) = create_forum_with_post_and_comment("forum", &user, &db_pool).await;

    let comment = get_comment_by_id(expected_comment.comment_id, &db_pool).await.expect("Should be able to get comment forum.");

    assert_eq!(comment, expected_comment);
    
    Ok(())
}

#[tokio::test]
async fn test_get_comment_forum() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (expected_forum, _, comment) = create_forum_with_post_and_comment("forum", &user, &db_pool).await;

    let forum = get_comment_forum(comment.comment_id, &db_pool).await.expect("Should be able to get comment forum.");

    assert_eq!(forum, expected_forum);

    Ok(())
}

#[tokio::test]
async fn test_get_post_comment_tree() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    app::forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &user,
        &db_pool,
    ).await?;

    let num_comments = 200;
    let mut rng = rand::thread_rng();

    let post = create_post_with_comments(
        forum_name,
        "Post with comments",
        num_comments,
        (1..num_comments+1).map(|i| match i {
            i if i > 2 && (i % 2 == 0) => Some(rng.gen_range(0..((i-1) as i64))+1),
            _ => None,
        }).collect(),
        (0..num_comments).map(|_| rng.gen_range(-100..101)).collect(),
        (0..num_comments).map(get_vote_from_comment_num).collect(),
        &user,
        &db_pool
    ).await?;

    // reload user to refresh moderator permissions
    let user = User::get(user.user_id, &db_pool).await.expect("Should reload user.");
    let pinned_comment = create_comment(post.post_id, None, "1", None, true, &user, &db_pool).await?;

    let comment_sort_type_array = [CommentSortType::Best, CommentSortType::Recent];

    for sort_type in comment_sort_type_array {
        let comment_tree = comment::ssr::get_post_comment_tree(
            post.post_id,
            SortType::Comment(sort_type),
            Some(user.user_id),
            COMMENT_BATCH_SIZE,
            0,
            &db_pool,
        ).await?;

        assert_eq!(comment_tree.is_empty(), false);
        assert_eq!(comment_tree[0].comment, pinned_comment);

        test_comment_vec(&comment_tree, sort_type, None, user.user_id, post.post_id);
        
        let offset_comment_tree = comment::ssr::get_post_comment_tree(
            post.post_id,
            SortType::Comment(sort_type),
            Some(user.user_id),
            COMMENT_BATCH_SIZE,
            COMMENT_BATCH_SIZE,
            &db_pool,
        ).await?;
        
        assert_eq!(offset_comment_tree.is_empty(), false);
        
        let init_load_last_comment = comment_tree.last().expect("Initial comment tree should have at least one comment.");
        let offset_load_first_comment = offset_comment_tree.first().expect("Offset comment tree should have at least one comment.");
        assert!(
            (
                init_load_last_comment.comment.is_pinned && !offset_load_first_comment.comment.is_pinned
            ) || match sort_type {
                CommentSortType::Best =>
                    init_load_last_comment.comment.score >= offset_load_first_comment.comment.score,
                CommentSortType::Recent =>
                    init_load_last_comment.comment.create_timestamp >= offset_load_first_comment.comment.create_timestamp,
            }
        );
        test_comment_vec(&offset_comment_tree, sort_type, None, user.user_id, post.post_id);
    }

    Ok(())
}

#[tokio::test]
async fn test_create_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_forum, post) = create_forum_with_post("forum", &user, &db_pool).await;

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
    assert_eq!(comment.is_creator_moderator, false); // user not refreshed yet
    assert_eq!(comment.moderator_id, None);
    assert_eq!(comment.moderator_name, None);
    assert_eq!(comment.is_pinned, false);
    assert_eq!(comment.score, 0);
    assert_eq!(comment.score_minus, 0);
    assert_eq!(comment.edit_timestamp, None);

    // cannot create pinned comment without moderator permissions (need to reload user to actualize them)
    assert_eq!(
        create_comment(post.post_id, Some(comment.comment_id), comment_body, None, true, &user, &db_pool).await,
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

    let post = get_post_by_id(post.post_id, &db_pool).await.expect("Should get post.");
    assert_eq!(post.num_comments, 2);

    Ok(())
}

#[tokio::test]
async fn test_update_comment() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;
    let user = create_test_user(&db_pool).await;

    let (_forum, _post, comment) = create_forum_with_post_and_comment("forum", &user, &db_pool).await;

    let updated_markdown_body = "# Here is a comment with markdown";
    let updated_html_body = get_styled_html_from_markdown(String::from(updated_markdown_body)).await.expect("Should get html from markdown.");
    let updated_comment = comment::ssr::update_comment(
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

    Ok(())
}
