use std::env;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::{IntoResponse, Response as AxumResponse},
};
use axum::http::{header, HeaderValue};
use leptos::prelude::{Errors, LeptosOptions};
use leptos::view;
use tower::util::ServiceExt;
use tower_http::services::ServeDir;

use sharesphere_utils::error_template::ErrorTemplate;
use sharesphere_utils::errors::AppError;

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let mut errors = Errors::default();
        errors.insert_with_default_key(AppError::NotFound);
        let handler = leptos_axum::render_app_to_stream(
            move || view! {<ErrorTemplate outside_errors=errors.clone()/>},
        );
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    
    let mut response = ServeDir::new(root)
        .oneshot(req)
        .await
        .unwrap_or_else(|err| match err {})
        .into_response();

    if env::var("LEPTOS_ENV").is_ok_and(|leptos_env| leptos_env == "PROD" ) {
        response.headers_mut().append(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
    }
    
    Ok(response)
}
