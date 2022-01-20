use actix_web::{
    cookie::Expiration,
    http::header::{CacheControl, CacheDirective},
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use skolplattformen::{
    auth::{start_session, Session},
    schedule::ScheduleCredentials,
};
use skool_cookie::{
    bake_cookie, cookie_config,
    crypto::{decrypt, encrypt},
    final_cookie, Cookie, CookieConfig,
};

use crate::{error::AppResult, extractor::JsonOrCookie, WebhookConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    refresh_token: String,
}

async fn login(Json(info): Json<LoginInfo>, req: HttpRequest) -> AppResult<HttpResponse> {
    let CookieConfig { key, .. } = cookie_config(&req);

    let refresh_token = encrypt(&info, &key)?;

    Ok(HttpResponse::Ok()
        .json(LoginResponse { refresh_token }))
}

async fn logout(req: HttpRequest) -> HttpResponse {
    let CookieConfig {
        key: _,
        domain,
        path,
    } = cookie_config(&req);

    HttpResponse::NoContent()
        .cookie(final_cookie::<Session>(domain.to_owned(), path.to_owned()).finish())
        .cookie(final_cookie::<ScheduleCredentials>(domain.to_owned(), path.to_owned()).finish())
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .body("")
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionInfo {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionRes {
    session: String,
}

async fn create_session(web::Json(info): web::Json<CreateSessionInfo>, req: HttpRequest) -> AppResult<HttpResponse> {
  let CookieConfig { key, .. } = cookie_config(&req);

  let login_info: LoginInfo = decrypt(&info.refresh_token, &key)?;
  let session = start_session(&login_info.username, &login_info.password).await?;
  let session = encrypt(&session, &key)?;

  Ok(HttpResponse::Ok().json(CreateSessionRes {
    session
  }))
}

// async fn token(
//     conf: web::Data<WebhookConfig>,
// ) -> AppResult<HttpResponse> {
//     let token = encrypt(&info, &conf.key)?;

//     Ok(HttpResponse::Ok()
//         .insert_header(CacheControl(vec![CacheDirective::Private]))
//         .body(token))
// }

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::get().to(logout)))
        .service(web::resource("/session").route(web::post().to(create_session)))
        // .service(web::resource("/token").route(web::get().to(token)))
        ;
}
