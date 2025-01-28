#![allow(dead_code)]

use std::cmp::Ordering;
use std::convert::Infallible;
use std::iter::zip;
use app::comment::{Comment, CommentSortType, CommentWithChildren, CommentWithContext};
use app::errors::AppError;
use app::ranking::{Vote, VoteValue};
use bytes::Bytes;
use float_cmp::approx_eq;
use futures_util::stream::once;
use multer::Multipart;
use server_fn::codec::MultipartData;
use sqlx::PgPool;
use app::post::{Post, PostSortType, PostWithSphereInfo};

pub const POST_SORT_TYPE_ARRAY: [PostSortType; 4] = [
    PostSortType::Hot,
    PostSortType::Trending,
    PostSortType::Best,
    PostSortType::Recent,
];

pub const COMMENT_SORT_TYPE_ARRAY: [CommentSortType; 2] = [
    CommentSortType::Best,
    CommentSortType::Recent,
];

pub fn sort_post_vec(
    post_vec: &mut [PostWithSphereInfo],
    sort_type: PostSortType,
    consider_pinned: bool,
) {
    post_vec.sort_by(|l, r| {
        match (consider_pinned, l.post.is_pinned, r.post.is_pinned) {
            (true, true, false) => Ordering::Less,
            (true, false, true) => Ordering::Greater,
            _ => match sort_type {
                PostSortType::Hot => r.post.recommended_score.partial_cmp(&l.post.recommended_score).unwrap(),
                PostSortType::Trending => r.post.trending_score.partial_cmp(&l.post.trending_score).unwrap(),
                PostSortType::Best => r.post.score.partial_cmp(&l.post.score).unwrap(),
                PostSortType::Recent => r.post.create_timestamp.partial_cmp(&l.post.create_timestamp).unwrap(),

            }
        }
    });
}

pub fn test_post_vec(
    post_vec: &[PostWithSphereInfo],
    expected_post_vec: &[PostWithSphereInfo],
) {
    assert_eq!(post_vec.len(), expected_post_vec.len());
    for (i, (post, expected_post)) in zip(post_vec, expected_post_vec).enumerate() {
        println!("post#{}", i);
        assert_eq!(post, expected_post);
    }
}

pub fn test_post_score(post: &Post) {
    let second_delta = post
        .scoring_timestamp
        .signed_duration_since(post.create_timestamp)
        .num_milliseconds();
    let num_days_old = (post
        .scoring_timestamp
        .signed_duration_since(post.create_timestamp)
        .num_milliseconds() as f64)
        / 86400000.0;

    println!(
        "Scoring timestamp: {}, create timestamp: {}, second delta: {second_delta}, num_days_old: {num_days_old}",
        post.scoring_timestamp,
        post.create_timestamp,
    );

    let expected_recommended_score = (post.score as f64) * f64::powf(2.0, 3.0 * (2.0 - num_days_old));
    let expected_trending_score = (post.score as f64) * f64::powf(2.0, 8.0 * (1.0 - num_days_old));

    println!("Recommended: {}, expected: {}", post.recommended_score, expected_recommended_score);
    assert!(approx_eq!(f32, post.recommended_score, expected_recommended_score as f32, epsilon = f32::EPSILON, ulps = 5));
    println!("Trending: {}, expected: {}", post.trending_score, expected_trending_score);
    assert!(approx_eq!(f32, post.trending_score, expected_trending_score as f32, epsilon = f32::EPSILON, ulps = 5));
}

pub fn get_vote_from_comment_num(comment_num: usize) -> Option<VoteValue> {
    match comment_num % 3 {
        0 => Some(VoteValue::Down),
        1 => None,
        _ => Some(VoteValue::Up),
    }
}

pub fn sort_comment_vec(
    comment_vec: &mut [CommentWithContext],
    sort_type: CommentSortType,
    consider_pinned: bool,
) {
    comment_vec.sort_by(|l, r| {
        match (consider_pinned, l.comment.is_pinned, r.comment.is_pinned) {
            (true, true, false) => Ordering::Less,
            (true, false, true) => Ordering::Greater,
            _ => match sort_type {
                CommentSortType::Best => r.comment.score.partial_cmp(&l.comment.score).unwrap(),
                CommentSortType::Recent => r.comment.create_timestamp.partial_cmp(&l.comment.create_timestamp).unwrap(),
            }
        }
    });
}

pub fn sort_comment_tree(
    comment_vec: &mut [CommentWithChildren],
    sort_type: CommentSortType,
    consider_pinned: bool,
) {
    comment_vec.sort_by(|l, r| {
        match (consider_pinned, l.comment.is_pinned, r.comment.is_pinned) {
            (true, true, false) => Ordering::Less,
            (true, false, true) => Ordering::Greater,
            _ => match sort_type {
                CommentSortType::Best => r.comment.score.partial_cmp(&l.comment.score).unwrap(),
                CommentSortType::Recent => r.comment.create_timestamp.partial_cmp(&l.comment.create_timestamp).unwrap(),
            }
        }
    });

    for comment in comment_vec.iter_mut() {
        sort_comment_tree(&mut comment.child_comments, sort_type, consider_pinned);
    }
}

pub fn test_comment_vec(
    comment_vec: &[CommentWithContext],
    expected_comment_vec: &[CommentWithContext],
) {
    assert_eq!(comment_vec.len(), expected_comment_vec.len());
    for (comment, expected_comment) in zip(comment_vec, expected_comment_vec) {
        assert_eq!(comment, expected_comment);
    }
}

pub fn test_comment_tree(
    comment_vec: &[CommentWithChildren],
    expected_comment_vec: &[CommentWithChildren],
) {
    assert_eq!(comment_vec.len(), expected_comment_vec.len());
    for (comment, expected_comment) in zip(comment_vec, expected_comment_vec) {
        assert_eq!(comment, expected_comment);
        test_comment_tree(&comment.child_comments, &expected_comment.child_comments);
    }
}

pub async fn get_comment_by_id(
    comment_id: i64,
    db_pool: &PgPool,
) -> Result<Comment, AppError> {
    let comment = sqlx::query_as!(
            Comment,
            "SELECT *
            FROM comments
            WHERE comment_id = $1",
            comment_id
        )
        .fetch_one(db_pool)
        .await?;

    Ok(comment)
}

pub async fn get_user_comment_vote(
    comment: &Comment,
    user_id: i64,
    db_pool: &PgPool,
) -> Result<Vote, AppError> {
    let vote = sqlx::query_as!(
            Vote,
            "SELECT *
            FROM votes
            WHERE
                post_id = $1 AND
                comment_id = $2 AND
                user_id = $3",
            comment.post_id,
            comment.comment_id,
            user_id,
        )
        .fetch_one(db_pool)
        .await?;

    Ok(vote)
}

pub fn get_png_data() -> &'static[u8] {
    &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR length
        0x49, 0x48, 0x44, 0x52, // IHDR chunk type
        0x00, 0x00, 0x00, 0x01, // width (1 pixel)
        0x00, 0x00, 0x00, 0x01, // height (1 pixel)
        0x08,                   // bit depth
        0x06,                   // color type (RGBA)
        0x00,                   // compression method
        0x00,                   // filter method
        0x00,                   // interlace method
        0xDE, 0xAD, 0xBE, 0xEF, // IHDR CRC (computed)
        0x00, 0x00, 0x00, 0x0A, // IDAT length
        0x49, 0x44, 0x41, 0x54, // IDAT chunk type
        0x08, 0x1D, 0x01, 0x00, // compressed pixel data (1 pixel, fully opaque)
        0x00, 0x00, 0x00,       // end of IDAT
        0x7D, 0xA8, 0xF8, 0x0C, // IDAT CRC (computed)
        0x00, 0x00, 0x00, 0x00, // IEND length
        0x49, 0x45, 0x4E, 0x44, // IEND chunk type
        0xAE, 0x42, 0x60, 0x82, // IEND CRC (computed)
    ]
}

pub async fn get_multipart_string(
    string_field_name: &str,
    string_value: &str,
) -> MultipartData {
    let boundary = "boundary-test";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"{string_field_name}\"\r\n\r\n\
         {string_value}\r\n\
         --{boundary}--\r\n"
    );

    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) });
    let multipart = Multipart::new(stream, boundary);
    MultipartData::Server(multipart)
}

pub async fn get_multipart_image(
    image_field_name: &str,
) -> MultipartData {
    let mut body = Vec::new();
    let boundary = "boundary-test";

    body.extend_from_slice(format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"{image_field_name}\"; filename=\"test.png\"\r\n\
         Content-Type: image/png\r\n\r\n"
    ).as_bytes());
    body.extend_from_slice(get_png_data()); // PNG magic bytes
    body.extend_from_slice(
        format!("\r\n--{boundary}--\r\n").as_bytes(),
    );

    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) });
    let multipart = Multipart::new(stream, boundary);
    MultipartData::Server(multipart)
}

pub async fn get_multipart_pdf_with_string(
    pdf_field_name: &str,
    string_field_name: &str,
    string_value: &str,
) -> MultipartData {
    let boundary = "boundary-test";

    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"{string_field_name}\"\r\n\r\n\
         {string_value}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"{pdf_field_name}\"; filename=\"test.pdf\"\r\n\
         Content-Type: application/pdf\r\n\r\n\
         %PDF-1.4\r\n\
         --{boundary}--\r\n"
    );

    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) });
    let multipart = Multipart::new(stream, boundary);
    MultipartData::Server(multipart)
}

pub async fn get_multipart_image_with_string(
    image_field_name: &str,
    string_field_name: &str,
    string_value: &str,
) -> MultipartData {
    let mut body = Vec::new();
    let boundary = "boundary-test";

    body.extend_from_slice(format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"{string_field_name}\"\r\n\r\n\
         {string_value}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"{image_field_name}\"; filename=\"test.png\"\r\n\
         Content-Type: image/png\r\n\r\n"
    ).as_bytes());
    body.extend_from_slice(get_png_data()); // PNG magic bytes
    body.extend_from_slice(
        format!("\r\n--{boundary}--\r\n").as_bytes(),
    );
    
    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) });
    let multipart = Multipart::new(stream, boundary);
    MultipartData::Server(multipart)
}

pub async fn get_invalid_multipart_image_with_string(
    image_field_name: &str,
    string_field_name: &str,
    string_value: &str,
) -> MultipartData {
    let mut body = Vec::new();
    let boundary = "boundary-test";

    body.extend_from_slice(format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"{string_field_name}\"\r\n\r\n\
         {string_value}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"{image_field_name}\"; filename=\"test.png\"\r\n\
         Content-Type: image/png\r\n\r\n"
    ).as_bytes());
    body.extend_from_slice(b"invalid png data."); // PNG magic bytes
    body.extend_from_slice(
        format!("\r\n--{boundary}--\r\n").as_bytes(),
    );

    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) });
    let multipart = Multipart::new(stream, boundary);
    MultipartData::Server(multipart)
}