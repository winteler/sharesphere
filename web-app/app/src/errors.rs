use http::status::StatusCode;
use strum_macros::{Display, EnumString};
use thiserror::Error;

#[derive(Clone, Debug, Display, EnumString, Error)]
pub enum AppError {
    #[strum(serialize = "Internal Server Error")]
    InternalServerError,
    #[strum(serialize = "Network Error")]
    NetworkError,
    #[strum(serialize = "Not Connected")]
    NotConnected,
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

//struct ServerFnApError(ServerFnError<AppError>);

/*impl<E: std::error::Error> From<E> for ServerFnError<AppError> {
    fn from(value: E) -> Self {
        ServerFnError::<AppError>::ServerError(value.to_string())
    }
}*/