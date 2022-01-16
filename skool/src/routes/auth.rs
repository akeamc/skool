use actix_web::{cookie::Expiration, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use skolplattformen::auth::{start_session, Session};
use skool_cookie::{bake_cookie, cookie_config, final_cookie, Cookie, CookieConfig};

use crate::extractor::JsonOrCookie;

#[derive(Debug, Serialize, Deserialize, Cookie)]
#[cookie_name("login_info")]
pub struct LoginInfo {
    username: String,
    password: String,
}

async fn login(JsonOrCookie(info): JsonOrCookie<LoginInfo>, req: HttpRequest) -> HttpResponse {
    let CookieConfig { key } = cookie_config(&req);

    let session = start_session(&info.username, &info.password).await.unwrap();
    let login_info = bake_cookie(&info, key).unwrap().permanent().finish();
    let session_cookie = bake_cookie(&session, key)
        .unwrap()
        .expires(Expiration::Session)
        .finish();

    HttpResponse::Ok()
        .cookie(session_cookie)
        .cookie(login_info)
        .body("OK")
}

async fn logout() -> HttpResponse {
    HttpResponse::NoContent()
        .cookie(final_cookie::<Session>().finish())
        .cookie(final_cookie::<LoginInfo>().finish())
        .body("")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::get().to(logout)));
}
