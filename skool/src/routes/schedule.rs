use std::str::FromStr;

use actix_web::{
    http::header::{self, CacheControl, CacheDirective},
    web, HttpRequest, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use agenda::build_calendar;
use chrono::{Datelike, Duration, IsoWeek, NaiveDate, Utc, Weekday};
use futures::{stream, FutureExt, StreamExt};
use mime::Mime;
use serde::Deserialize;
use skolplattformen::{
    auth::{start_session, Session},
    schedule::{
        get_schedule_credentials, get_timetable, lessons_by_week, list_timetables,
        ScheduleCredentials,
    },
};
use skool_crypto::{crypto::decrypt, crypto_config, CryptoConfig};
use tracing::{debug, instrument};

use crate::{
    error::{AppError, AppResult},
    WebhookConfig,
};

use super::auth::LoginInfo;

async fn timetables(
    credentials: web::Query<ScheduleCredentials>,
    bearer: BearerAuth,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let CryptoConfig { key, .. } = crypto_config(&req);
    let session = decrypt::<Session>(bearer.token(), key)?;

    let timetables = list_timetables(&session.into_client()?, &credentials).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .json(timetables))
}

async fn timetable(
    id: web::Path<String>,
    credentials: web::Query<ScheduleCredentials>,
    bearer: BearerAuth,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let CryptoConfig { key, .. } = crypto_config(&req);
    let session = decrypt::<Session>(bearer.token(), key)?;

    let timetable = get_timetable(&session.into_client()?, &credentials, &id).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .json(timetable))
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

#[instrument(skip(credentials, bearer, req))]
async fn lessons(
    query: web::Query<LessonsQuery>,
    id: web::Path<String>,
    credentials: web::Query<ScheduleCredentials>,
    bearer: BearerAuth,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let CryptoConfig { key, .. } = crypto_config(&req);
    let session = decrypt::<Session>(bearer.token(), key)?;

    let client = session.into_client()?;
    let timetable = get_timetable(&client, &credentials, &id)
        .await?
        .ok_or(AppError::TimetableNotFound)?;
    let week = query
        .iso_week()
        .ok_or_else(|| AppError::BadRequest("invalid week".to_owned()))?;
    let lessons = lessons_by_week(&client, &credentials, &timetable, week).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .json(lessons))
}

#[derive(Debug, Deserialize)]
struct IcalQuery {
    webhook_token: String,
}

async fn lessons_ical(
    id: web::Path<String>,
    query: web::Query<IcalQuery>,
    conf: web::Data<WebhookConfig>,
) -> AppResult<HttpResponse> {
    let info = decrypt::<LoginInfo>(&query.webhook_token, &conf.key)?;
    let client = start_session(&info.username, &info.password)
        .await?
        .into_client()?;
    let schedule_credentials = get_schedule_credentials(&client).await?;
    let timetable = get_timetable(&client, &schedule_credentials, &id)
        .await?
        .ok_or(AppError::TimetableNotFound)?;
    let now = Utc::now();

    let mut lessons = vec![];

    let weeks = stream::iter(0..25).map(|i| (now + Duration::weeks(i)).iso_week());

    let mut stream = weeks
        .map(|w| lessons_by_week(&client, &schedule_credentials, &timetable, w))
        .buffer_unordered(5);

    while let Some(response) = stream.next().await {
        let mut vec = response?;

        lessons.append(&mut vec);
    }

    debug!("found {} lessons", lessons.len());

    let calendar = build_calendar(lessons.into_iter());

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .insert_header(header::ContentType(
            Mime::from_str("text/calendar").unwrap(),
        ))
        .body(calendar.to_string()))
}

async fn credentials(bearer: BearerAuth, req: HttpRequest) -> AppResult<HttpResponse> {
    let CryptoConfig { key, .. } = crypto_config(&req);
    let session = decrypt::<Session>(bearer.token(), key)?;
    let credentials = get_schedule_credentials(&session.into_client()?).await?;

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .insert_header((header::VARY, header::AUTHORIZATION.as_str()))
        .json(credentials))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/timetables")
            .service(web::resource("").route(web::get().to(timetables)))
            .service(web::resource("/{id}").route(web::get().to(timetable)))
            .service(web::resource("/{id}/lessons").route(web::get().to(lessons)))
            .service(web::resource("/{id}/lessons.ics").route(web::get().to(lessons_ical))),
    )
    .service(web::resource("/credentials").route(web::get().to(credentials)));
}
