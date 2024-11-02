#![allow(dead_code)]

use std::convert::Infallible;

use app::comment::Comment;
use app::errors::AppError;
use app::ranking::Vote;
use bytes::Bytes;
use futures_util::stream::once;
use multer::Multipart;
use server_fn::codec::MultipartData;
use sqlx::PgPool;

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
    //let body = format!(
    //    "--{boundary}\r\n
    //     Content-Disposition: form-data; name=\"{string_field_name}\"\r\n\r\n
    //     {string_value}\r\n
    //     --{boundary}\r\n
    //     Content-Disposition: form-data; name=\"{image_field_name}\"; filename=\"test.png\"\r\n
    //     Content-Type: image/png\r\n\r\n
    //     {png_data}\r\n
    //     --{boundary}--\r\n",
    //);
    
    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) });
    let multipart = Multipart::new(stream, boundary);
    MultipartData::Server(multipart)
}