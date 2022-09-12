use actix_web::{
    body::BoxBody,
    http::{header, StatusCode},
    HttpResponse, ResponseError,
};
use reqwest::header::HeaderValue;
use skolplattformen::schedule::AuthError;
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

    #[error("{0}")]
    Auth(#[from] auth1_sdk::actix::FromRequestError),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::TimetableNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidToken => StatusCode::BAD_REQUEST,
            Self::Auth(e) => e.status_code(),
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Self::Auth(e) => e.error_response(),
            e => {
                let mut res = HttpResponse::new(e.status_code());
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                );
                res.set_body(BoxBody::new(e.to_string()))
            }
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

pub type Result<T, E = AppError> = core::result::Result<T, E>;
