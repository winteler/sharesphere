use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use http::status::StatusCode;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos::server_fn::codec::JsonEncoding;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::icons::{AuthErrorIcon, BannedIcon, InternalErrorIcon, InvalidRequestIcon, NetworkErrorIcon, NotAuthorizedIcon, NotFoundIcon, TooHeavyIcon};

const NOT_AUTHENTICATED_MESSAGE: &str = "Please authenticate yourself.";
const AUTH_FAILED_MESSAGE: &str = "Sorry, we had some trouble authenticating you.";
const INTERNAL_ERROR_MESSAGE: &str = "Something went wrong.";
const NOT_AUTHORIZED_MESSAGE: &str = "You're in a restricted area, please do not resist.";
const SPHERE_BAN_UNTIL_MESSAGE: &str = "You are banned from this sphere until";
const PERMANENT_SPHERE_BAN_MESSAGE: &str = "You are permanently banned from this website.";
const GLOBAL_BAN_UNTIL_MESSAGE: &str = "You are globally banned until";
const PERMANENT_GLOBAL_BAN_MESSAGE: &str = "You are permanently banned from this website.";
const BAD_REQUEST_MESSAGE: &str = "Sorry, we didn't understand your request.";
const UNAVAILABLE_MESSAGE: &str = "Sorry, we've got noise on the line.";
const NOT_FOUND_MESSAGE: &str = "There's nothing here";

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppError {
    AuthenticationError(String),
    NotAuthenticated,
    InsufficientPrivileges,
    SphereBanUntil(chrono::DateTime<chrono::Utc>),
    PermanentSphereBan,
    GlobalBanUntil(chrono::DateTime<chrono::Utc>),
    PermanentGlobalBan,
    CommunicationError(ServerFnErrorErr),
    DatabaseError(String),
    InternalServerError(String),
    NotFound,
    PayloadTooLarge(usize),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::AuthenticationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotAuthenticated | AppError::InsufficientPrivileges | AppError::SphereBanUntil(_) |
            AppError::PermanentSphereBan | AppError::GlobalBanUntil(_) | AppError::PermanentGlobalBan => StatusCode::FORBIDDEN,
            AppError::CommunicationError(error) => match error {
                ServerFnErrorErr::Args(_) | ServerFnErrorErr::MissingArg(_) | ServerFnErrorErr::Serialization(_) | ServerFnErrorErr::Deserialization(_) => StatusCode::BAD_REQUEST,
                ServerFnErrorErr::Registration(_) | ServerFnErrorErr::Request(_) | ServerFnErrorErr::Response(_) => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::AuthenticationError(_) => String::from(AUTH_FAILED_MESSAGE),
            AppError::NotAuthenticated => String::from(NOT_AUTHENTICATED_MESSAGE),
            AppError::InsufficientPrivileges => String::from(NOT_AUTHORIZED_MESSAGE),
            AppError::SphereBanUntil(timestamp) => format!("{} {}", SPHERE_BAN_UNTIL_MESSAGE, timestamp),
            AppError::PermanentSphereBan => String::from(PERMANENT_SPHERE_BAN_MESSAGE),
            AppError::GlobalBanUntil(timestamp) => format!("{} {}", GLOBAL_BAN_UNTIL_MESSAGE, timestamp),
            AppError::PermanentGlobalBan => String::from(PERMANENT_GLOBAL_BAN_MESSAGE),
            AppError::CommunicationError(error) => match error {
                ServerFnErrorErr::Args(_) | ServerFnErrorErr::MissingArg(_) |
                ServerFnErrorErr::Serialization(_) | ServerFnErrorErr::Deserialization(_) => String::from(BAD_REQUEST_MESSAGE),
                ServerFnErrorErr::Registration(_) | ServerFnErrorErr::Request(_) | ServerFnErrorErr::Response(_) => String::from(UNAVAILABLE_MESSAGE),
                _ => String::from(INTERNAL_ERROR_MESSAGE),
            },
            AppError::DatabaseError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::InternalServerError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::NotFound => String::from(NOT_FOUND_MESSAGE),
            AppError::PayloadTooLarge(mb_limit) => format!("Payload exceeds the {mb_limit} Bytes limit."),
        }
    }

    pub fn error_detail(&self) -> String {
        match self {
            AppError::AuthenticationError(e) => e.clone(),
            AppError::NotAuthenticated => String::from(NOT_AUTHENTICATED_MESSAGE),
            AppError::InsufficientPrivileges => String::from("Insufficient privileges"),
            AppError::SphereBanUntil(timestamp) => format!("{} {}", SPHERE_BAN_UNTIL_MESSAGE, timestamp),
            AppError::PermanentSphereBan => String::from(PERMANENT_SPHERE_BAN_MESSAGE),
            AppError::GlobalBanUntil(timestamp) => format!("{} {}", GLOBAL_BAN_UNTIL_MESSAGE, timestamp),
            AppError::PermanentGlobalBan => String::from(PERMANENT_GLOBAL_BAN_MESSAGE),
            AppError::CommunicationError(error) => match error {
                ServerFnErrorErr::Args(e) | ServerFnErrorErr::MissingArg(e) |
                ServerFnErrorErr::Serialization(e) | ServerFnErrorErr::Deserialization(e) => e.clone(),
                ServerFnErrorErr::Registration(e) | ServerFnErrorErr::Request(e) | ServerFnErrorErr::Response(e) => e.clone(),
                _ => String::from(INTERNAL_ERROR_MESSAGE),
            },
            AppError::DatabaseError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::InternalServerError(e) => e.clone(),
            AppError::NotFound => String::from(NOT_FOUND_MESSAGE),
            AppError::PayloadTooLarge(mb_limit) => format!("Payload exceeds the {mb_limit} Bytes limit."),
        }
    }

    /// Constructs a new [`AppError::InternalServerError`] from some other type.
    pub fn new(msg: impl ToString) -> Self {
        Self::InternalServerError(msg.to_string())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}

impl FromStr for AppError {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(error: ServerFnErrorErr) -> Self {
        match error {
            ServerFnErrorErr::ServerError(message) => serde_json::from_str(message.as_str()).unwrap_or(AppError::InternalServerError(message.clone())),
            _ => AppError::CommunicationError(error),
        }
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use sqlx;
    use std::io::Error;
    use openidconnect::SignatureVerificationError;
    use crate::errors::AppError;

    impl From<sqlx::Error> for AppError {
        fn from(error: sqlx::Error) -> Self {
            match error {
                sqlx::Error::RowNotFound => AppError::NotFound,
                _ => AppError::DatabaseError(error.to_string()),
            }
        }
    }

    impl From<std::env::VarError> for AppError {
        fn from(error: std::env::VarError) -> Self {
            AppError::InternalServerError(error.to_string())
        }
    }

    impl From<std::string::FromUtf8Error> for AppError {
        fn from(error: std::string::FromUtf8Error) -> Self {
            AppError::InternalServerError(error.to_string())
        }
    }

    impl From<url::ParseError> for AppError {
        fn from(error: url::ParseError) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<openidconnect::ClaimsVerificationError> for AppError {
        fn from(error: openidconnect::ClaimsVerificationError) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<openidconnect::ConfigurationError> for AppError {
        fn from(error: openidconnect::ConfigurationError) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<openidconnect::SigningError> for AppError {
        fn from(error: openidconnect::SigningError) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<SignatureVerificationError> for AppError {
        fn from(value: SignatureVerificationError) -> Self {
            AppError::AuthenticationError(value.to_string())
        }
    }

    impl<T: std::error::Error> From<openidconnect::DiscoveryError<T>> for AppError {
        fn from(error: openidconnect::DiscoveryError<T>) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl<A: std::error::Error, B: openidconnect::ErrorResponse> From<openidconnect::RequestTokenError<A, B>> for AppError {
        fn from(error: openidconnect::RequestTokenError<A, B>) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<quick_xml::Error> for AppError {
        fn from(error: quick_xml::Error) -> Self {
            AppError::InternalServerError(error.to_string())
        }
    }

    impl From<reqwest::Error> for AppError {
        fn from(value: reqwest::Error) -> Self {
            AppError::InternalServerError(value.to_string())
        }
    }

    impl From<Error> for AppError {
        fn from(value: Error) -> Self {
            AppError::InternalServerError(value.to_string())
        }
    }
}

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

#[cfg(test)]
mod tests {
    use crate::errors::{AppError, AUTH_FAILED_MESSAGE, BAD_REQUEST_MESSAGE, GLOBAL_BAN_UNTIL_MESSAGE, INTERNAL_ERROR_MESSAGE, NOT_AUTHENTICATED_MESSAGE, NOT_AUTHORIZED_MESSAGE, NOT_FOUND_MESSAGE, PERMANENT_GLOBAL_BAN_MESSAGE, PERMANENT_SPHERE_BAN_MESSAGE, SPHERE_BAN_UNTIL_MESSAGE, UNAVAILABLE_MESSAGE};
    use http::StatusCode;
    use leptos::prelude::{ServerFnErrorErr};
    use quick_xml::errors::SyntaxError;
    use std::str::FromStr;

    #[test]
    fn test_app_error_status_code() {
        let test_string = String::from("test");
        let test_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let server_fn_error = ServerFnErrorErr::ServerError(String::from("test"));
        let args_error = ServerFnErrorErr::Args(String::from("test"));
        let missing_arg_error = ServerFnErrorErr::MissingArg(String::from("test"));
        let request_error = ServerFnErrorErr::Request(String::from("test"));
        let response_error = ServerFnErrorErr::Response(String::from("test"));
        let registration_error = ServerFnErrorErr::Registration(String::from("test"));
        let serialization_error = ServerFnErrorErr::Serialization(String::from("test"));
        let deserialization_error = ServerFnErrorErr::Deserialization(String::from("test"));
        assert_eq!(AppError::AuthenticationError(test_string.clone()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::NotAuthenticated.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::InsufficientPrivileges.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::SphereBanUntil(test_timestamp).status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::PermanentSphereBan.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::GlobalBanUntil(test_timestamp).status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::PermanentGlobalBan.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::CommunicationError(server_fn_error).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::CommunicationError(args_error).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::CommunicationError(missing_arg_error).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::CommunicationError(serialization_error).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::CommunicationError(deserialization_error).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::CommunicationError(request_error).status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(AppError::CommunicationError(response_error).status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(AppError::CommunicationError(registration_error).status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(AppError::DatabaseError(test_string.clone()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::InternalServerError(test_string.clone()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::NotFound.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_app_error_user_message() {
        let test_string = String::from("test");
        let test_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let server_fn_error = ServerFnErrorErr::ServerError(String::from("test"));
        let args_error = ServerFnErrorErr::Args(String::from("test"));
        let missing_arg_error = ServerFnErrorErr::MissingArg(String::from("test"));
        let request_error = ServerFnErrorErr::Request(String::from("test"));
        let response_error = ServerFnErrorErr::Response(String::from("test"));
        let registration_error = ServerFnErrorErr::Registration(String::from("test"));
        let serialization_error = ServerFnErrorErr::Serialization(String::from("test"));
        let deserialization_error = ServerFnErrorErr::Deserialization(String::from("test"));
        assert_eq!(AppError::AuthenticationError(test_string.clone()).user_message(), String::from(AUTH_FAILED_MESSAGE));
        assert_eq!(AppError::NotAuthenticated.user_message(), String::from(NOT_AUTHENTICATED_MESSAGE));
        assert_eq!(AppError::InsufficientPrivileges.user_message(), String::from(NOT_AUTHORIZED_MESSAGE));
        assert_eq!(AppError::SphereBanUntil(test_timestamp).user_message(), format!("{} {}", SPHERE_BAN_UNTIL_MESSAGE, test_timestamp.clone().to_string()));
        assert_eq!(AppError::PermanentSphereBan.user_message(), String::from(PERMANENT_SPHERE_BAN_MESSAGE));
        assert_eq!(AppError::GlobalBanUntil(test_timestamp).user_message(), format!("{} {}", GLOBAL_BAN_UNTIL_MESSAGE, test_timestamp.to_string()));
        assert_eq!(AppError::PermanentGlobalBan.user_message(), String::from(PERMANENT_GLOBAL_BAN_MESSAGE));
        assert_eq!(AppError::CommunicationError(server_fn_error).user_message(), String::from(INTERNAL_ERROR_MESSAGE));
        assert_eq!(AppError::CommunicationError(args_error).user_message(), String::from(BAD_REQUEST_MESSAGE));
        assert_eq!(AppError::CommunicationError(missing_arg_error).user_message(), String::from(BAD_REQUEST_MESSAGE));
        assert_eq!(AppError::CommunicationError(serialization_error).user_message(), String::from(BAD_REQUEST_MESSAGE));
        assert_eq!(AppError::CommunicationError(deserialization_error).user_message(), String::from(BAD_REQUEST_MESSAGE));
        assert_eq!(AppError::CommunicationError(request_error).user_message(), String::from(UNAVAILABLE_MESSAGE));
        assert_eq!(AppError::CommunicationError(response_error).user_message(), String::from(UNAVAILABLE_MESSAGE));
        assert_eq!(AppError::CommunicationError(registration_error).user_message(), String::from(UNAVAILABLE_MESSAGE));
        assert_eq!(AppError::DatabaseError(test_string.clone()).user_message(), String::from(INTERNAL_ERROR_MESSAGE));
        assert_eq!(AppError::InternalServerError(test_string.clone()).user_message(), String::from(INTERNAL_ERROR_MESSAGE));
        assert_eq!(AppError::NotFound.user_message(), String::from(NOT_FOUND_MESSAGE));
    }

    #[test]
    fn test_app_error_new() {
        let test_str = "test";
        assert_eq!(AppError::new(test_str), AppError::InternalServerError(String::from(test_str)));
    }

    #[test]
    fn test_app_error_display_and_from_string() {
        let test_string = String::from("test");
        let test_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let server_fn_error = ServerFnErrorErr::ServerError(String::from("test"));
        let server_fn_error_2 = ServerFnErrorErr::MissingArg(test_string.clone());
        assert_eq!(
            AppError::from_str(AppError::AuthenticationError(test_string.clone()).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::AuthenticationError(test_string.clone())
        );
        assert_eq!(
            AppError::from_str(AppError::NotAuthenticated.to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::NotAuthenticated
        );
        assert_eq!(
            AppError::from_str(AppError::InsufficientPrivileges.to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::InsufficientPrivileges
        );
        assert_eq!(
            AppError::from_str(AppError::SphereBanUntil(test_timestamp).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::SphereBanUntil(test_timestamp)
        );
        assert_eq!(
            AppError::from_str(AppError::PermanentSphereBan.to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::PermanentSphereBan
        );
        assert_eq!(
            AppError::from_str(AppError::GlobalBanUntil(test_timestamp).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::GlobalBanUntil(test_timestamp)
        );
        assert_eq!(
            AppError::from_str(AppError::PermanentGlobalBan.to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::PermanentGlobalBan
        );
        assert_eq!(
            AppError::from_str(AppError::CommunicationError(server_fn_error.clone()).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::CommunicationError(server_fn_error)
        );
        assert_eq!(
            AppError::from_str(AppError::CommunicationError(server_fn_error_2.clone()).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::CommunicationError(server_fn_error_2)
        );
        assert_eq!(
            AppError::from_str(AppError::DatabaseError(test_string.clone()).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::DatabaseError(test_string.clone())
        );
        assert_eq!(
            AppError::from_str(AppError::InternalServerError(test_string.clone()).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::InternalServerError(test_string.clone())
        );
        assert_eq!(
            AppError::from_str(AppError::NotFound.to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::NotFound
        );
        assert!(AppError::from_str("invalid").is_err());
    }

    #[test]
    fn test_app_error_from_sqlx_error() {
        let error_string = String::from("test");
        assert_eq!(AppError::from(sqlx::Error::RowNotFound), AppError::NotFound);
        assert_eq!(AppError::from(sqlx::Error::PoolTimedOut), AppError::DatabaseError(sqlx::Error::PoolTimedOut.to_string()));
        assert_eq!(AppError::from(sqlx::Error::ColumnNotFound(error_string.clone())), AppError::DatabaseError(sqlx::Error::ColumnNotFound(error_string).to_string()));
    }

    #[test]
    fn test_app_error_from_env_var_error() {
        let env_var_error = std::env::var("not_existing");
        assert!(env_var_error.is_err());
        let env_var_error =  env_var_error.unwrap_err();
        assert_eq!(AppError::from(env_var_error.clone()), AppError::InternalServerError(env_var_error.to_string()));
    }

    #[test]
    fn test_app_error_from_string_utf8_error() {
        // some invalid bytes, in a vector
        let invalid_bytes = vec![0, 159, 146, 150];
        let error = String::from_utf8(invalid_bytes);
        assert!(error.is_err());
        let error =  error.unwrap_err();
        assert_eq!(AppError::from(error.clone()), AppError::InternalServerError(error.to_string()));
    }

    #[test]
    fn test_app_error_from_openidconnect_url_parse_error() {
        let error = url::ParseError::InvalidDomainCharacter;
        assert_eq!(AppError::from(error), AppError::AuthenticationError(error.to_string()));
    }

    #[test]
    fn test_app_error_from_openidconnect_discovery_error() {
        // as this is a generic error type, we need to provide it with a type implementing error
        assert_eq!(
            AppError::from(openidconnect::DiscoveryError::<openidconnect::ConfigurationError>::Validation(String::from("test"))),
            AppError::AuthenticationError(openidconnect::DiscoveryError::<openidconnect::ConfigurationError>::Validation(String::from("test")).to_string())
        );
    }

    #[test]
    fn test_app_error_from_quick_xml_error() {
        let error = quick_xml::Error::Syntax(SyntaxError::UnclosedComment);
        assert_eq!(AppError::from(error.clone()), AppError::InternalServerError(error.to_string()));
    }
}