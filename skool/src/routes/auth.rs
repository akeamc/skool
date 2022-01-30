use actix_web::{
    web::{self},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use skolplattformen::schedule::start_session;
use skool_crypto::{
    crypto::{decrypt, encrypt},
    crypto_config, CryptoConfig,
};

use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CreateSessionInfo {
    UsernamePassword(LoginInfo),
    LoginToken { login_token: String },
}

#[derive(Debug, Serialize)]
pub struct CreateSessionRes {
    session_token: String,
    login_token: Option<String>,
}

async fn create_session(
    web::Json(info): web::Json<CreateSessionInfo>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let CryptoConfig { key, .. } = crypto_config(&req);

    let (login_info, generate_token) = match info {
        CreateSessionInfo::UsernamePassword(info) => (info, true),
        CreateSessionInfo::LoginToken { login_token } => (decrypt(&login_token, key)?, false),
    };

    let session = start_session(&login_info.username, &login_info.password).await?;
    let session_token = encrypt(&session, key)?;
    let login_token = if generate_token {
        Some(encrypt(&login_info, key)?)
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(CreateSessionRes {
        session_token,
        login_token,
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/session").route(web::post().to(create_session)));
}
