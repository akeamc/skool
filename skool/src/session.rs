use actix_web::{web, FromRequest};
use auth1_sdk::Identity;
use chrono::IsoWeek;
use deadpool_redis::redis;
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use skolplattformen::schedule::{lessons_by_week, list_timetables};
use skool_agenda::Lesson;
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::{
    credentials::Credentials,
    crypt::{decrypt_bytes, encrypt_bytes},
    error::AppError,
    Result,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Session {
    Skolplattformen(skolplattformen::schedule::Session),
}

impl Session {
    pub async fn list_lessons(self, week: IsoWeek) -> Result<Vec<Lesson>> {
        match self {
            Session::Skolplattformen(session) => {
                let client = session.try_into_client()?;

                let timetable = &list_timetables(&client).await?[0];

                let lessons = lessons_by_week(&client, timetable, week).await?;

                Ok(lessons)
            }
        }
    }

    pub const fn ttl(&self) -> usize {
        match self {
            Session::Skolplattformen(_) => 15 * 60,
        }
    }
}

pub fn cache_key(user: Uuid) -> String {
    format!("v{}:sessions:{user}", env!("CARGO_PKG_VERSION"))
}

pub async fn purge<C: redis::aio::ConnectionLike>(conn: &mut C, user: Uuid) -> Result<()> {
    redis::cmd("DEL")
        .arg(cache_key(user))
        .query_async(conn)
        .await?;
    Ok(())
}

impl FromRequest for Session {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();

        let ident = Identity::from_request(&req, payload);
        let key = req
            .app_data::<web::Data<crate::Config>>()
            .expect("web::Data<skool::Config> missing in app_data")
            .aes_key;

        async move {
            let ident = ident.await?;

            let mut redis = req
                .app_data::<deadpool_redis::Pool>()
                .expect("web::Data<deadpool_redis::Pool> missing in app_data")
                .get()
                .await?;

            let cache_key = cache_key(ident.claims.sub);

            let cached = redis::cmd("GET")
                .arg(&cache_key)
                .query_async::<_, Option<Vec<u8>>>(&mut redis)
                .await?
                .and_then(|bytes| decrypt_bytes::<Self>(&bytes, &key).ok());

            if let Some(cached) = cached {
                debug!("found cached session");
                return Ok(cached);
            }

            debug!("no cached session found");

            let db = req
                .app_data::<web::Data<PgPool>>()
                .expect("web::Data<PgPool> missing in app_data")
                .clone();

            let credentials = Credentials::get(ident.claims.sub, db.as_ref(), key).await?;

            let session = credentials.into_session().await?;

            redis::pipe()
                .set(&cache_key, encrypt_bytes(&session, &key)?)
                .expire(&cache_key, session.ttl())
                .query_async::<_, ()>(&mut redis)
                .await?;

            Ok(session)
        }
        .boxed_local()
    }
}
