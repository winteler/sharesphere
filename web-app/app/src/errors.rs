use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use http::status::StatusCode;
use leptos::prelude::*;
use leptos::{component, prelude::ServerFnError, view, IntoView};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::icons::{AuthErrorIcon, BannedIcon, InternalErrorIcon, InvalidRequestIcon, NetworkErrorIcon, NotAuthorizedIcon, NotFoundIcon};

const NOT_AUTHENTICATED_MESSAGE: &str = "Please authenticate yourself.";
const AUTH_FAILED_MESSAGE: &str = "Sorry, we had some trouble authenticating you.";
const INTERNAL_ERROR_MESSAGE: &str = "Something went wrong.";
const NOT_AUTHORIZED_MESSAGE: &str = "You're in a restricted area, please do not resist.";
const FORUM_BAN_UNTIL_MESSAGE: &str = "You are banned from this forum until";
const PERMANENT_FORUM_BAN_MESSAGE: &str = "You are permanently banned from this website.";
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
    ForumBanUntil(chrono::DateTime<chrono::Utc>),
    PermanentForumBan,
    GlobalBanUntil(chrono::DateTime<chrono::Utc>),
    PermanentGlobalBan,
    CommunicationError(ServerFnError),
    DatabaseError(String),
    InternalServerError(String),
    NotFound,
    PayloadTooLarge(usize),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::AuthenticationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotAuthenticated | AppError::InsufficientPrivileges | AppError::ForumBanUntil(_) |
            AppError::PermanentForumBan | AppError::GlobalBanUntil(_) | AppError::PermanentGlobalBan => StatusCode::FORBIDDEN,
            AppError::CommunicationError(error) => match error {
                ServerFnError::Args(_) | ServerFnError::MissingArg(_) | ServerFnError::Serialization(_) | ServerFnError::Deserialization(_) => StatusCode::BAD_REQUEST,
                ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => StatusCode::SERVICE_UNAVAILABLE,
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
            AppError::ForumBanUntil(timestamp) => format!("{} {}", FORUM_BAN_UNTIL_MESSAGE, timestamp),
            AppError::PermanentForumBan => String::from(PERMANENT_FORUM_BAN_MESSAGE),
            AppError::GlobalBanUntil(timestamp) => format!("{} {}", GLOBAL_BAN_UNTIL_MESSAGE, timestamp),
            AppError::PermanentGlobalBan => String::from(PERMANENT_GLOBAL_BAN_MESSAGE),
            AppError::CommunicationError(error) => match error {
                ServerFnError::Args(_) | ServerFnError::MissingArg(_) |
                ServerFnError::Serialization(_) | ServerFnError::Deserialization(_) => String::from(BAD_REQUEST_MESSAGE),
                ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => String::from(UNAVAILABLE_MESSAGE),
                _ => String::from(INTERNAL_ERROR_MESSAGE),
            },
            AppError::DatabaseError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::InternalServerError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::NotFound => String::from(NOT_FOUND_MESSAGE),
            AppError::PayloadTooLarge(mb_limit) => format!("Payload exceeds the {mb_limit} MB limit.")
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

impl From<&ServerFnError<AppError>> for AppError {
    fn from(error: &ServerFnError<AppError>) -> Self {
        match error {
            ServerFnError::WrappedServerError(app_error) => app_error.clone(),
            ServerFnError::ServerError(message) => AppError::from_str(message.as_str()).unwrap_or(AppError::InternalServerError(message.clone())),
            _ => AppError::CommunicationError(ServerFnError::from(error)),
        }
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use sqlx;
    use std::io::Error;

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

    impl From<openidconnect::url::ParseError> for AppError {
        fn from(error: openidconnect::url::ParseError) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<openidconnect::ClaimsVerificationError> for AppError {
        fn from(error: openidconnect::ClaimsVerificationError) -> Self {
            AppError::AuthenticationError(error.to_string())
        }
    }

    impl From<openidconnect::SigningError> for AppError {
        fn from(error: openidconnect::SigningError) -> Self {
            AppError::AuthenticationError(error.to_string())
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

    impl From<std::io::Error> for AppError {
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
        AppError::ForumBanUntil(_) | AppError::PermanentForumBan | AppError::GlobalBanUntil(_) | AppError::PermanentGlobalBan => view! { <BannedIcon/> }.into_any(),
        AppError::CommunicationError(error) => match error {
            ServerFnError::Args(_) | ServerFnError::MissingArg(_) => view! { <InvalidRequestIcon/> }.into_any(),
            ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => view! { <NetworkErrorIcon/> }.into_any(),
            _ => view! { <InternalErrorIcon/> }.into_any(),
        },
        AppError::DatabaseError(_) => view! { <InternalErrorIcon/> }.into_any(),
        AppError::InternalServerError(_) => view! { <InternalErrorIcon/> }.into_any(),
        AppError::NotFound => view! { <NotFoundIcon/> }.into_any(),
        // TODO custom icon for payload too large
        AppError::PayloadTooLarge(_) => view! { <InternalErrorIcon/> }.into_any(),
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
    use std::str::FromStr;

    use http::StatusCode;
    use leptos::prelude::ServerFnError;
    use leptos::server_fn::error::NoCustomError;

    use crate::errors::{AppError, AUTH_FAILED_MESSAGE, BAD_REQUEST_MESSAGE, FORUM_BAN_UNTIL_MESSAGE, GLOBAL_BAN_UNTIL_MESSAGE, INTERNAL_ERROR_MESSAGE, NOT_AUTHENTICATED_MESSAGE, NOT_AUTHORIZED_MESSAGE, NOT_FOUND_MESSAGE, PERMANENT_FORUM_BAN_MESSAGE, PERMANENT_GLOBAL_BAN_MESSAGE, UNAVAILABLE_MESSAGE};

    #[test]
    fn test_app_error_status_code() {
        let test_string = String::from("test");
        let test_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let server_fn_error = ServerFnError::new("test");
        let args_error = ServerFnError::Args(String::from("test"));
        let missing_arg_error = ServerFnError::MissingArg(String::from("test"));
        let request_error = ServerFnError::Request(String::from("test"));
        let response_error = ServerFnError::Response(String::from("test"));
        let registration_error = ServerFnError::Registration(String::from("test"));
        let serialization_error = ServerFnError::Serialization(String::from("test"));
        let deserialization_error = ServerFnError::Deserialization(String::from("test"));
        let wrapper_error = ServerFnError::WrappedServerError(NoCustomError);
        assert_eq!(AppError::AuthenticationError(test_string.clone()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::NotAuthenticated.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::InsufficientPrivileges.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::ForumBanUntil(test_timestamp).status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::PermanentForumBan.status_code(), StatusCode::FORBIDDEN);
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
        assert_eq!(AppError::CommunicationError(wrapper_error).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::DatabaseError(test_string.clone()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::InternalServerError(test_string.clone()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::NotFound.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_app_error_user_message() {
        let test_string = String::from("test");
        let test_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let server_fn_error = ServerFnError::new("test");
        let args_error = ServerFnError::Args(String::from("test"));
        let missing_arg_error = ServerFnError::MissingArg(String::from("test"));
        let request_error = ServerFnError::Request(String::from("test"));
        let response_error = ServerFnError::Response(String::from("test"));
        let registration_error = ServerFnError::Registration(String::from("test"));
        let serialization_error = ServerFnError::Serialization(String::from("test"));
        let deserialization_error = ServerFnError::Deserialization(String::from("test"));
        let wrapper_error = ServerFnError::WrappedServerError(NoCustomError);
        assert_eq!(AppError::AuthenticationError(test_string.clone()).user_message(), String::from(AUTH_FAILED_MESSAGE));
        assert_eq!(AppError::NotAuthenticated.user_message(), String::from(NOT_AUTHENTICATED_MESSAGE));
        assert_eq!(AppError::InsufficientPrivileges.user_message(), String::from(NOT_AUTHORIZED_MESSAGE));
        assert_eq!(AppError::ForumBanUntil(test_timestamp).user_message(), format!("{} {}", FORUM_BAN_UNTIL_MESSAGE, test_timestamp.clone().to_string()));
        assert_eq!(AppError::PermanentForumBan.user_message(), String::from(PERMANENT_FORUM_BAN_MESSAGE));
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
        assert_eq!(AppError::CommunicationError(wrapper_error).user_message(), String::from(INTERNAL_ERROR_MESSAGE));
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
        let server_fn_error = ServerFnError::new("test");
        let server_fn_error_2 = ServerFnError::MissingArg(test_string.clone());
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
            AppError::from_str(AppError::ForumBanUntil(test_timestamp).to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::ForumBanUntil(test_timestamp)
        );
        assert_eq!(
            AppError::from_str(AppError::PermanentForumBan.to_string().as_str()).expect("AppError should be convert to string and back"),
            AppError::PermanentForumBan
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
    fn test_app_error_from_server_fn_error() {
        let internal_error_str = "Internal error";
        let request_error = ServerFnError::Request(String::from("Request error"));
        let response_error = ServerFnError::Response(String::from("Response error"));
        let args_error = ServerFnError::Args(String::from("Args error"));
        let missing_arg_error = ServerFnError::MissingArg(String::from("Missing arg error"));
        let serialization_error = ServerFnError::Serialization(String::from("Serialization error"));
        let deserialization_error = ServerFnError::Deserialization(String::from("Deserialization error"));
        let registration_error = ServerFnError::Registration(String::from("Registration error"));
        let wrapped_error = ServerFnError::WrappedServerError(NoCustomError);

        assert_eq!(AppError::from(&ServerFnError::new(internal_error_str)), AppError::InternalServerError(String::from(internal_error_str)));
        assert_eq!(AppError::from(&request_error), AppError::CommunicationError(request_error));
        assert_eq!(AppError::from(&response_error), AppError::CommunicationError(response_error));
        assert_eq!(AppError::from(&args_error), AppError::CommunicationError(args_error));
        assert_eq!(AppError::from(&missing_arg_error), AppError::CommunicationError(missing_arg_error));
        assert_eq!(AppError::from(&serialization_error), AppError::CommunicationError(serialization_error));
        assert_eq!(AppError::from(&deserialization_error), AppError::CommunicationError(deserialization_error));
        assert_eq!(AppError::from(&registration_error), AppError::CommunicationError(registration_error));
        assert_eq!(AppError::from(&wrapped_error), AppError::CommunicationError(wrapped_error));
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
        let error = openidconnect::url::ParseError::InvalidDomainCharacter;
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
        let error = quick_xml::Error::InvalidPrefixBind {
            prefix: vec![1],
            namespace: vec![1],
        };
        assert_eq!(AppError::from(error.clone()), AppError::InternalServerError(error.to_string()));
    }
}