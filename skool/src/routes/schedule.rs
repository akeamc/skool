use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, FromRequest, HttpRequest, HttpResponse,
};

use serde::{de, Deserialize};

use skolplattformen::schedule::lessons_by_week;
use skool_agenda::Lesson;
use tracing::instrument;

use crate::{
    class,
    error::AppError,
    session::{self, Session},
    share,
    util::IsoWeek,
    ApiContext, Result,
};

mod links;

#[derive(Debug, Default, Deserialize)]
enum Selection {
    Class(String),
    OtherUser(share::Id),
    #[default]
    CurrentUser,
}

#[derive(Debug)]
struct Query {
    week: IsoWeek,
    selection: Selection,
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
            share: Option<share::Id>,
        }

        let Map {
            year,
            week,
            class,
            share,
        } = Map::deserialize(deserializer)?;
        let week =
            IsoWeek::from_parts(year, week).ok_or_else(|| de::Error::custom("invalid iso week"))?;

        let filter = match (class, share) {
            (Some(class), None) => Selection::Class(class),
            (None, Some(id)) => Selection::OtherUser(id),
            (None, None) => Selection::default(),
            _ => return Err(de::Error::custom("contradictory selection")),
        };

        Ok(Self {
            week,
            selection: filter,
        })
    }
}

#[instrument(skip(ctx))]
async fn schedule(
    query: web::Query<Query>,
    req: HttpRequest,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let query = query.into_inner();
    let lessons = match &query.selection {
        Selection::Class(class) => {
            let session = Session::extract(&req).await?;
            let school = class::from_session(session).await?.school;
            let session = session::in_class(&school, class, &ctx)
                .await?
                .ok_or(AppError::NotFound("class not found"))?;

            list_lessons(session, query).await?
        }
        Selection::OtherUser(link) => share::list_lessons(link, query.week.0, &ctx).await?,
        Selection::CurrentUser => {
            let session = Session::extract(&req).await?;
            list_lessons(session, query).await?
        }
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
            let selection = match query.selection {
                Selection::Class(ref class) => skolplattformen::schedule::Selection::Class(class),
                Selection::CurrentUser => {
                    skolplattformen::schedule::Selection::Student(&timetable.person_guid)
                }
                Selection::OtherUser(_) => unreachable!(),
            };
            let lessons =
                lessons_by_week(&client, &timetable.unit_guid, &selection, query.week.0).await?;

            Ok(lessons)
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/links").configure(links::config))
        .service(web::resource("").route(web::get().to(schedule)));
}
