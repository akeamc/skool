use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};
use chrono::{Datelike, IsoWeek, NaiveDate, Weekday};

use serde::Deserialize;
use skolplattformen::schedule::{lessons_by_week, list_timetables};

use tracing::instrument;

use crate::{credentials::Credentials, error::AppError, Result};

#[derive(Debug, Deserialize)]
struct ScheduleQuery {
    year: i32,
    week: u32,
}

impl ScheduleQuery {
    pub fn iso_week(&self) -> Option<IsoWeek> {
        NaiveDate::from_isoywd_opt(self.year, self.week, Weekday::Mon).map(|d| d.iso_week())
    }
}

#[instrument(skip(creds))]
async fn schedule(query: web::Query<ScheduleQuery>, creds: Credentials) -> Result<HttpResponse> {
    let client = creds.into_client().await?;

    let timetable = &list_timetables(&client).await?[0];

    let week = query
        .iso_week()
        .ok_or_else(|| AppError::BadRequest("invalid week".to_owned()))?;
    let lessons = lessons_by_week(&client, timetable, week).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(lessons))
}

// async fn lessons_ical(
//     id: web::Path<String>,
//     query: web::Query<IcalQuery>,
//     conf: web::Data<WebhookConfig>,
// ) -> AppResult<HttpResponse> {
//     let info = decrypt::<LoginInfo>(&query.webhook_token, &conf.key)?;
//     let client = start_session(&info.username, &info.password)
//         .await?
//         .try_into_client()?;
//     let timetable = get_timetable(&client, &id)
//         .await?
//         .ok_or(AppError::TimetableNotFound)?;
//     let now = Utc::now();

//     let mut lessons = vec![];

//     let weeks = stream::iter(0..25).map(|i| (now + Duration::weeks(i)).iso_week());

//     let mut stream = weeks
//         .map(|w| lessons_by_week(&client, &timetable, w))
//         .buffer_unordered(5);

//     while let Some(response) = stream.next().await {
//         let mut vec = response?;

//         lessons.append(&mut vec);
//     }

//     debug!("found {} lessons", lessons.len());

//     let calendar = build_calendar(lessons.into_iter());

//     Ok(HttpResponse::Ok()
//         .insert_header(CacheControl(vec![CacheDirective::Private]))
//         .insert_header(header::ContentType(
//             Mime::from_str("text/calendar").unwrap(),
//         ))
//         .body(calendar.to_string()))
// }

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(schedule)));
}
