use axum::{http::StatusCode, response::IntoResponse};
use deadpool_redis::redis::RedisError;

use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("internal server error")]
    InternalError,

    #[error("{0}")]
    BadRequest(&'static str),

    #[error("{0}")]
    NotFound(&'static str),

    #[error("timetable not found")]
    TimetableNotFound,

    #[error("missing credentials")]
    MissingCredentials,

    #[error("invalid share link")]
    InvalidShareLink,

    #[error("{0}")]
    Auth(#[from] auth1_sdk::axum::IdentityRejection),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Auth(e) => e.into_response(),
            e => {
                let status = match e {
                    AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
                    AppError::NotFound(_) => StatusCode::NOT_FOUND,
                    AppError::TimetableNotFound => StatusCode::NOT_FOUND,
                    AppError::MissingCredentials => StatusCode::UNAUTHORIZED,
                    AppError::InvalidShareLink => StatusCode::UNAUTHORIZED,
                    Self::Auth(_e) => unreachable!(),
                };

                (status, e.to_string()).into_response()
            }
        }
    }
}

impl From<skolplattformen::Error> for AppError {
    fn from(e: skolplattformen::Error) -> Self {
        match e {
            skolplattformen::Error::BadCredentials => Self::BadRequest("bad credentials"),
            skolplattformen::Error::Http(_) => e.into(),
            skolplattformen::Error::ScrapingFailed { .. } => {
                error!("{:?}", e);
                Self::InternalError
            }
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        error!("http request failed: {}", e);
        Self::InternalError
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        error!("sqlx error: {e}");
        Self::InternalError
    }
}

impl From<crate::crypt::Error> for AppError {
    fn from(e: crate::crypt::Error) -> Self {
        error!("crypt error: {e}");
        Self::InternalError
    }
}

impl From<RedisError> for AppError {
    fn from(e: RedisError) -> Self {
        error!("redis error: {e}");
        Self::InternalError
    }
}

impl From<deadpool_redis::PoolError> for AppError {
    fn from(e: deadpool_redis::PoolError) -> Self {
        error!("redis pool error: {e}");
        Self::InternalError
    }
}
