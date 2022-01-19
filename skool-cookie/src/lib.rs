use std::borrow::Cow;

use actix_web::{
    cookie::{time::Duration, Cookie, CookieBuilder, SameSite},
    dev::ServiceRequest,
    http::StatusCode,
    web::Data,
    ResponseError,
};
use crypto::{decrypt, encrypt, CryptoError, Key};
use serde::{de::DeserializeOwned, Serialize};
use structopt::StructOpt;
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

pub fn cookie_config(req: &impl UsableRequest) -> &CookieConfig {
    req.app_data::<Data<CookieConfig>>()
        .expect("CookieConfig not found")
}

#[derive(Debug, Clone, StructOpt)]
pub struct CookieConfig {
    #[structopt(name = "cookie-key", env = "COOKIE_KEY", hide_env_values = true)]
    pub key: Key,

    #[structopt(name = "cookie-domain", env = "COOKIE_DOMAIN", long)]
    pub domain: String,

    #[structopt(name = "cookie-path", env = "COOKIE_PATH", long)]
    pub path: String,
}

pub trait CookieDough {
    const COOKIE_NAME: &'static str;

    fn from_req(req: &impl UsableRequest) -> Result<Self, CookieError>
    where
        Self: std::marker::Sized;
}

pub trait UsableRequest {
    fn cookie(&self, name: &str) -> Option<Cookie<'static>>;

    fn app_data<T: 'static>(&self) -> Option<&T>;
}

impl UsableRequest for HttpRequest {
    fn cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.cookie(name)
    }

    fn app_data<T: 'static>(&self) -> Option<&T> {
        self.app_data()
    }
}

impl UsableRequest for ServiceRequest {
    fn cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.cookie(name)
    }

    fn app_data<T: 'static>(&self) -> Option<&T> {
        self.app_data()
    }
}

pub fn bake_cookie<V: CookieDough + Serialize>(
    value: &V,
    key: &Key,
    domain: String,
    path: String,
) -> Result<CookieBuilder<'static>, CryptoError> {
    let val = encrypt(value, key)?;
    Ok(Cookie::build(V::COOKIE_NAME, val)
        .http_only(true)
        .same_site(SameSite::Strict)
        .domain(domain)
        .path(path))
}

pub fn final_cookie<'c, T: CookieDough>(domain: String, path: String) -> CookieBuilder<'c> {
    Cookie::build(T::COOKIE_NAME, "")
        .http_only(true)
        .domain(domain)
        .path(path)
        .max_age(Duration::seconds(0))
}
