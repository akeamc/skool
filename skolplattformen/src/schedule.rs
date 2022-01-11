use chrono::{IsoWeek, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::{Europe::Stockholm, Tz};
use lazy_static::lazy_static;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use scraper::Selector;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::util::get_html;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleCredentials {
    pub scope: String,
}

impl ScheduleCredentials {
    pub fn as_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("X-Scope", HeaderValue::from_str(&self.scope).unwrap());
        headers
    }
}

#[derive(Debug, Error)]
pub enum GetScopeError {
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("scraping failed")]
    ScrapingFailed,
}

pub async fn get_scope(client: &Client) -> Result<String, GetScopeError> {
    lazy_static! {
        static ref NOVA_WIDGET: Selector = Selector::parse("nova-widget").unwrap();
    }

    let doc = get_html(
        client,
        "https://fns.stockholm.se/ng/timetable/timetable-viewer/fns.stockholm.se/",
    )
    .await?;

    let scope = doc
        .select(&NOVA_WIDGET)
        .next()
        .map(|e| e.value().attr("scope"))
        .flatten()
        .ok_or(GetScopeError::ScrapingFailed)?
        .to_owned();

    Ok(scope)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Timetable {
    pub school_guid: String,
    pub unit_guid: String,
    #[serde(alias = "schoolID")]
    pub school_id: String,
    #[serde(alias = "timetableID")]
    pub timetable_id: String,
    pub person_guid: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
struct ResponseWrapper<T> {
    data: T,
}

pub async fn list_timetables(
    client: &Client,
    credentials: &ScheduleCredentials,
) -> Result<Vec<Timetable>, reqwest::Error> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        get_personal_timetables_response: PersonalTimetablesResponse,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PersonalTimetablesResponse {
        // teacher_timetables: Option<Vec<Timetable>>,
        student_timetables: Option<Vec<Timetable>>,
        // children_timetables: Option<Vec<Timetable>>,
    }

    let ResponseWrapper { data } = client
        .post("https://fns.stockholm.se/ng/api/services/skola24/get/personal/timetables")
        .json(&json!({
            "getPersonalTimetablesRequest": {
                "hostName": "fns.stockholm.se"
            }
        }))
        .headers(credentials.as_headers())
        .send()
        .await?
        .json::<ResponseWrapper<Data>>()
        .await?;

    let PersonalTimetablesResponse { student_timetables } = data.get_personal_timetables_response;

    Ok(student_timetables.unwrap_or_default())
}

pub async fn get_timetable(
    client: &Client,
    credentials: &ScheduleCredentials,
    id: &str,
) -> Result<Option<Timetable>, reqwest::Error> {
    let timetables = list_timetables(client, credentials).await?;

    Ok(timetables.into_iter().find(|t| t.timetable_id == id))
}

async fn get_render_key(
    client: &Client,
    credentials: &ScheduleCredentials,
) -> Result<String, reqwest::Error> {
    #[derive(Debug, Deserialize)]
    struct Data {
        key: String,
    }

    let ResponseWrapper { data } = client
        .post("https://fns.stockholm.se/ng/api/get/timetable/render/key")
        .headers(credentials.as_headers())
        .json("")
        .send()
        .await?
        .json::<ResponseWrapper<Data>>()
        .await?;

    Ok(data.key)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    // guid_id: String,
    texts: Vec<String>,
    time_start: String,
    time_end: String,
    day_of_week_number: u8,
    // block_name: String,
}

impl Lesson {
    const TIME_FMT: &'static str = "%H:%M:%s";
    const TZ: Tz = Stockholm;

    pub fn weekday(&self) -> Option<Weekday> {
        match self.day_of_week_number {
            1 => Some(Weekday::Mon),
            2 => Some(Weekday::Tue),
            3 => Some(Weekday::Wed),
            4 => Some(Weekday::Thu),
            5 => Some(Weekday::Fri),
            6 => Some(Weekday::Sat),
            7 => Some(Weekday::Sun),
            _ => None,
        }
    }
}

pub fn try_into_agenda_lesson(lesson: Lesson, date: NaiveDate) -> Option<agenda::Lesson> {
    let start = NaiveTime::parse_from_str(&lesson.time_start, Lesson::TIME_FMT).ok()?;
    let end = NaiveTime::parse_from_str(&lesson.time_end, Lesson::TIME_FMT).ok()?;

    let mut texts = lesson.texts.into_iter();
    let course = texts.next().filter(|s| !s.is_empty());
    // texts is sometimes [course, location] and sometimes [course, teacher, location]
    let location = texts.next_back().filter(|s| !s.is_empty());
    let teacher = texts.next().filter(|s| !s.is_empty());

    Some(agenda::Lesson {
        start: Lesson::TZ
            .from_local_datetime(&date.and_time(start))
            .unwrap()
            .with_timezone(&Utc),
        end: Lesson::TZ
            .from_local_datetime(&date.and_time(end))
            .unwrap()
            .with_timezone(&Utc),
        course,
        teacher,
        location,
    })
}

pub async fn lessons_by_week(
    client: &Client,
    credentials: &ScheduleCredentials,
    timetable: &Timetable,
    week: IsoWeek,
) -> Result<Vec<agenda::Lesson>, reqwest::Error> {
    let render_key = get_render_key(client, credentials).await?;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        lesson_info: Option<Vec<Lesson>>,
    }

    let ResponseWrapper { data } = client
        .post("https://fns.stockholm.se/ng/api/render/timetable")
        .headers(credentials.as_headers())
        .json(&json!({
            "renderKey": render_key,
            "host": "fns.stockholm.se".to_owned(),
            "unitGuid": timetable.unit_guid.to_owned(),
            "width": 732,
            "height": 550,
            "selectionType": 5,
            "selection": timetable.person_guid.to_owned(),
            "week": week.week(),
            "year": week.year(),
        }))
        .send()
        .await?
        .json::<ResponseWrapper<Data>>()
        .await?;

    let lessons = data
        .lesson_info
        .unwrap_or_default()
        .into_iter()
        .filter_map(|lesson| {
            let date = NaiveDate::from_isoywd(week.year(), week.week(), lesson.weekday()?);
            try_into_agenda_lesson(lesson, date)
        })
        .collect::<Vec<_>>();

    Ok(lessons)
}
