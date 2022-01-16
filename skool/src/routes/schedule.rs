use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};
use chrono::{Datelike, IsoWeek, NaiveDate, Weekday};
use serde::Deserialize;
use skolplattformen::{
    auth::Session,
    schedule::{get_scope, get_timetable, lessons_by_week, list_timetables, ScheduleCredentials},
};

async fn scope(session: Session) -> HttpResponse {
    let scope = get_scope(&session.into_client()).await.unwrap();
    HttpResponse::Ok().body(scope)
}

async fn timetables(
    session: Session,
    credentials: web::Query<ScheduleCredentials>,
) -> HttpResponse {
    let timetables = list_timetables(&session.into_client(), &credentials)
        .await
        .unwrap();
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(timetables)
}

async fn timetable(
    session: Session,
    credentials: web::Query<ScheduleCredentials>,
    id: web::Path<String>,
) -> HttpResponse {
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

impl LessonsQuery {
    pub fn iso_week(&self) -> Option<IsoWeek> {
        NaiveDate::from_isoywd_opt(self.year, self.week, Weekday::Mon).map(|d| d.iso_week())
    }
}

async fn lessons(
    session: Session,
    credentials: web::Query<ScheduleCredentials>,
    query: web::Query<LessonsQuery>,
    id: web::Path<String>,
) -> HttpResponse {
    let client = session.into_client();
    let timetable = get_timetable(&client, &credentials, &id).await.unwrap();
    let week = query.iso_week().expect("invalid date");
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/scope").route(web::get().to(scope)))
        .service(web::resource("/timetables").route(web::get().to(timetables)))
        .service(web::resource("/timetables/{id}").route(web::get().to(timetable)))
        .service(web::resource("/timetables/{id}/lessons").route(web::get().to(lessons)));
}
