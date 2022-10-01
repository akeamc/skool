use actix_web::{web, FromRequest};
use aes_gcm_siv::{Aes256GcmSiv, Key};
use auth1_sdk::Identity;
use deadpool_redis::redis::{self, aio::ConnectionLike};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::{
    credentials::Credentials,
    crypt::{decrypt_bytes, encrypt_bytes},
    error::AppError,
    ApiContext, Result,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Session {
    Skolplattformen(skolplattformen::schedule::Session),
}

impl Session {
    pub const fn ttl(&self) -> usize {
        match self {
            Session::Skolplattformen(_) => 15 * 60,
        }
    }
}

pub fn cache_key(user: Uuid) -> String {
    format!("v{}:sessions:{user}", env!("CARGO_PKG_VERSION"))
}

pub async fn save_to_cache<C: ConnectionLike>(
    session: &Session,
    user: Uuid,
    aes_key: &Key<Aes256GcmSiv>,
    conn: &mut C,
) -> Result<()> {
    let cache_key = cache_key(user);

    redis::pipe()
        .set(&cache_key, encrypt_bytes(session, aes_key)?)
        .expire(&cache_key, session.ttl())
        .query_async::<_, ()>(conn)
        .await?;

    Ok(())
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
        let ctx = web::Data::<ApiContext>::extract(&req).into_inner().unwrap();

        async move {
            let ident = ident.await?;

            let mut redis = ctx.redis.get().await?;

            let cache_key = cache_key(ident.claims.sub);

            let cached = redis::cmd("GET")
                .arg(&cache_key)
                .query_async::<_, Option<Vec<u8>>>(&mut redis)
                .await?
                .and_then(|bytes| decrypt_bytes::<Self>(&bytes, ctx.aes_key()).ok());

            if let Some(cached) = cached {
                debug!("found cached session");
                return Ok(cached);
            }

            debug!("no cached session found");

            let credentials =
                Credentials::get(ident.claims.sub, &ctx.postgres, ctx.aes_key()).await?;

            let session = credentials.into_session().await?;

            save_to_cache(&session, ident.claims.sub, ctx.aes_key(), &mut redis).await?;

            Ok(session)
        }
        .boxed_local()
    }
}
