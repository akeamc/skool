use std::ops::{RangeBounds, RangeInclusive};

use aes_gcm_siv::aead::OsRng;
use chrono::{DateTime, NaiveDate, Utc, Weekday};
use rand::Rng;
use serde::{Deserialize, Serialize};
use skolplattformen::schedule::lessons_by_week;
use skool_agenda::Lesson;
use sqlx::postgres::types::PgRange;

use crate::{
    error::AppError,
    session::{self, Session},
    util, ApiContext, Result,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct Id(#[serde(with = "hex::serde")] [u8; 32]);

impl Id {
    #[allow(clippy::new_without_default)] // debatable
    pub fn new() -> Self {
        let mut v = [0; 32];
        OsRng.fill(&mut v);
        Self(v)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    /// Optional expiration of the token.
    pub expires_at: Option<DateTime<Utc>>,
    // /// Whether to share the entire timetable or just basic availability info
    // /// (busy or free).
    // pub detailed: bool,
    /// Range of weeks to share. `None` means all weeks are shared.
    pub range: util::Range<NaiveDate>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            expires_at: None,
            range: util::Range::full(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Link {
    pub id: Id,
    #[sqlx(flatten)]
    pub options: Options,
}

impl Link {
    pub fn new(options: Options) -> Self {
        Self {
            id: Id::new(),
            options,
        }
    }
}

pub async fn get_session(
    id: &Id,
    range: Option<RangeInclusive<NaiveDate>>,
    ctx: &ApiContext,
) -> Result<(Session, PgRange<NaiveDate>)> {
    let record = sqlx::query!(
        "SELECT owner, expires_at, range FROM links WHERE id = $1",
        &id.0
    )
    .fetch_optional(&ctx.postgres)
    .await?
    .ok_or(AppError::InvalidShareLink)?;

    if let Some(range) = range {
        if !(record.range.contains(range.start()) && record.range.contains(range.end())) {
            return Err(AppError::InvalidShareLink);
        };
    }

    if let Some(expires_at) = record.expires_at {
        if Utc::now() >= expires_at {
            return Err(AppError::InvalidShareLink);
        }
    }

    let session = session::get(record.owner, ctx)
        .await?
        .ok_or(AppError::InvalidShareLink)?;

    Ok((session, record.range))
}

pub async fn list_lessons(id: &Id, week: chrono::IsoWeek, ctx: &ApiContext) -> Result<Vec<Lesson>> {
    let start = NaiveDate::from_isoywd_opt(week.year(), week.week(), Weekday::Mon).unwrap();
    let end = NaiveDate::from_isoywd_opt(week.year(), week.week(), Weekday::Sun).unwrap();

    let lessons = match get_session(id, Some(start..=end), ctx).await?.0 {
        Session::Skolplattformen(session) => {
            let client = skolplattformen::Client::new(session)?;
            let timetable = crate::skolplattformen::single_timetable(&client).await?;
            lessons_by_week(
                &client,
                &timetable.unit_guid,
                &skolplattformen::schedule::Selection::Student(&timetable.person_guid),
                week,
            )
            .await?
        }
    };

    Ok(lessons)
}
