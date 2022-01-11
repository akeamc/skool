use std::pin::Pin;

use actix_web::{http::StatusCode, web, FromRequest, HttpRequest, ResponseError};
use futures::Future;
use serde::de::DeserializeOwned;
use skool_cookie::CookieError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonOrCookieError {
    #[error("{0}")]
    CookieError(#[from] CookieError),

    #[error("{0}")]
    ActixError(#[from] actix_web::Error),

    #[error("neither json nor cookie provided")]
    Neither,
}

impl ResponseError for JsonOrCookieError {
    fn status_code(&self) -> StatusCode {
        match self {
            JsonOrCookieError::CookieError(e) => e.status_code(),
            JsonOrCookieError::ActixError(e) => e.as_response_error().status_code(),
            JsonOrCookieError::Neither => StatusCode::BAD_REQUEST,
        }
    }
}

pub struct JsonOrCookie<T>(pub T);

impl<T> FromRequest for JsonOrCookie<T>
where
    T: DeserializeOwned + FromRequest,
{
    type Error = JsonOrCookieError;

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        let mut payload = payload.take();

        Box::pin(async move {
            if let Ok(json) = web::Json::<T>::from_request(&req, &mut payload).await {
                return Ok(Self(json.0));
            }

            if let Ok(cookie) = T::from_request(&req, &mut payload).await {
                return Ok(Self(cookie));
            }

            Err(Self::Error::Neither)
        })
    }
}
