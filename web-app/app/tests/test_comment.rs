use leptos::ServerFnError;
use rand::Rng;

use app::comment::{CommentSortType, CommentWithChildren};
use app::forum;
use app::post;
use app::ranking::{SortType, Vote, VoteValue};
use app::{comment, ranking};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

fn get_vote_from_comment_num(comment_num: usize) -> Option<VoteValue> {
    match comment_num % 3 {
        0 => Some(VoteValue::Down),
        1 => None,
        _ => Some(VoteValue::Up),
    }
}

fn test_comment_with_children(
    comment_with_children: &CommentWithChildren,
    sort_type: CommentSortType,
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
        .expect("Failed to get comment number");
    let expected_vote_value = get_vote_from_comment_num(comment_num);
    if let Some(expected_vote_value) = expected_vote_value {
        let vote: Vote = comment_with_children
            .vote
            .clone()
            .expect(format!("Expected vote for comment {comment_num}").as_str());
        assert_eq!(vote.value, expected_vote_value);
        assert_eq!(vote.user_id, expected_user_id);
        assert_eq!(vote.post_id, expected_post_id);
        assert_eq!(vote.comment_id, Some(comment_with_children.comment.comment_id));
    } else {
        assert!(comment_with_children.vote.is_none());
    }

    // Test child comments
    let mut index = 0usize;
    for child_comment in &comment_with_children.child_comments {
        // Test that parent id is correct
        assert!(child_comment.comment.parent_id.is_some());
        assert_eq!(
            child_comment.comment.parent_id.unwrap(),
            comment_with_children.comment.comment_id
        );
        // Test that the child comments are correctly sorted
        if index > 0 {
            let previous_child_comment = &comment_with_children.child_comments.get(index - 1);
            assert!(previous_child_comment.is_some());
            let previous_child_comment = previous_child_comment.unwrap();
            assert!(match sort_type {
                CommentSortType::Best =>
                    child_comment.comment.score <= previous_child_comment.comment.score,
                CommentSortType::Recent =>
                    child_comment.comment.create_timestamp
                        <= previous_child_comment.comment.create_timestamp,
            });
        }
        index += 1;
        test_comment_with_children(child_comment, sort_type, expected_user_id, expected_post_id);
    }
}

#[tokio::test]
async fn test_comment_tree() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        test_user.user_id,
        db_pool.clone(),
    ).await?;

    let post = post::ssr::create_post(
        forum_name,
        "post",
        "body",
        None,
        false,
        None,
        &test_user,
        db_pool.clone(),
    ).await?;

    let mut rng = rand::thread_rng();
    let mut comment_id_vec = Vec::<i64>::new();

    for i in 1..21 {
        let parent_id = comment_id_vec.get(i % 5);

        let comment = comment::ssr::create_comment(
            post.post_id,
            parent_id.cloned(),
            i.to_string().as_str(),
            None,
            &test_user,
            db_pool.clone(),
        ).await?;

        comment_id_vec.push(comment.comment_id);

        set_comment_score(
            comment.comment_id,
            rng.gen_range(-100..101),
            db_pool.clone(),
        ).await?;

        let vote_value = get_vote_from_comment_num(i);

        if vote_value.is_some() {
            ranking::ssr::vote_on_content(
                vote_value.unwrap(),
                post.post_id,
                Some(comment.comment_id),
                None,
                &test_user,
                db_pool.clone(),
            ).await?;
        }
    }

    let comment_sort_type_array = [CommentSortType::Best, CommentSortType::Recent];

    for sort_type in comment_sort_type_array {
        let comment_tree = comment::ssr::get_post_comment_tree(
            post.post_id,
            SortType::Comment(sort_type),
            Some(test_user.user_id),
            db_pool.clone(),
        ).await?;

        for comment in comment_tree {
            // Check that root comments don't have a parent id
            assert!(comment.comment.parent_id.is_none());
            // Call recursive function to check the rest of the tree
            test_comment_with_children(&comment, sort_type, test_user.user_id, post.post_id);
        }
    }

    Ok(())
}
