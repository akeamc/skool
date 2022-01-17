use std::rc::Rc;

use actix_web::{
    cookie::Expiration,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{CacheControl, CacheDirective},
    web, HttpMessage, HttpResponse,
};
use chrono::{Datelike, IsoWeek, NaiveDate, Weekday};
use futures::{
    future::{ok, LocalBoxFuture, Ready},
    FutureExt,
};
use serde::Deserialize;
use skolplattformen::{
    auth::Session,
    schedule::{get_scope, get_timetable, lessons_by_week, list_timetables, ScheduleCredentials},
};
use skool_cookie::{bake_cookie, cookie_config, CookieConfig, CookieDough, UsableRequest};

use crate::error::{AppError, AppResult};

async fn timetables(
    session: Session,
    credentials: web::ReqData<ScheduleCredentials>,
) -> AppResult<HttpResponse> {
    let timetables = list_timetables(&session.into_client()?, &credentials).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(timetables))
}

async fn timetable(
    session: Session,
    credentials: web::ReqData<ScheduleCredentials>,
    id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let timetable = get_timetable(&session.into_client()?, &credentials, &id).await?;
    Ok(HttpResponse::Ok().json(timetable))
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

async fn lessons(
    session: Session,
    query: web::Query<LessonsQuery>,
    id: web::Path<String>,
    credentials: web::ReqData<ScheduleCredentials>,
) -> AppResult<HttpResponse> {
    let client = session.into_client()?;
    let timetable = get_timetable(&client, &credentials, &id).await.unwrap();
    let week = query.iso_week().expect("invalid date");
    let lessons = lessons_by_week(&client, &credentials, &timetable.unwrap(), week).await?;
    Ok(HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::Private,
            CacheDirective::MaxAge(300),
        ]))
        .json(lessons))
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

async fn schedule_credentials(req: &impl UsableRequest) -> AppResult<(ScheduleCredentials, bool)> {
    match ScheduleCredentials::from_req(req) {
        Ok(credentials) => Ok((credentials, false)),
        Err(_) => {
            let client = Session::from_req(req)?.into_client()?;
            let scope = get_scope(&client).await?;

            Ok((ScheduleCredentials { scope }, true))
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
        .boxed_local()
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/timetables")
            .wrap(CredentialsTransform {})
            .service(web::resource("").route(web::get().to(timetables)))
            .service(web::resource("/{id}").route(web::get().to(timetable)))
            .service(web::resource("/{id}/lessons").route(web::get().to(lessons))),
    );
}
