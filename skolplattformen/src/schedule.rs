//! Abstractions that make interacting with the
//! [Skolplattformen](https://grundskola.stockholm/skolplattformen/)-[Skola24](https://www.skola24.com/)
//! mess just a bit more sufferable.
use std::collections::HashMap;

use chrono::{IsoWeek, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::{Europe::Stockholm, Tz};
use csscolorparser::Color;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, StatusCode,
};
use select::predicate::Name;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, instrument, trace};
use uuid::Uuid;

use cookie_store::{Cookie, CookieStore};
use reqwest_cookie_store::CookieStoreMutex;
use select::{document::Document, predicate::Class};

use crate::{
    util::{get_doc, scrape_form},
    USER_AGENT,
};

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
    pub timetable_id: String,

    /// GUID of the timetable user.
    pub person_guid: String,

    /// First name of the timetable user.
    pub first_name: String,

    /// Last name of the timetable user.
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
struct ResponseWrapper<T> {
    data: T,
}

/// List all timetables.
#[instrument(skip(client))]
pub async fn list_timetables(client: &AuthorizedClient) -> Result<Vec<Timetable>, reqwest::Error> {
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

    let ResponseWrapper { data } = res.json::<ResponseWrapper<Data>>().await?;

    trace!("deserialized");

    let PersonalTimetablesResponse { student_timetables } = data.get_personal_timetables_response;

    Ok(student_timetables.unwrap_or_default())
}

/// Get a [`Timetable`] by id.
#[instrument(skip(client))]
pub async fn get_timetable(
    client: &AuthorizedClient,
    id: &str,
) -> Result<Option<Timetable>, reqwest::Error> {
    let timetables = list_timetables(client).await?;

    Ok(timetables.into_iter().find(|t| t.timetable_id == id))
}

#[instrument(skip_all)]
async fn get_render_key(client: &AuthorizedClient) -> Result<String, reqwest::Error> {
    #[derive(Debug, Deserialize)]
    struct Data {
        key: String,
    }

    trace!("sending request");

    let ResponseWrapper { data } = client
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

        let mut texts = self.texts.into_iter();
        let course = texts.next().filter(|s| !s.is_empty());
        // `texts` is sometimes [course, location] and sometimes [course, teacher, location]
        let location = texts.next_back().filter(|s| !s.is_empty());
        let teacher = texts.next().filter(|s| !s.is_empty());

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

/// List lessons in a [`Timetable`] for a specific [`IsoWeek`].
#[instrument(skip(client, timetable))]
pub async fn lessons_by_week(
    client: &AuthorizedClient,
    timetable: &Timetable,
    week: IsoWeek,
) -> Result<Vec<skool_agenda::Lesson>, reqwest::Error> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        lesson_info: Option<Vec<Lesson>>,
        box_list: Option<Vec<Box>>,
    }

    let render_key = get_render_key(client).await?;

    let res = client
        .0
        .post("https://fns.stockholm.se/ng/api/render/timetable")
        .json(&json!({
            "renderKey": render_key,
            "host": "fns.stockholm.se".to_owned(),
            "unitGuid": timetable.unit_guid.clone(),
            "width": 732,
            "height": 550,
            "selectionType": 5,
            "selection": timetable.person_guid.clone(),
            "week": week.week(),
            "year": week.year(),
        }))
        .send()
        .await?;

    trace!(status = ?res.status(), content_length = res.content_length());

    let ResponseWrapper { data } = res.json::<ResponseWrapper<Data>>().await?;

    trace!("deserialized");

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
            let date = NaiveDate::from_isoywd(week.year(), week.week(), lesson.weekday()?);
            lesson.checked_agenda_lesson(date, color)
        })
        .collect::<Vec<_>>();

    debug!("found {} lessons", lessons.len());

    Ok(lessons)
}

/// An error that can occur when authorizing.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Bad login credentials, i.e. username and password.
    #[error("bad credentials")]
    BadCredentials,

    /// Some HTTP request failed.
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    /// The scraping failed, most likely due to some unexpected HTML.
    #[error("scraping failed: {details}")]
    ScrapingFailed {
        /// Detailed error information (human readable).
        details: String,
    },
}

#[derive(Debug)]
/// A wrapper around [`Client`] that prevents unauthorized [`Client`]s
/// from accidentaly being passed to Skolplattformen functions.
pub struct AuthorizedClient(Client);

/// Skolplattformen session info.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// Cookies used in this session.
    pub cookies: Vec<Cookie<'static>>,

    /// Skola24 `X-Scope` header.
    pub scope: String,
}

impl Session {
    /// Construct an [`AuthorizedClient`] from this session.
    ///
    /// # Errors
    ///
    /// This function returns an error if the [`Session`]s `scope` for
    /// some reason is an invalid header value, or if the [`Client`]
    /// fails to build.
    #[allow(clippy::missing_panics_doc)]
    pub fn try_into_client(self) -> Result<AuthorizedClient, AuthError> {
        // the only way from_cookies() can be Err is if the iterator yields an Err, so we're safe
        let cookie_store =
            CookieStore::from_cookies(self.cookies.into_iter().map(Result::<_, ()>::Ok), true)
                .unwrap();
        let cookie_store = Arc::new(reqwest_cookie_store::CookieStoreMutex::new(cookie_store));

        let mut headers = HeaderMap::new();

        let scope_header = match HeaderValue::from_str(&self.scope) {
            Ok(v) => v,
            Err(_) => {
                return Err(AuthError::ScrapingFailed {
                    details: format!("invalid header value \"{}\"", self.scope),
                })
            }
        };

        headers.insert("X-Scope", scope_header);

        let client = Client::builder()
            .cookie_provider(cookie_store)
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()?;

        Ok(AuthorizedClient(client))
    }
}

#[instrument(skip(password, cookie_store))]
async fn fill_jar_with_session_data(
    username: &str,
    password: &str,
    cookie_store: Arc<CookieStoreMutex>,
) -> Result<String, AuthError> {
    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .user_agent(USER_AGENT)
        .build()?;

    trace!("GETting login page");

    let doc = get_doc(&client, "https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/authenticate?customer=https://login001.stockholm.se&targetsystem=TimetableViewer").await?;

    let student_href = doc
        .find(Class("navBtn"))
        .find(|e| e.inner_html() == "Elever")
        .and_then(|e| e.attr("href"))
        .ok_or(AuthError::ScrapingFailed {
            details: "no student login button found".into(),
        })?;

    let student_login_url = format!(
        "https://login001.stockholm.se/siteminderagent/forms/{}",
        student_href
    );

    trace!(student_login_url = student_login_url.as_str());

    let student_doc = get_doc(&client, &student_login_url).await?;

    let username_password_href = student_doc
        .find(Class("beta"))
        .next()
        .and_then(|e| e.attr("href"))
        .ok_or(AuthError::ScrapingFailed {
            details: "no username-password option found".into(),
        })?;

    let doc = get_doc(
        &client,
        format!(
            "https://login001.stockholm.se/siteminderagent/forms/{}",
            username_password_href
        ),
    )
    .await?;

    let mut form_body = scrape_form(&doc).ok_or(AuthError::ScrapingFailed {
        details: "no login form found".into(),
    })?;

    form_body.insert("user".to_owned(), username.to_owned());
    form_body.insert("password".to_owned(), password.to_owned());
    form_body.insert("submit".to_owned(), String::new());

    let res = client
        .post("https://login001.stockholm.se/siteminderagent/forms/login.fcc")
        .form(&form_body)
        .send()
        .await?;

    let form_body = scrape_form(&Document::from(res.text().await?.as_str())).ok_or(
        AuthError::ScrapingFailed {
            details: "no sso request form found".into(),
        },
    )?;

    let res = client
        .post("https://login001.stockholm.se/affwebservices/public/saml2sso")
        .form(&form_body)
        .send()
        .await?;

    if res.status() == StatusCode::BAD_REQUEST {
        debug!("bad credentials :(");
        return Err(AuthError::BadCredentials);
    }

    let form_body = scrape_form(&Document::from(res.text().await?.as_str())).ok_or(
        AuthError::ScrapingFailed {
            details: "no sso response form found".into(),
        },
    )?;

    client
        .post("https://fnsservicesso1.stockholm.se/sso-ng/saml-2.0/response")
        .form(&form_body)
        .send()
        .await?;

    let scope = get_scope(&client).await?;

    trace!("all requests done");

    Ok(scope)
}

/// Start a session.
///
/// ```
/// # dotenv::dotenv().ok();
/// # let username = std::env::var("SKOLPLATTFORMEN_TEST_USERNAME").expect("SKOLPLATTFORMEN_TEST_USERNAME not set");
/// # let password = std::env::var("SKOLPLATTFORMEN_TEST_PASSWORD").expect("SKOLPLATTFORMEN_TEST_PASSWORD not set");
/// #
/// # tokio_test::block_on(async {
/// let session = skolplattformen::schedule::start_session(&username, &password).await.unwrap();
///
/// assert!(session.cookie_store.iter_any().count() > 0);
/// # })
/// ```
#[instrument(skip(password))]
pub async fn start_session(username: &str, password: &str) -> Result<Session, AuthError> {
    let cookie_store = CookieStore::default();
    let cookie_store = Arc::new(reqwest_cookie_store::CookieStoreMutex::new(cookie_store));

    let scope = fill_jar_with_session_data(username, password, cookie_store.clone()).await?;

    let lock = Arc::try_unwrap(cookie_store).expect("lock still has multiple owners");
    let cookie_store = lock.into_inner().expect("mutex cannot be locked");
    let cookies = cookie_store
        .iter_any()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    debug!("got {} cookies. yum", cookies.len());

    Ok(Session { cookies, scope })
}

#[instrument(skip_all)]
async fn get_scope(client: &Client) -> Result<String, AuthError> {
    let doc = get_doc(
        client,
        "https://fns.stockholm.se/ng/timetable/timetable-viewer/fns.stockholm.se/",
    )
    .await?;

    let scope = doc
        .find(Name("nova-widget"))
        .next()
        .and_then(|e| e.attr("scope"))
        .ok_or(AuthError::ScrapingFailed {
            details: "no scope found".into(),
        })?
        .to_owned();

    debug!(scope = scope.as_str());

    Ok(scope)
}
