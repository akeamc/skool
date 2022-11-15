use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    expires: Option<DateTime<Utc>>,
    detailed: bool,
    // range: Option<RangeInclusive<IsoWeek>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    id: Uuid,
    options: Options,
}
