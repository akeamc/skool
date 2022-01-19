use actix_web::{http::StatusCode, ResponseError};
use skolplattformen::{auth::AuthError, schedule::GetScopeError};
use skool_cookie::{crypto::CryptoError, CookieError};
use thiserror::Error;

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
            AuthError::ReqwestError(_) => Self::InternalError,
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
    fn from(_: reqwest::Error) -> Self {
        Self::InternalError
    }
}

impl From<CookieError> for AppError {
    fn from(e: CookieError) -> Self {
        match e {
            CookieError::Crypto(e) => e.into(),
            CookieError::MissingCookie => Self::BadRequest("missing cookie".to_owned()),
        }
    }
}

impl From<GetScopeError> for AppError {
    fn from(e: GetScopeError) -> Self {
        match e {
            GetScopeError::ReqwestError(e) => e.into(),
            GetScopeError::ScrapingFailed => AppError::InternalError,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
