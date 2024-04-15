use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use http::status::StatusCode;
use leptos::ServerFnError;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    CommunicationError(ServerFnError),
    InternalServerError(String),
    NotAuthenticated,
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::CommunicationError(error) => match error {
                ServerFnError::Args(_) | ServerFnError::MissingArg(_) => StatusCode::BAD_REQUEST,
                ServerFnError::Registration(_) | ServerFnError::Request(_) | ServerFnError::Response(_) => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotAuthenticated => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,

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
    use crate::errors::AppError::{InternalServerError, NotFound};

    impl From<sqlx::Error> for AppError {
        fn from(error: sqlx::Error) -> Self {
            match error {
                sqlx::Error::RowNotFound => NotFound,
                _ => InternalServerError(error.to_string()),
            }
        }
    }
}