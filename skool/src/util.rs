use std::ops::Bound;

use chrono::{NaiveDate, Weekday};

mod range {
    use std::ops::{Bound, Deref};

    use serde::{Deserialize, Serialize};
    use sqlx::postgres::types::PgRange;

    #[derive(Debug, Clone, sqlx::Type)]
    #[sqlx(transparent)]
    pub struct Range<T>(PgRange<T>);

    impl<T> Range<T> {
        pub const fn new(start: Bound<T>, end: Bound<T>) -> Self {
            Self(PgRange { start, end })
        }

        pub const fn full() -> Self {
            Self::new(Bound::Unbounded, Bound::Unbounded)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Parts<T> {
        start: Bound<T>,
        end: Bound<T>,
    }

    impl<T> Serialize for Range<T>
    where
        T: Serialize + Clone,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Parts {
                start: self.0.start.clone(),
                end: self.0.end.clone(),
            }
            .serialize(serializer)
        }
    }

    impl<'de, T> Deserialize<'de> for Range<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let Parts { start, end } = Parts::deserialize(deserializer)?;

            Ok(Self(PgRange { start, end }))
        }
    }

    impl<T> Deref for Range<T> {
        type Target = PgRange<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> AsRef<PgRange<T>> for Range<T> {
        fn as_ref(&self) -> &PgRange<T> {
            &self.0
        }
    }
}

pub use range::Range;
use sqlx::postgres::types::PgRange;

pub trait IsoWeekExt: Sized {
    fn with_weekday(self, weekday: Weekday) -> Option<NaiveDate>;
}

impl IsoWeekExt for chrono::IsoWeek {
    fn with_weekday(self, weekday: Weekday) -> Option<NaiveDate> {
        NaiveDate::from_isoywd_opt(self.year(), self.week(), weekday)
    }
}

pub trait PgRangeExt<T> {
    fn full() -> Self;
}

impl<T> PgRangeExt<T> for PgRange<T> {
    fn full() -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        }
    }
}
