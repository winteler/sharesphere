use http::status::StatusCode;
use strum_macros::{Display, EnumString};
use thiserror::Error;

#[derive(Clone, Debug, Display, EnumString, Error)]
pub enum AppError {
    #[strum(serialize = "Internal Server Error")]
    InternalServerError,
    #[strum(serialize = "Not Authenticated")]
    NotAuthenticated,
    #[strum(serialize = "Not Found")]
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotAuthenticated => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

/*impl<E: std::error::Error> From<E> for AppError {
    fn from(value: E) -> Self {
        ServerFnError::<AppError>::ServerError(value.to_string())
    }
}*/

#[cfg(feature = "ssr")]
mod ssr {
    use sqlx;

    use crate::errors::AppError;
    use crate::errors::AppError::{InternalServerError, NotFound};

    impl From<sqlx::Error> for AppError {
        fn from(error: sqlx::Error) -> Self {
            match error {
                sqlx::Error::RowNotFound => NotFound,
                _ => InternalServerError,
            }
        }
    }

    /*impl<T> From<openidconnect::StandardErrorResponse<T>> for AppError {
        fn from(_error: openidconnect::StandardErrorResponse<T>) -> Self {
            InternalServerError
        }
    }*/
}