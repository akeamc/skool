use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};
use chrono::{Datelike, IsoWeek, NaiveDate, Weekday};

use serde::Deserialize;

use ::skolplattformen::schedule::lessons_by_week;
use skolplattformen::schedule::list_timetables;
use skool_agenda::Lesson;
use tracing::{error, instrument};

use crate::{error::AppError, session::Session, Result};

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

#[instrument(skip(session))]
async fn schedule(query: web::Query<ScheduleQuery>, session: Session) -> Result<HttpResponse> {
    let week = query
        .iso_week()
        .ok_or_else(|| AppError::BadRequest("invalid week".to_owned()))?;

    let lessons = list_lessons(session, week).await?;

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(3600),
            CacheDirective::Extension("stale-while-revalidate".into(), Some("604800".into())),
        ]))
        .json(lessons))
}

async fn list_lessons(session: Session, week: IsoWeek) -> Result<Vec<Lesson>> {
    match session {
        Session::Skolplattformen(session) => {
            let client = session.try_into_client()?;
            let mut timetables = list_timetables(&client).await?.into_iter();
            let timetable = timetables.next().ok_or_else(|| {
                error!("got 0 timetables");
                AppError::InternalError
            })?;
            let lessons = lessons_by_week(&client, &timetable, week).await?;

            Ok(lessons)
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(schedule)));
}
