use aes_gcm_siv::aead::OsRng;
use chrono::{DateTime, NaiveDate, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
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
    pub last_used: Option<DateTime<Utc>>,
}

impl Link {
    pub fn new(options: Options) -> Self {
        Self {
            id: Id::new(),
            options,
            last_used: None,
        }
    }
}

pub async fn get_session(id: &Id, ctx: &ApiContext) -> Result<(Session, PgRange<NaiveDate>)> {
    let record = sqlx::query!(
        "SELECT owner, expires_at, range FROM links WHERE id = $1",
        &id.0
    )
    .fetch_optional(&ctx.postgres)
    .await?
    .ok_or(AppError::InvalidShareLink)?;

    if let Some(expires_at) = record.expires_at {
        if Utc::now() >= expires_at {
            return Err(AppError::InvalidShareLink);
        }
    }

    let session = session::get(record.owner, ctx)
        .await?
        .ok_or(AppError::InvalidShareLink)?;

    sqlx::query!("UPDATE links SET last_used = NOW() WHERE id = $1", &id.0)
        .execute(&ctx.postgres)
        .await?;

    Ok((session, record.range))
}
