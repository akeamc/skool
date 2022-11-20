use chrono::{Datelike, NaiveDate, Weekday};
use serde::{de, Deserialize};

#[derive(Debug, Clone, Copy)]
pub struct IsoWeek(pub chrono::IsoWeek);

impl<'de> Deserialize<'de> for IsoWeek {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Parts {
            year: i32,
            week: u32,
        }

        let Parts { year, week } = Parts::deserialize(deserializer)?;

        IsoWeek::from_parts(year, week).ok_or_else(|| de::Error::custom("invalid iso week"))
    }
}

impl IsoWeek {
    pub fn from_parts(year: i32, week: u32) -> Option<Self> {
        NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).map(|d| Self(d.iso_week()))
    }
}
