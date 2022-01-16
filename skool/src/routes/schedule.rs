use actix_web::{
    dev::{Service, ServiceRequest},
    http::header::{CacheControl, CacheDirective, HeaderName, HeaderValue},
    web::{self, Payload},
    FromRequest, HttpMessage, HttpRequest, HttpResponse,
};
use chrono::{Datelike, IsoWeek, NaiveDate, Weekday};
use serde::Deserialize;
use skolplattformen::{
    auth::Session,
    schedule::{get_scope, get_timetable, lessons_by_week, list_timetables, ScheduleCredentials},
};
use skool_cookie::CookieDough;

use crate::error::AppResult;

async fn timetables(
    session: Session,
    credentials: web::ReqData<ScheduleCredentials>,
) -> AppResult<HttpResponse> {
    let timetables = list_timetables(&session.into_client()?, &credentials).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(timetables))
}

async fn timetable(
    session: Session,
    credentials: web::ReqData<ScheduleCredentials>,
    id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let timetable = get_timetable(&session.into_client()?, &credentials, &id).await?;
    Ok(HttpResponse::Ok().json(timetable))
}

#[derive(Debug, Deserialize)]
struct LessonsQuery {
    year: i32,
    week: u32,
}

impl LessonsQuery {
    pub fn iso_week(&self) -> Option<IsoWeek> {
        NaiveDate::from_isoywd_opt(self.year, self.week, Weekday::Mon).map(|d| d.iso_week())
    }
}

async fn lessons(
    session: Session,
    query: web::Query<LessonsQuery>,
    id: web::Path<String>,
    credentials: web::ReqData<ScheduleCredentials>,
) -> AppResult<HttpResponse> {
    let client = session.into_client()?;
    let timetable = get_timetable(&client, &credentials, &id).await.unwrap();
    let week = query.iso_week().expect("invalid date");
    let lessons = lessons_by_week(&client, &credentials, &timetable.unwrap(), week).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(lessons))
}

pub async fn schedule_credentials(req: &HttpRequest) -> AppResult<ScheduleCredentials> {
    match ScheduleCredentials::from_req(&req) {
        Ok(credentials) => Ok(credentials),
        Err(_) => {
            let session = Session::from_req(&req)?;
            let scope = get_scope(&session.into_client()?).await?;

            Ok(ScheduleCredentials { scope })
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/timetables")
            .wrap_fn(|req, srv| {
                let (req, payload) = req.into_parts();
                let credentials = schedule_credentials(&req);
                let req = ServiceRequest::from_parts(req, payload);
                req.extensions_mut().insert(credentials);
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut().insert(
                        HeaderName::from_static("x-bruh"),
                        HeaderValue::from_static("indeed"),
                    );
                    Ok(res)
                }
            })
            .service(web::resource("").route(web::get().to(timetables)))
            .service(web::resource("/{id}").route(web::get().to(timetable)))
            .service(web::resource("/{id}/lessons").route(web::get().to(lessons))),
    );
}
