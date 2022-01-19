use std::{rc::Rc, str::FromStr};

use actix_web::{
    cookie::Expiration,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, CacheControl, CacheDirective},
    web, HttpMessage, HttpResponse,
};
use agenda::build_calendar;
use chrono::{Datelike, Duration, IsoWeek, NaiveDate, Utc, Weekday};
use futures::{
    future::{ok, LocalBoxFuture, Ready},
    stream, FutureExt, StreamExt,
};
use mime::Mime;
use serde::Deserialize;
use skolplattformen::{
    auth::{start_session, Session},
    schedule::{
        get_schedule_credentials, get_timetable, lessons_by_week, list_timetables,
        ScheduleCredentials,
    },
};
use skool_cookie::{
    bake_cookie, cookie_config, crypto::decrypt, CookieConfig, CookieDough, UsableRequest,
};
use tracing::{debug, instrument, trace, trace_span, Instrument};

use crate::{
    error::{AppError, AppResult},
    WebhookConfig,
};

use super::auth::LoginInfo;

async fn timetables(
    session: Session,
    credentials: web::ReqData<ScheduleCredentials>,
) -> AppResult<HttpResponse> {
    let timetables = list_timetables(&session.into_client()?, &credentials).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .json(timetables))
}

async fn timetable(
    session: Session,
    credentials: web::ReqData<ScheduleCredentials>,
    id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let timetable = get_timetable(&session.into_client()?, &credentials, &id).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .json(timetable))
}

#[derive(Debug, Deserialize)]
struct LessonsQuery {
    year: i32,
    week: u32,
}

impl LessonsQuery {
    pub fn iso_week(&self) -> Option<IsoWeek> {
        NaiveDate::from_isoywd_opt(self.year, self.week, Weekday::Mon).map(|d| d.iso_week())
    }
}

#[instrument(skip(session, credentials))]
async fn lessons(
    session: Session,
    query: web::Query<LessonsQuery>,
    id: web::Path<String>,
    credentials: web::ReqData<ScheduleCredentials>,
) -> AppResult<HttpResponse> {
    let client = session.into_client()?;
    let timetable = get_timetable(&client, &credentials, &id)
        .await?
        .ok_or(AppError::TimetableNotFound)?;
    let week = query
        .iso_week()
        .ok_or_else(|| AppError::BadRequest("invalid week".to_owned()))?;
    let lessons = lessons_by_week(&client, &credentials, &timetable, week).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .json(lessons))
}

#[derive(Debug, Deserialize)]
struct IcalQuery {
    token: String,
}

async fn lessons_ical(
    id: web::Path<String>,
    query: web::Query<IcalQuery>,
    conf: web::Data<WebhookConfig>,
) -> AppResult<HttpResponse> {
    let info = decrypt::<LoginInfo>(&query.token, &conf.key)?;
    let client = start_session(&info.username, &info.password)
        .await?
        .into_client()?;
    let schedule_credentials = get_schedule_credentials(&client).await?;
    let timetable = get_timetable(&client, &schedule_credentials, &id)
        .await?
        .ok_or(AppError::TimetableNotFound)?;
    let now = Utc::now();

    let mut lessons = vec![];

    let weeks = stream::iter(0..25).map(|i| (now + Duration::weeks(i)).iso_week());

    let mut stream = weeks
        .map(|w| lessons_by_week(&client, &schedule_credentials, &timetable, w))
        .buffer_unordered(5);

    while let Some(response) = stream.next().await {
        let mut vec = response?;

        lessons.append(&mut vec);
    }

    debug!("found {} lessons", lessons.len());

    let calendar = build_calendar(lessons.into_iter());

    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::Private]))
        .insert_header(header::ContentType(
            Mime::from_str("text/calendar").unwrap(),
        ))
        .body(calendar.to_string()))
}

struct CredentialsTransform {}

impl<S, B> Transform<S, ServiceRequest> for CredentialsTransform
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = CredentialsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CredentialsMiddleware {
            service: Rc::new(service),
        })
    }
}

struct CredentialsMiddleware<S> {
    service: Rc<S>,
}

#[instrument(skip_all)]
async fn schedule_credentials(req: &impl UsableRequest) -> AppResult<(ScheduleCredentials, bool)> {
    match ScheduleCredentials::from_req(req) {
        Ok(credentials) => {
            trace!("found in cookie");

            Ok((credentials, false))
        }
        Err(_) => {
            trace!("no cookie :(");

            let client = Session::from_req(req)?.into_client()?;
            let credentials = get_schedule_credentials(&client).await?;

            Ok((credentials, true))
        }
    }
}

impl<S, B> Service<ServiceRequest> for CredentialsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        let span = trace_span!("credentials_middleware");

        async move {
            let (credentials, set_cookie) = schedule_credentials(&req).await?;

            let cookie = if set_cookie {
                let CookieConfig { key } = cookie_config(&req);
                let cookie = bake_cookie(&credentials, key)
                    .map_err(AppError::from)?
                    .expires(Expiration::Session)
                    .finish();
                Some(cookie)
            } else {
                None
            };

            req.extensions_mut().insert(credentials);

            let mut res = srv.call(req).await?;

            if let Some(cookie) = cookie {
                res.response_mut().add_cookie(&cookie)?;
            }

            Ok(res)
        }
        .instrument(span)
        .boxed_local()
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/timetables")
            .service(
                web::resource("")
                    .route(web::get().to(timetables))
                    .wrap(CredentialsTransform {}),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(timetable))
                    .wrap(CredentialsTransform {}),
            )
            .service(
                web::resource("/{id}/lessons")
                    .route(web::get().to(lessons))
                    .wrap(CredentialsTransform {}),
            )
            .service(web::resource("/{id}/lessons.ics").route(web::get().to(lessons_ical))),
    );
}
