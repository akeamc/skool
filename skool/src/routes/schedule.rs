use std::ops::RangeBounds;

use actix_web::{
    http::header::{self, CacheControl, CacheDirective},
    web, FromRequest, HttpRequest, HttpResponse,
};

use chrono::{Datelike, Duration, Utc, Weekday};
use futures::{stream, StreamExt, TryStreamExt};
use icalendar::Calendar;
use serde::{de, Deserialize};

use skolplattformen::schedule::lessons_by_week;
use skool_agenda::{Lesson, LessonLike};
use tracing::instrument;

use crate::{
    class,
    error::AppError,
    session::{self, Session},
    share,
    util::{IsoWeek, IsoWeekExt},
    ApiContext, Result,
};

mod links;

#[derive(Debug, Default, Deserialize)]
enum Selection {
    Class(String),
    OtherUser(share::Id),
    #[default]
    CurrentUser,
}

#[derive(Debug)]
struct Query {
    week: IsoWeek,
    selection: Selection,
}

// workaround for https://github.com/serde-rs/serde/issues/1183
impl<'de> Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct Map {
            year: i32,
            week: u32,
            class: Option<String>,
            share: Option<share::Id>,
        }

        let Map {
            year,
            week,
            class,
            share,
        } = Map::deserialize(deserializer)?;
        let week =
            IsoWeek::from_parts(year, week).ok_or_else(|| de::Error::custom("invalid iso week"))?;

        let filter = match (class, share) {
            (Some(class), None) => Selection::Class(class),
            (None, Some(id)) => Selection::OtherUser(id),
            (None, None) => Selection::default(),
            _ => return Err(de::Error::custom("contradictory selection")),
        };

        Ok(Self {
            week,
            selection: filter,
        })
    }
}

#[instrument(skip(ctx))]
async fn schedule(
    query: web::Query<Query>,
    req: HttpRequest,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let query = query.into_inner();
    let lessons = match &query.selection {
        Selection::Class(class) => {
            let session = Session::extract(&req).await?;
            let school = class::from_session(session).await?.school;
            let session = session::in_class(&school, class, &ctx)
                .await?
                .ok_or(AppError::NotFound("class not found"))?;

            list_lessons(session, query).await?
        }
        Selection::OtherUser(link) => share::list_lessons(link, query.week.0, &ctx).await?,
        Selection::CurrentUser => {
            let session = Session::extract(&req).await?;
            list_lessons(session, query).await?
        }
    };

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(3600),
        ]))
        .json(lessons))
}

#[derive(Debug, serde::Serialize, Deserialize)]
struct IcalQuery {
    id: share::Id,
}

#[instrument(skip_all)]
async fn ical(
    web::Query(query): web::Query<IcalQuery>,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let first = Utc::now().date_naive() - Duration::weeks(4);
    let (session, range) = share::get_session(&query.id, None, &ctx).await?;
    let mut calendar = Calendar::new();

    match session {
        Session::Skolplattformen(session) => {
            let client = skolplattformen::Client::new(session)?;
            let timetable = crate::skolplattformen::single_timetable(&client).await?;

            let weeks = first
                .iter_weeks()
                .map(|d| d.iso_week())
                .take_while(|w| {
                    range.contains(&w.with_weekday(Weekday::Mon).unwrap())
                        && range.contains(&w.with_weekday(Weekday::Sun).unwrap())
                })
                .take(28);

            let selection = skolplattformen::schedule::Selection::Student(&timetable.person_guid);

            let mut lessons = stream::iter(weeks)
                .map(|week| lessons_by_week(&client, &timetable.unit_guid, &selection, week))
                .buffer_unordered(4);

            while let Some(lessons) = lessons.try_next().await? {
                calendar.extend(lessons.into_iter().map(|l| l.to_event()))
            }
        }
    }

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType("text/calendar".parse().unwrap()))
        .body(calendar.to_string()))
}

async fn list_lessons(session: Session, query: Query) -> Result<Vec<Lesson>> {
    match session {
        Session::Skolplattformen(session) => {
            let client = skolplattformen::Client::new(session)?;
            let timetable = crate::skolplattformen::single_timetable(&client).await?;
            let selection = match query.selection {
                Selection::Class(ref class) => skolplattformen::schedule::Selection::Class(class),
                Selection::CurrentUser => {
                    skolplattformen::schedule::Selection::Student(&timetable.person_guid)
                }
                Selection::OtherUser(_) => unreachable!(),
            };
            let lessons =
                lessons_by_week(&client, &timetable.unit_guid, &selection, query.week.0).await?;

            Ok(lessons)
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/links").configure(links::config))
        .service(web::resource("").route(web::get().to(schedule)))
        .service(web::resource("/ical").route(web::get().to(ical)));
}
