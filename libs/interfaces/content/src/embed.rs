use leptos::prelude::*;

#[cfg(feature = "ssr")]
use {
    http::header::{ACCEPT, USER_AGENT},
    http::{HeaderMap, HeaderValue},
    reqwest::Client,
    crate::checks::check_string_length,
    crate::constants::MAX_LINK_LENGTH,
};

#[server]
pub async fn get_oembed_data(url: String) -> Result<OEmbedReply, AppError> {
    check_string_length(&url, "Url", MAX_LINK_LENGTH as usize, false)?;
    let mut oembed_data = fetch_api::<OEmbedReply>(&url)
        .await
        .ok_or(AppError::new(format!("Cannot get oEmbed data at endpoint {url}")))?;
    
    match oembed_data.oembed_type {
        OEmbedType::Video(ref mut video) => video.html = clean_html(&video.html),
        OEmbedType::Rich(ref mut rich) => rich.html = clean_html(&rich.html),
        _ => ()
    };
    
    Ok(oembed_data)
}