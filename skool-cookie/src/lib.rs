use actix_web::{
    cookie::{time::Duration, Cookie, CookieBuilder, SameSite},
    http::StatusCode,
    web::Data,
    ResponseError,
};
use crypto::{decrypt, encrypt, CryptoError, Key};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub mod crypto;

pub use skool_cookie_derive::Cookie;

pub use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
pub use futures::future;

#[derive(Debug, Error)]
pub enum CookieError {
    #[error("decryption failed")]
    Crypto(#[from] CryptoError),

    #[error("missing cookie")]
    MissingCookie,
}

impl ResponseError for CookieError {
    fn status_code(&self) -> StatusCode {
        match self {
            CookieError::Crypto(_) => StatusCode::BAD_REQUEST,
            CookieError::MissingCookie => StatusCode::BAD_REQUEST,
        }
    }
}

pub fn eat_paranoid_cookie<T>(cookie: Cookie, key: &Key) -> Result<T, CryptoError>
where
    T: DeserializeOwned,
{
    decrypt(cookie.value(), key)
}

pub fn cookie_config(req: &HttpRequest) -> &CookieConfig {
    req.app_data::<Data<CookieConfig>>()
        .expect("CookieConfig not found")
}

#[derive(Debug, Clone, Copy)]
pub struct CookieConfig {
    pub key: Key,
}

pub trait CookieDough {
    const COOKIE_NAME: &'static str;
}

pub fn bake_cookie<V: CookieDough + Serialize>(
    value: &V,
    key: &Key,
) -> Result<CookieBuilder<'static>, CryptoError> {
    let val = encrypt(value, key)?;
    Ok(Cookie::build(V::COOKIE_NAME, val)
        .http_only(true)
        .same_site(SameSite::Strict)
        .domain("localhost")
        .path("/"))
}

pub fn final_cookie<T: CookieDough>() -> CookieBuilder<'static> {
    Cookie::build(T::COOKIE_NAME, "")
        .http_only(true)
        .domain("localhost")
        .path("/")
        .max_age(Duration::seconds(0))
}
