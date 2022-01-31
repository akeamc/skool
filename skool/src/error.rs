use actix_web::{http::StatusCode, ResponseError};
use skolplattformen::schedule::AuthError;
use skool_webtoken::crypto::CryptoError;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("internal server error")]
    InternalError,

    #[error("{0}")]
    BadRequest(String),

    #[error("timetable not found")]
    TimetableNotFound,

    #[error("invalid token")]
    InvalidToken,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::TimetableNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidToken => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<AuthError> for AppError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::BadCredentials => Self::BadRequest("bad credentials".to_owned()),
            AuthError::ReqwestError(_) => e.into(),
            AuthError::ScrapingFailed { .. } => {
                error!("{:?}", e);
                Self::InternalError
            }
        }
    }
}

impl From<CryptoError> for AppError {
    fn from(e: CryptoError) -> Self {
        match e {
            CryptoError::Encode(_) => Self::InternalError,
            CryptoError::Decode(_) => Self::InvalidToken,
            CryptoError::Aes => Self::InvalidToken,
            CryptoError::Base64(_) => Self::InvalidToken,
            CryptoError::CiphertextTooShort => Self::InvalidToken,
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        error!("http request failed: {}", e);
        Self::InternalError
    }
}

pub type AppResult<T> = Result<T, AppError>;
