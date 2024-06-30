use leptos::ServerFnError;
use rand::Rng;

use app::{forum, post, ranking};
use app::editor::get_styled_html_from_markdown;
use app::post::{Post, PostSortType, ssr};
use app::ranking::{SortType, VoteInfo, VoteValue};

pub use crate::common::*;
pub use crate::data_factory::*;

mod common;
mod data_factory;

pub fn test_post_vec(
    post_vec: &Vec<Post>,
    expected_post_vec: &Vec<Post>,
    sort_type: PostSortType,
    expected_user_id: i64,
) {
    let mut index = 0usize;
    for post in post_vec {
        assert_eq!(post.creator_id, expected_user_id);
        assert!(expected_post_vec.contains(post));
        if index > 0 {
            let previous_post = &post_vec[index - 1];
            assert!(match sort_type {
                PostSortType::Hot => post.recommended_score <= previous_post.recommended_score,
                PostSortType::Trending => post.trending_score <= previous_post.trending_score,
                PostSortType::Best => post.score <= previous_post.score,
                PostSortType::Recent => post.create_timestamp <= previous_post.create_timestamp,
            });
        }
        index += 1;
    }
}

#[tokio::test]
async fn test_get_subscribed_post_vec() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum1_name = "1";
    let forum2_name = "2";
    let num_post = 10usize;
    let mut expected_post_vec = Vec::<Post>::new();

    let (forum1, mut expected_forum1_post_vec) = create_forum_with_posts(
        forum1_name,
        10,
        Some((0..10).map(|i| i).collect()),
        &test_user,
        db_pool.clone(),
    ).await?;
    expected_post_vec.append(&mut expected_forum1_post_vec);

    create_forum_with_posts(
        forum2_name,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_vec = ssr::get_subscribed_post_vec(
        test_user.user_id,
        SortType::Post(PostSortType::Hot),
        num_post as i64,
        0,
        db_pool.clone(),
    ).await?;
    assert!(post_vec.is_empty());

    forum::ssr::subscribe(forum1.forum_id, test_user.user_id, db_pool.clone()).await?;

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_subscribed_post_vec(
            test_user.user_id,
            SortType::Post(sort_type),
            num_post as i64,
            0,
            db_pool.clone(),
        ).await?;
        test_post_vec(&post_vec, &expected_post_vec, sort_type, test_user.user_id);
    }

    forum::ssr::unsubscribe(forum1.forum_id, test_user.user_id, db_pool.clone()).await?;
    let post_vec = ssr::get_subscribed_post_vec(
        test_user.user_id,
        SortType::Post(PostSortType::Hot),
        num_post as i64,
        0,
        db_pool.clone(),
    ).await?;
    assert!(post_vec.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_sorted_post_vec() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum1_name = "1";
    let forum2_name = "2";
    let num_post = 10usize;
    let mut expected_post_vec = Vec::<Post>::new();

    let (_, mut expected_forum1_post_vec) = create_forum_with_posts(
        forum1_name,
        10,
        Some((0..10).map(|i| i).collect()),
        &test_user,
        db_pool.clone(),
    ).await?;
    expected_post_vec.append(&mut expected_forum1_post_vec);

    let (_, mut expected_forum2_post_vec) = create_forum_with_posts(
        forum2_name,
        num_post,
        Some((0..num_post).map(|i| i as i32).collect()),
        &test_user,
        db_pool.clone(),
    ).await?;
    expected_post_vec.append(&mut expected_forum2_post_vec);

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_sorted_post_vec(SortType::Post(sort_type), num_post as i64, 0, db_pool.clone()).await?;
        test_post_vec(&post_vec, &expected_post_vec, sort_type, test_user.user_id);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_post_vec_by_forum_name() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    let num_posts = 20usize;
    let mut expected_post_vec = Vec::<Post>::with_capacity(num_posts);

    let (_, mut expected_forum_post_vec) = create_forum_with_posts(
        forum_name,
        num_posts,
        Some((0..num_posts).map(|i| (i as i32) / 2).collect()),
        &test_user,
        db_pool.clone(),
    ).await?;

    expected_post_vec.append(&mut expected_forum_post_vec);

    let post_sort_type_array = [
        PostSortType::Hot,
        PostSortType::Trending,
        PostSortType::Best,
        PostSortType::Recent,
    ];

    for sort_type in post_sort_type_array {
        let post_vec = ssr::get_post_vec_by_forum_name(
            forum_name,
            SortType::Post(sort_type),
            num_posts as i64,
            0,
            db_pool.clone(),
        ).await?;

        test_post_vec(&post_vec, &expected_post_vec, sort_type, test_user.user_id);
    }

    let partial_load_num_post = num_posts / 2;

    let post_vec = ssr::get_post_vec_by_forum_name(
        forum_name,
        SortType::Post(PostSortType::Hot),
        partial_load_num_post as i64,
        0,
        db_pool.clone(),
    ).await?;

    assert_eq!(post_vec.len(), partial_load_num_post);

    Ok(())
}

#[tokio::test]
async fn test_update_post() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
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

    let updated_title = "updated post";
    let updated_markdown_body = "# Here is a post with markdown";
    let updated_html_body = get_styled_html_from_markdown(String::from(updated_markdown_body)).await?;
    let updated_post = post::ssr::update_post(
        post.post_id,
        updated_title,
        &updated_html_body,
        Some(updated_markdown_body),
        false,
        None,&test_user,
        db_pool
    ).await?;

    assert_eq!(updated_post.title, updated_title);
    assert_eq!(updated_post.body, updated_html_body);
    assert_eq!(updated_post.markdown_body, Some(String::from(updated_markdown_body)));
    assert!(
        updated_post.edit_timestamp.is_some() &&
        updated_post.edit_timestamp.unwrap() > updated_post.create_timestamp &&
        updated_post.create_timestamp == post.create_timestamp
    );

    Ok(())
}

#[tokio::test]
async fn test_post_scores() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
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

    set_post_score(post.post_id, rng.gen_range(-100..101), db_pool.clone()).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), db_pool.clone())
            .await?;

    let post = post_with_vote.post;
    let post_num_days_old = (post
        .scoring_timestamp
        .signed_duration_since(post.create_timestamp)
        .num_seconds() as f32)
        / 86400.0;
    let expected_recommended_score =
        (post.score as f32) * f32::powf(2.0, 3.0 * (2.0 - post_num_days_old));
    let expected_trending_score =
        (post.score as f32) * f32::powf(2.0, 8.0 * (1.0 - post_num_days_old));

    assert_eq!(post.recommended_score, expected_recommended_score);
    assert_eq!(post.trending_score, expected_trending_score);
    assert_eq!(post_with_vote.vote, None);

    Ok(())
}

#[tokio::test]
async fn test_post_votes() -> Result<(), ServerFnError> {
    let db_pool = get_db_pool().await;
    let test_user = create_test_user(&db_pool).await;

    let forum_name = "forum";
    forum::ssr::create_forum(
        forum_name,
        "forum",
        false,
        &test_user,
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

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), db_pool.clone())
            .await?;
    assert!(post_with_vote.vote.is_none());

    let vote_value = VoteValue::Up;
    ranking::ssr::vote_on_content(
        vote_value,
        post.post_id,
        None,
        None,
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), db_pool.clone())
            .await?;
    assert!(post_with_vote.vote.is_some());
    let vote = post_with_vote.vote.unwrap();
    assert_eq!(vote.value, vote_value);
    assert_eq!(vote.user_id, test_user.user_id);
    assert_eq!(vote.post_id, post.post_id);
    assert_eq!(vote.comment_id, None);

    // assert error when repeating vote
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        db_pool.clone(),
    )
        .await
        .is_err());

    // assert error when existing vote is not referenced
    assert!(ranking::ssr::vote_on_content(
        VoteValue::Up,
        post.post_id,
        None,
        None,
        &test_user,
        db_pool.clone(),
    )
        .await
        .is_err());

    ranking::ssr::vote_on_content(
        VoteValue::Down,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), db_pool.clone())
            .await?;
    let vote = post_with_vote.vote.expect("Vote not found");
    assert_eq!(vote.value, VoteValue::Down);
    assert_eq!(vote.user_id, test_user.user_id);
    assert_eq!(vote.post_id, post.post_id);
    assert_eq!(vote.comment_id, None);

    ranking::ssr::vote_on_content(
        VoteValue::None,
        post.post_id,
        None,
        Some(VoteInfo {
            vote_id: vote.vote_id,
            value: vote.value,
        }),
        &test_user,
        db_pool.clone(),
    ).await?;

    let post_with_vote =
        post::ssr::get_post_with_info_by_id(post.post_id, Some(&test_user), db_pool.clone())
            .await?;
    assert!(post_with_vote.vote.is_none());

    Ok(())
}
