use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use http::status::StatusCode;
use leptos::{component, IntoView, ServerFnError, view};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::icons::{AuthErrorIcon, InternalErrorIcon, InvalidRequestIcon, NetworkErrorIcon, NotAuthorizedIcon, NotFoundIcon};

const AUTH_FAILED_MESSAGE: &str = "Sorry, we had some trouble identifying you";
const INTERNAL_ERROR_MESSAGE: &str = "Something went wrong";
const NOT_AUTHORIZED_MESSAGE: &str = "You're in a restricted area, please do not resist";

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
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::AuthenticationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotAuthenticated | AppError::InsufficientPrivileges | AppError::ForumBanUntil(_) |
            AppError::PermanentForumBan | AppError::GlobalBanUntil(_) | AppError::PermanentGlobalBan => StatusCode::FORBIDDEN,
            AppError::CommunicationError(error) => match error {
                ServerFnError::Args(_) | ServerFnError::MissingArg(_) => StatusCode::BAD_REQUEST,
                ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::AuthenticationError(_) => String::from(AUTH_FAILED_MESSAGE),
            AppError::NotAuthenticated | AppError::InsufficientPrivileges => String::from(NOT_AUTHORIZED_MESSAGE),
            AppError::ForumBanUntil(timestamp) => format!("You are banned from this forum until {}", timestamp.to_string()),
            AppError::PermanentForumBan => String::from("You are permanently banned from this forum."),
            AppError::GlobalBanUntil(timestamp) => format!("You are globally banned until {}", timestamp.to_string()),
            AppError::PermanentGlobalBan => String::from("You are permanently banned from this website."),
            AppError::CommunicationError(error) => match error {
                ServerFnError::Args(_) | ServerFnError::MissingArg(_) => String::from("Sorry, we didn't understand your request"),
                ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => String::from("Sorry, we've got noise on the line."),
                _ => String::from(INTERNAL_ERROR_MESSAGE),
            },
            AppError::DatabaseError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::InternalServerError(_) => String::from(INTERNAL_ERROR_MESSAGE),
            AppError::NotFound => String::from("There's nothing here"),
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

impl From<&ServerFnError> for AppError {
    fn from(error: &ServerFnError) -> Self {
        match error {
            ServerFnError::ServerError(message) => AppError::from_str(message.as_str()).unwrap_or(AppError::InternalServerError(message.clone())),
            _ => AppError::CommunicationError(error.clone()),
        }
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use sqlx;

    use crate::errors::AppError;

    impl From<sqlx::Error> for AppError {
        fn from(error: sqlx::Error) -> Self {
            match error {
                sqlx::Error::RowNotFound => AppError::NotFound,
                _ => AppError::InternalServerError(error.to_string()),
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

    impl<T: std::error::Error> From<openidconnect::DiscoveryError<T>> for AppError {
        fn from(_error: openidconnect::DiscoveryError<T>) -> Self {
            AppError::AuthenticationError(String::from("Discovery failed"))
        }
    }

    impl From<quick_xml::Error> for AppError {
        fn from(error: quick_xml::Error) -> Self {
            AppError::InternalServerError(error.to_string())
        }
    }
}

#[component]
pub fn AppErrorIcon(
    app_error: AppError,
) -> impl IntoView {
    match app_error {
        AppError::AuthenticationError(_) => view! { <AuthErrorIcon/> },
        AppError::NotAuthenticated | AppError::InsufficientPrivileges |AppError::ForumBanUntil(_) |
        AppError::PermanentForumBan | AppError::GlobalBanUntil(_) | AppError::PermanentGlobalBan => view! { <NotAuthorizedIcon/> }, // TODO better icon for bans (judge, hammer?)
        AppError::CommunicationError(error) => match error {
            ServerFnError::Args(_) | ServerFnError::MissingArg(_) => view! { <InvalidRequestIcon/> },
            ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => view! { <NetworkErrorIcon/> },
            _ => view! { <InternalErrorIcon/> },
        },
        AppError::DatabaseError(_) => view! { <InternalErrorIcon/> },
        AppError::InternalServerError(_) => view! { <InternalErrorIcon/> },
        AppError::NotFound => view! { <NotFoundIcon/> },
    }
}