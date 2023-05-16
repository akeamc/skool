use std::{iter, ops::RangeBounds};

use axum::{
    body::Body,
    extract::{FromRequestParts, Query, State},
    http::{request::Parts, Request},
    response::IntoResponse,
    routing::get,
    Json, Router,
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
    AppState, Result,
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
struct ScheduleQuery {
    week: IsoWeek,
    selection: Selection,
}

// workaround for https://github.com/serde-rs/serde/issues/1183
impl<'de> Deserialize<'de> for ScheduleQuery {
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
    ctx: &AppState,
    parts: &mut Parts,
) -> Result<(Session, PgRange<NaiveDate>)> {
    Ok(match selection {
        Selection::Class(class) => {
            let session = Session::from_request_parts(parts, ctx).await?;
            let school = class::from_session(session).await?.school;
            let session = session::in_class(&school, class, ctx)
                .await?
                .ok_or(AppError::NotFound("class not found"))?;

            (session, PgRange::full())
        }
        Selection::OtherUser(link) => share::get_session(link, ctx).await?,
        Selection::CurrentUser => (
            Session::from_request_parts(parts, ctx).await?,
            PgRange::full(),
        ),
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
    Query(query): Query<ScheduleQuery>,
    State(ctx): State<AppState>,
    req: Request<Body>,
) -> Result<impl IntoResponse> {
    let (mut parts, _) = req.into_parts();
    let (session, allowed_range) = get_session(&query.selection, &ctx, &mut parts).await?;

    if !(allowed_range.contains(&query.week.with_weekday(Weekday::Mon).unwrap())
        && allowed_range.contains(&query.week.with_weekday(Weekday::Sun).unwrap()))
    {
        return Err(AppError::InvalidShareLink);
    }

    let lessons = get_lessons(session, iter::once(query.week)).await?;

    Ok(([("cache-control", "private; max-age=3600")], Json(lessons)))
}

#[instrument(skip(ctx, req))]
async fn ical(
    Query(query): Query<ScheduleQuery>,
    State(ctx): State<AppState>,
    req: Request<Body>,
) -> Result<impl IntoResponse> {
    let (mut parts, _) = req.into_parts();
    let first = Utc::now().date_naive() - Duration::weeks(4);
    let (session, range) = get_session(&query.selection, &ctx, &mut parts).await?;
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

    Ok((
        [
            ("content-type", "text/calendar"),
            ("cache-control", "no-cache"),
        ],
        calendar.to_string(),
    ))
}

pub fn routes() -> Router<AppState> {
    Router::<_>::new()
        .nest("/links", links::routes())
        .route("/", get(schedule))
        .route("/ical", get(ical))
}
