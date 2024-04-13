use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use thiserror::Error;
pub const INTERNAL_SERVER_ERROR_STR : &str = "Internal Server Error";

#[derive(Clone, Debug, EnumString, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("Internal Server Error")]
    #[strum(serialize = "Internal Server Error")]
    InternalServerError,
    #[error("Network Error")]
    #[strum(serialize = "Network Error")]
    NetworkError,
    #[error("Not Connected")]
    #[strum(serialize = "Not Connected")]
    NotConnected,
    #[error("Not Found")]
    #[strum(serialize = "Not Found")]
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NetworkError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotConnected => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

/*struct ServerFnApError(ServerFnError<AppError>);

impl<E: std::error::Error> From<E> for ServerFnApError {
    fn from(value: E) -> Self {
        ServerFnApError {0: ServerFnError::ServerError(value.to_string()) }
    }
}*/