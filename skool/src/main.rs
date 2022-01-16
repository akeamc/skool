use actix_cors::Cors;
use actix_web::{
    cookie::Expiration,
    http::header::{CacheControl, CacheDirective},
    middleware::Logger,
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::{Datelike, NaiveDate};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use skolplattformen::{
    auth::{start_session, Session},
    schedule::{get_scope, get_timetable, lessons_by_week, list_timetables, ScheduleCredentials},
};
use skool::extractor::JsonOrCookie;
use skool_cookie::{bake_cookie, cookie_config, final_cookie, Cookie, CookieConfig};

#[derive(Debug, Serialize, Deserialize, Cookie)]
#[cookie_name("login_info")]
struct LoginInfo {
    username: String,
    password: String,
}

async fn session_status(session: Session) -> impl Responder {
    dbg!(session);
    HttpResponse::Ok().body("session is ok")
}

async fn login(JsonOrCookie(info): JsonOrCookie<LoginInfo>, req: HttpRequest) -> impl Responder {
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

async fn logout() -> impl Responder {
    HttpResponse::NoContent()
        .cookie(final_cookie::<Session>().finish())
        .cookie(final_cookie::<LoginInfo>().finish())
        .body("")
}

async fn scope(session: Session) -> impl Responder {
    let scope = get_scope(&session.into_client()).await.unwrap();
    HttpResponse::Ok().body(scope)
}

async fn timetables(
    session: Session,
    credentials: web::Query<ScheduleCredentials>,
) -> impl Responder {
    let timetables = list_timetables(&session.into_client(), &credentials)
        .await
        .unwrap();
    HttpResponse::Ok().json(timetables)
}

async fn timetable(
    session: Session,
    credentials: web::Query<ScheduleCredentials>,
    id: web::Path<String>,
) -> impl Responder {
    let timetable = get_timetable(&session.into_client(), &credentials, &id)
        .await
        .unwrap();
    HttpResponse::Ok().json(timetable)
}

#[derive(Debug, Deserialize)]
struct LessonsQuery {
    year: i32,
    week: u32,
}

async fn lessons(
    session: Session,
    credentials: web::Query<ScheduleCredentials>,
    query: web::Query<LessonsQuery>,
    id: web::Path<String>,
) -> impl Responder {
    let client = session.into_client();
    let timetable = get_timetable(&client, &credentials, &id).await.unwrap();
    let week = NaiveDate::from_isoywd_opt(query.year, query.week, chrono::Weekday::Mon)
        .expect("invalid date")
        .iso_week();
    let lessons = lessons_by_week(&client, &credentials, &timetable.unwrap(), week)
        .await
        .unwrap();
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(lessons)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let conf = CookieConfig {
        key: *b"bruhbruhbruhbruhbruhbruhbruhbruh",
    };

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(conf))
            .route("/login", web::post().to(login))
            .route("/logout", web::get().to(logout))
            .route("/session", web::get().to(session_status))
            .route("/scope", web::get().to(scope))
            .route("/timetables", web::get().to(timetables))
            .route("/timetables/{id}", web::get().to(timetable))
            .route("/timetables/{id}/lessons", web::get().to(lessons))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
