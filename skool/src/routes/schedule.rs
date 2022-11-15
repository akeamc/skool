use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};

use serde::Deserialize;

use skolplattformen::schedule::lessons_by_week;
use skolplattformen::schedule::list_timetables;
use skool_agenda::Lesson;
use tracing::{error, instrument};

use crate::{error::AppError, session::Session, util::IsoWeek, Result};

#[derive(Debug, Deserialize)]
struct ScheduleQuery {
    #[serde(flatten)]
    week: IsoWeek,
}

#[instrument(skip(session))]
async fn schedule(query: web::Query<ScheduleQuery>, session: Session) -> Result<HttpResponse> {
    let lessons = list_lessons(session, query.week.0).await?;

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(3600),
        ]))
        .json(lessons))
}

async fn list_lessons(session: Session, week: chrono::IsoWeek) -> Result<Vec<Lesson>> {
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
