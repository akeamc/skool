use std::{iter, ops::RangeBounds};

use actix_web::{
    http::header::{self, CacheControl, CacheDirective},
    web, FromRequest, HttpRequest, HttpResponse,
};

use chrono::{Datelike, Duration, IsoWeek, NaiveDate, Utc, Weekday};
use futures::{stream, StreamExt, TryStreamExt};
use icalendar::Calendar;
use serde::{de, Deserialize};

use skolplattformen::schedule::lessons_by_week;
use skool_agenda::{Lesson, LessonLike};
use sqlx::postgres::types::PgRange;
use tracing::instrument;

use crate::{
    class,
    error::AppError,
    session::{self, Session},
    share,
    util::{IsoWeekExt, PgRangeExt},
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
        struct Fields {
            year: Option<i32>,
            week: Option<u32>,
            class: Option<String>,
            share: Option<share::Id>,
        }

        let Fields {
            year,
            week,
            class,
            share,
        } = Fields::deserialize(deserializer)?;

        let week = match (year, week) {
            (Some(year), Some(week)) => NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)
                .ok_or_else(|| de::Error::custom("invalid iso week"))?
                .iso_week(),
            (Some(_), None) => return Err(de::Error::missing_field("week")),
            (None, Some(_)) => return Err(de::Error::missing_field("year")),
            (None, None) => Utc::now().date_naive().iso_week(),
        };

        let selection = match (class, share) {
            (Some(class), None) => Selection::Class(class),
            (None, Some(id)) => Selection::OtherUser(id),
            (None, None) => Selection::default(),
            _ => return Err(de::Error::custom("contradictory selection")),
        };

        Ok(Self { week, selection })
    }
}

async fn get_session(
    selection: &Selection,
    ctx: &ApiContext,
    req: &HttpRequest,
) -> Result<(Session, PgRange<NaiveDate>)> {
    Ok(match selection {
        Selection::Class(class) => {
            let session = Session::extract(req).await?;
            let school = class::from_session(session).await?.school;
            let session = session::in_class(&school, class, ctx)
                .await?
                .ok_or(AppError::NotFound("class not found"))?;

            (session, PgRange::full())
        }
        Selection::OtherUser(link) => share::get_session(link, ctx).await?,
        Selection::CurrentUser => (Session::extract(req).await?, PgRange::full()),
    })
}

async fn get_lessons(
    session: Session,
    weeks: impl IntoIterator<Item = IsoWeek>,
) -> Result<Vec<Lesson>> {
    match session {
        Session::Skolplattformen(session) => {
            let client = skolplattformen::Client::new(session)?;
            let timetable = crate::skolplattformen::single_timetable(&client).await?;
            let selection = skolplattformen::schedule::Selection::Student(&timetable.person_guid);

            let mut weeks = stream::iter(weeks)
                .map(|week| lessons_by_week(&client, &timetable.unit_guid, &selection, week))
                .buffer_unordered(8);
            let mut out = Vec::new();
            while let Some(week) = weeks.try_next().await? {
                out.extend(week);
            }
            Ok(out)
        }
    }
}

#[instrument(skip(ctx, req))]
async fn schedule(
    web::Query(query): web::Query<Query>,
    req: HttpRequest,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let (session, allowed_range) = get_session(&query.selection, &ctx, &req).await?;

    if !(allowed_range.contains(&query.week.with_weekday(Weekday::Mon).unwrap())
        && allowed_range.contains(&query.week.with_weekday(Weekday::Sun).unwrap()))
    {
        return Err(AppError::InvalidShareLink);
    }

    let lessons = get_lessons(session, iter::once(query.week)).await?;

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(3600),
        ]))
        .json(lessons))
}

#[instrument(skip(ctx, req))]
async fn ical(
    web::Query(query): web::Query<Query>,
    ctx: web::Data<ApiContext>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let first = Utc::now().date_naive() - Duration::weeks(4);
    let (session, range) = get_session(&query.selection, &ctx, &req).await?;
    let mut calendar = Calendar::new();

    let weeks = first
        .iter_weeks()
        .map(|d| d.iso_week())
        .take_while(|w| {
            range.contains(&w.with_weekday(Weekday::Mon).unwrap())
                && range.contains(&w.with_weekday(Weekday::Sun).unwrap())
        })
        .take(28);
    let lessons = get_lessons(session, weeks).await?;
    calendar.extend(lessons.into_iter().map(|l| l.to_event()));

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType("text/calendar".parse().unwrap()))
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .body(calendar.to_string()))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/links").configure(links::config))
        .service(web::resource("").route(web::get().to(schedule)))
        .service(web::resource("/ical").route(web::get().to(ical)));
}
