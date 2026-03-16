use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use http::status::StatusCode;
use leptos::prelude::*;
use leptos::server_fn::codec::JsonEncoding;
use leptos_fluent::{move_tr};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors};
use crate::icons::{AuthErrorIcon, BannedIcon, InternalErrorIcon, InvalidRequestIcon, NetworkErrorIcon, NotAuthorizedIcon, NotFoundIcon, TooHeavyIcon};


#[component]
pub fn AppErrorIcon(
    app_error: AppError,
) -> impl IntoView {
    match app_error {
        AppError::AuthenticationError(_) => view! { <AuthErrorIcon/> }.into_any(),
        AppError::NotAuthenticated => view! { <AuthErrorIcon/> }.into_any(),
        AppError::InsufficientPrivileges => view! { <NotAuthorizedIcon/> }.into_any(),
        AppError::SphereBanUntil(_) | AppError::PermanentSphereBan | AppError::GlobalBanUntil(_) | AppError::PermanentGlobalBan => view! { <BannedIcon/> }.into_any(),
        AppError::CommunicationError(error) => match error {
            ServerFnErrorErr::Args(_) | ServerFnErrorErr::MissingArg(_) => view! { <InvalidRequestIcon/> }.into_any(),
            ServerFnErrorErr::Registration(_) | ServerFnErrorErr::Request(_) | ServerFnErrorErr::Response(_) => view! { <NetworkErrorIcon/> }.into_any(),
            _ => view! { <InternalErrorIcon/> }.into_any(),
        },
        AppError::DatabaseError(_) => view! { <InternalErrorIcon/> }.into_any(),
        AppError::InternalServerError(_) => view! { <InternalErrorIcon/> }.into_any(),
        AppError::NotFound => view! { <NotFoundIcon/> }.into_any(),
        AppError::PayloadTooLarge(_) => view! { <TooHeavyIcon/> }.into_any(),
    }
}

/// Displays an error
#[component]
pub fn ErrorDisplay(
    error: AppError
) -> impl IntoView {
    let error_string = error.to_string();
    let status_code =  error.status_code().as_u16();
    let user_message = error.user_message();

    log::error!("Caught error, status_code: {status_code}, error message: {error_string}");
    view! {
        <div class="w-full flex items-center gap-2 justify-center">
            <AppErrorIcon app_error=error/>
            <div class="flex flex-col">
                <h2 class="text-2xl">{status_code}</h2>
                <h3 class="text-xl">{user_message}</h3>
            </div>
        </div>
    }.into_any()
}

/// Displays an error with its detailed message
#[component]
pub fn ErrorDetail(
    error: AppError
) -> impl IntoView {
    let error_string = error.to_string();
    let status_code = error.status_code().as_u16();
    let error_detail = error.error_detail();

    log::error!("Caught error, status_code: {status_code}, error message: {error_string}");
    view! {
        <div class="w-full flex items-center gap-2 justify-center">
            <AppErrorIcon app_error=error/>
            <div>{error_detail}</div>
        </div>
    }.into_any()
}