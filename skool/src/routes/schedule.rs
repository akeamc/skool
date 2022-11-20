use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};

use serde::{de, Deserialize};

use skolplattformen::schedule::{lessons_by_week, Selection};
use skool_agenda::Lesson;
use tracing::instrument;

use crate::{
    class,
    error::AppError,
    session::{self, Session},
    util::IsoWeek,
    ApiContext, Result,
};

#[derive(Debug, Default, Deserialize)]
enum Filter {
    Class(String),
    #[default]
    CurrentUser,
}

#[derive(Debug)]
struct Query {
    week: IsoWeek,
    filter: Filter,
}

// workaround for https://github.com/serde-rs/serde/issues/1183
impl<'de> Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct Map {
            year: i32,
            week: u32,
            class: Option<String>,
        }

        let Map { year, week, class } = Map::deserialize(deserializer)?;
        let week =
            IsoWeek::from_parts(year, week).ok_or_else(|| de::Error::custom("invalid iso week"))?;

        let filter = match class {
            Some(class) => Filter::Class(class),
            None => Filter::default(),
        };

        Ok(Self { week, filter })
    }
}

#[instrument(skip(session, ctx))]
async fn schedule(
    query: web::Query<Query>,
    session: Session,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let query = query.into_inner();
    let lessons = match &query.filter {
        Filter::Class(class) => {
            let school = class::from_session(session).await?.school;
            let session = session::in_class(&school, class, &ctx)
                .await?
                .ok_or(AppError::NotFound("class not found"))?;

            list_lessons(session, query).await?
        }
        Filter::CurrentUser => list_lessons(session, query).await?,
    };

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(3600),
        ]))
        .json(lessons))
}

async fn list_lessons(session: Session, query: Query) -> Result<Vec<Lesson>> {
    match session {
        Session::Skolplattformen(session) => {
            let client = skolplattformen::Client::new(session)?;
            let timetable = crate::skolplattformen::single_timetable(&client).await?;
            let selection = match query.filter {
                Filter::Class(ref class) => Selection::Class(class),
                Filter::CurrentUser => Selection::Student(&timetable.person_guid),
            };
            let lessons =
                lessons_by_week(&client, &timetable.unit_guid, &selection, query.week.0).await?;

            Ok(lessons)
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(schedule)));
}
