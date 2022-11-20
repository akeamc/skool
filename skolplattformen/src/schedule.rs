//! Abstractions that make interacting with the
//! [Skolplattformen](https://grundskola.stockholm/skolplattformen/)-[Skola24](https://www.skola24.com/)
//! mess just a bit more sufferable.

use std::collections::HashMap;

use chrono::{IsoWeek, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::{Europe::Stockholm, Tz};
use csscolorparser::Color;
use reqwest::header::HeaderValue;
use select::predicate::Name;
use serde::{de, ser, Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, instrument, trace};
use uuid::Uuid;

use crate::{client::Client, util::get_doc, Error, Result};

/// A (very dumb) Skola24 timetable structure.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Timetable {
    /// GUID of the school.
    pub school_guid: String,

    /// Unit GUID (no idea what it is).
    pub unit_guid: String,

    /// School name, but someone found "id" more suitable.
    #[serde(alias = "schoolID")]
    pub school_id: String,

    /// ID.
    #[serde(alias = "timetableID")]
    pub timetable_id: Option<String>,

    /// GUID of the timetable user.
    pub person_guid: String,

    /// First name of the timetable user.
    pub first_name: String,

    /// Last name of the timetable user.
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
struct Validation {
    // code: u32,
    // message: String,
}

#[derive(Debug, Deserialize)]
struct ResponseWrapper<T> {
    data: T,
    validation: Vec<Validation>,
}

/// List all timetables.
#[instrument(skip(client))]
pub async fn list_timetables(client: &Client) -> Result<Vec<Timetable>> {
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

    trace!("sending request");

    let res = client
        .0
        .post("https://fns.stockholm.se/ng/api/services/skola24/get/personal/timetables")
        .json(&json!({
            "getPersonalTimetablesRequest": {
                "hostName": "fns.stockholm.se"
            }
        }))
        .send()
        .await?;

    trace!(status = ?res.status());

    let ResponseWrapper { data, .. } = res.json::<ResponseWrapper<Data>>().await?;
    let PersonalTimetablesResponse { student_timetables } = data.get_personal_timetables_response;

    Ok(student_timetables.unwrap_or_default())
}

#[instrument(skip_all)]
async fn get_render_key(client: &Client) -> Result<String> {
    #[derive(Debug, Deserialize)]
    struct Data {
        key: String,
    }

    trace!("sending request");

    let ResponseWrapper { data, .. } = client
        .0
        .post("https://fns.stockholm.se/ng/api/get/timetable/render/key")
        .json("")
        .send()
        .await?
        .json::<ResponseWrapper<Data>>()
        .await?;

    trace!(?data);

    Ok(data.key)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Lesson {
    guid_id: String,
    texts: Vec<String>,
    time_start: String,
    time_end: String,
    day_of_week_number: u8,
    // block_name: String,
}

impl Lesson {
    const TIME_FMT: &'static str = "%H:%M:%s";
    const TZ: Tz = Stockholm;

    fn weekday(&self) -> Option<Weekday> {
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

    fn checked_agenda_lesson(
        self,
        date: NaiveDate,
        color: Option<Color>,
    ) -> Option<skool_agenda::Lesson> {
        let start = NaiveTime::parse_from_str(&self.time_start, Lesson::TIME_FMT).ok()?;
        let end = NaiveTime::parse_from_str(&self.time_end, Lesson::TIME_FMT).ok()?;

        let mut texts = self.texts.into_iter().filter(|s| !s.is_empty());
        let course = texts.next();
        // `texts` is sometimes [course, location] and sometimes [course, teacher, location]
        let location = texts.next_back();
        let teacher = texts.next();

        Some(skool_agenda::Lesson {
            start: Lesson::TZ
                .from_local_datetime(&date.and_time(start))
                .single()?
                .with_timezone(&Utc),
            end: Lesson::TZ
                .from_local_datetime(&date.and_time(end))
                .single()?
                .with_timezone(&Utc),
            course,
            teacher,
            location,
            id: Uuid::new_v5(&UUID_NAMESPACE, self.guid_id.as_bytes()),
            color,
        })
    }
}

const UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x66, 0x2c, 0x31, 0x31, 0xb1, 0x81, 0x40, 0xdc, 0x88, 0xb4, 0x05, 0x2b, 0x18, 0xce, 0x53, 0x4b,
]);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Box {
    // x: u32,
    // y: u32,
    // width: u32,
    // height: u32,
    b_color: Color,
    // f_color: String,
    // id: usize,
    // parent_id: usize,
    // type: String,
    lesson_guids: Option<Vec<String>>,
}

/// Timetable selection.
#[derive(Debug, Clone, Copy)]
pub enum Selection<'a> {
    /// Select a class by GUID.
    Class(&'a str),
    /// Select a student by person GUID.
    Student(&'a str),
}

impl Serialize for Selection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Map<'a> {
            selection: &'a str,
            selection_type: u8,
        }

        let map = match self {
            Selection::Class(selection) => Map {
                selection,
                selection_type: 0,
            },
            Selection::Student(selection) => Map {
                selection,
                selection_type: 5,
            },
        };

        map.serialize(serializer)
    }
}

/// List lessons in a [`Timetable`] for a specific [`IsoWeek`].
///
/// `unit_guid` can be found in the [`Timetable`] struct.
#[instrument(skip(client))]
pub async fn lessons_by_week(
    client: &Client,
    unit_guid: &str,
    selection: &Selection<'_>,
    week: IsoWeek,
) -> Result<Vec<skool_agenda::Lesson>> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        lesson_info: Option<Vec<Lesson>>,
        box_list: Option<Vec<Box>>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Req<'a> {
        render_key: &'a str,
        host: &'a str,
        unit_guid: &'a str,
        width: u32,
        height: u32,
        #[serde(flatten)]
        selection: &'a Selection<'a>,
        week: u32,
        year: i32,
    }

    let render_key = &get_render_key(client).await?;

    let res = client
        .0
        .post("https://fns.stockholm.se/ng/api/render/timetable")
        .json(&Req {
            render_key,
            host: "fns.stockholm.se",
            unit_guid,
            width: 732,
            height: 550,
            selection,
            week: week.week(),
            year: week.year(),
        })
        .send()
        .await?;

    trace!(status = ?res.status(), content_length = res.content_length());

    let ResponseWrapper { data, validation } = res.json::<ResponseWrapper<Data>>().await?;

    if !validation.is_empty() {
        error!(?validation, "skola24 validation error");
        return Err(Error::ScrapingFailed {
            details: "skola24 validation error".into(),
        });
    }

    let guid_colors: HashMap<String, Color> = data
        .box_list
        .unwrap_or_default()
        .into_iter()
        .flat_map(|b| {
            b.lesson_guids
                .unwrap_or_default()
                .into_iter()
                .map(move |l| (l, b.b_color.clone()))
        })
        .collect();

    let lessons = data
        .lesson_info
        .unwrap_or_default()
        .into_iter()
        .filter_map(|lesson| {
            let color = guid_colors
                .get(&lesson.guid_id)
                .map(std::clone::Clone::clone);
            let date = NaiveDate::from_isoywd_opt(week.year(), week.week(), lesson.weekday()?)?;
            lesson.checked_agenda_lesson(date, color)
        })
        .collect::<Vec<_>>();

    debug!("found {} lessons", lessons.len());

    Ok(lessons)
}

/// A Skola24 class.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    /// Class GUID.
    pub group_guid: String,
    /// Name of the class.
    pub group_name: String,
}

/// A student.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Student {
    // pub class_name: Option<String>,
    // pub name: Option<String>,
    // pub name_and_class: Option<String>,
    // pub no_longer_in_group: bool,
    /// GUID of the student.
    pub person_guid: String,
}

/// Available filters.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Classes.
    pub classes: Vec<Class>,
    // courses: _,
    // groups: _,
    // periods: _,
    // rooms: _,
    /// Students.
    pub students: Vec<Student>,
    // subjects: _,
    // teachers: _,
}

/// Get the available filters ("selection" in Skola24 terms).
///
/// # Errors
///
/// Returns an error if the RPC fails.
pub async fn available_filters(client: &Client, unit_guid: &str) -> Result<Filters> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Req<'a> {
        host_name: &'a str,
        unit_guid: &'a str,
        filters: FiltersReq,
    }

    #[derive(Debug, Serialize)]
    #[allow(clippy::struct_excessive_bools)]
    struct FiltersReq {
        class: bool,
        course: bool,
        group: bool,
        period: bool,
        room: bool,
        student: bool,
        subject: bool,
        teacher: bool,
    }

    impl Default for FiltersReq {
        fn default() -> Self {
            Self {
                class: true,
                course: true,
                group: true,
                period: true,
                room: true,
                student: true,
                subject: true,
                teacher: true,
            }
        }
    }

    let res = client
        .0
        .post("https://fns.stockholm.se/ng/api/get/timetable/selection")
        .json(&Req {
            host_name: "fns.stockholm.se",
            unit_guid,
            filters: FiltersReq::default(),
        })
        .send()
        .await?
        .json::<ResponseWrapper<_>>()
        .await?;

    Ok(res.data)
}

/// Skola24 Scope.
#[derive(Debug)]
pub struct Scope(HeaderValue);

impl Scope {
    /// Get the inner [`HeaderValue`].
    pub fn into_inner(self) -> HeaderValue {
        self.0
    }
}

impl Serialize for Scope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.0.to_str().map_err(ser::Error::custom)?;
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom).map(Self)
    }
}

#[instrument(skip_all)]
pub(crate) async fn get_scope(client: &reqwest::Client) -> Result<Scope, Error> {
    let doc = get_doc(
        client,
        "https://fns.stockholm.se/ng/timetable/timetable-viewer/fns.stockholm.se/",
    )
    .await?;

    let scope: HeaderValue = doc
        .find(Name("nova-widget"))
        .next()
        .and_then(|e| e.attr("scope"))
        .ok_or(Error::ScrapingFailed {
            details: "no scope found".into(),
        })?
        .parse()
        .map_err(|_| Error::ScrapingFailed {
            details: "invalid characters in `Scope`".into(),
        })?;

    debug!(?scope);

    Ok(Scope(scope))
}

#[cfg(test)]
mod tests {
    use std::env;

    use async_once_cell::OnceCell;
    use chrono::{Datelike, NaiveDate};

    use crate::client::Client;

    use super::{lessons_by_week, Selection};

    async fn client() -> Client {
        static CLIENT: OnceCell<Client> = OnceCell::new();

        CLIENT
            .get_or_init(async move {
                dotenv::dotenv();
                let username = env::var("SKOLPLATTFORMEN_TEST_USERNAME")
                    .expect("SKOLPLATTFORMEN_TEST_USERNAME not set");
                let password = env::var("SKOLPLATTFORMEN_TEST_PASSWORD")
                    .expect("SKOLPLATTFORMEN_TEST_PASSWORD not set");
                let session = crate::session::login(&username, &password).await.unwrap();
                Client::new(session).unwrap()
            })
            .await
            .clone()
    }

    #[tokio::test]
    async fn selection() {
        let client = client().await;
        let timetables = super::list_timetables(&client).await.unwrap();
        let filters = super::available_filters(&client, &timetables[0].unit_guid)
            .await
            .unwrap();

        assert!(!filters.classes.is_empty());
        assert!(!filters.students.is_empty());
    }

    #[tokio::test]
    async fn lessons() {
        let client = client().await;
        let mut timetables = super::list_timetables(&client).await.unwrap();
        let timetable = timetables.pop().unwrap();
        let lessons = lessons_by_week(
            &client,
            &timetable.unit_guid,
            &Selection::Student(&timetable.person_guid),
            NaiveDate::from_ymd_opt(2022, 11, 17).unwrap().iso_week(),
        )
        .await
        .unwrap();

        assert!(!lessons.is_empty());
    }
}
