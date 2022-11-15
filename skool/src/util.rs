use std::fmt::Display;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(into = "SerdeWeek", try_from = "SerdeWeek")]
pub struct IsoWeek(pub chrono::IsoWeek);

#[derive(Debug, Serialize, Deserialize)]
struct SerdeWeek {
    year: i32,
    week: u32,
}

impl From<IsoWeek> for SerdeWeek {
    fn from(w: IsoWeek) -> Self {
        Self {
            year: w.0.year(),
            week: w.0.week(),
        }
    }
}

#[derive(Debug)]
struct SerdeWeekError;

impl Display for SerdeWeekError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid iso week")
    }
}

impl TryFrom<SerdeWeek> for IsoWeek {
    type Error = SerdeWeekError;

    fn try_from(v: SerdeWeek) -> Result<Self, Self::Error> {
        NaiveDate::from_isoywd_opt(v.year, v.week, Weekday::Mon)
            .map(|d| d.iso_week())
            .ok_or(SerdeWeekError)
            .map(Self)
    }
}
