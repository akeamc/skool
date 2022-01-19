use std::borrow::Cow;

use chrono::{DateTime, Duration, Utc};

use csscolorparser::Color;
use icalendar::{Calendar, Component, Event};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait LessonLike {
    fn teacher(&self) -> Option<Cow<str>>;

    fn location(&self) -> Option<Cow<str>>;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn duration(&self) -> Duration {
        self.end() - self.start()
    }

    fn course(&self) -> Option<Cow<str>>;

    fn id(&self) -> Uuid;

    fn color(&self) -> Option<&Color>;

    fn to_event(&self) -> Event {
        let mut event = Event::new();

        event
            .starts(self.start())
            .ends(self.end())
            .uid(&self.id().to_string())
            .summary(&self.course().unwrap_or_else(|| "(Namnlös)".into()));

        if let Some(location) = self.location() {
            event.location(&location);
        }

        if let Some(teacher) = self.teacher() {
            event.description(&teacher);
        }

        event.done()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lesson {
    pub teacher: Option<String>,
    pub location: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub course: Option<String>,
    pub id: Uuid,
    pub color: Option<Color>,
}

impl LessonLike for Lesson {
    fn teacher(&self) -> Option<Cow<str>> {
        self.teacher.as_ref().map(|s| s.into())
    }

    fn location(&self) -> Option<Cow<str>> {
        self.location.as_ref().map(|s| s.into())
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn course(&self) -> Option<Cow<str>> {
        self.course.as_ref().map(|s| s.into())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }
}

pub fn build_calendar<T: LessonLike>(lessons: impl Iterator<Item = T>) -> Calendar {
    Calendar::from_iter(lessons.map(|l| l.to_event()))
}
