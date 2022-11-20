use actix_web::{web, FromRequest};
use aes_gcm_siv::{Aes256GcmSiv, Key};
use auth1_sdk::Identity;
use deadpool_redis::redis::{self, aio::ConnectionLike};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    class::SchoolHash,
    credentials,
    crypt::{decrypt_bytes, encrypt_bytes},
    error::AppError,
    ApiContext, Result,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Session {
    Skolplattformen(skolplattformen::Session),
}

impl Session {
    pub const fn ttl(&self) -> usize {
        15 * 60
    }

    pub async fn create(credentials: &credentials::Private) -> Result<Self> {
        match &credentials {
            credentials::Private::Skolplattformen { username, password } => {
                let session = skolplattformen::login(username, password).await?;
                Ok(Self::Skolplattformen(session))
            }
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

#[instrument(skip(ctx))]
pub async fn get(owner: Uuid, ctx: &ApiContext) -> Result<Option<Session>> {
    let mut redis = ctx.redis.get().await?;

    let cache_key = cache_key(owner);

    let cached = redis::cmd("GET")
        .arg(&cache_key)
        .query_async::<_, Option<Vec<u8>>>(&mut redis)
        .await?
        .and_then(|bytes| decrypt_bytes::<Session>(&bytes, ctx.aes_key()).ok());

    if let Some(cached) = cached {
        debug!("found cached session");
        return Ok(Some(cached));
    }

    debug!("no cached session found");

    match credentials::get(owner, &ctx.postgres, ctx.aes_key()).await? {
        Some(credentials) => {
            let session = Session::create(&credentials.private).await?;

            save_to_cache(&session, owner, ctx.aes_key(), &mut redis).await?;

            Ok(Some(session))
        }
        None => Ok(None),
    }
}

#[instrument(skip(ctx))]
pub async fn in_class(
    school: &SchoolHash,
    class: &str,
    ctx: &ApiContext,
) -> Result<Option<Session>> {
    let user = match sqlx::query!(
        "SELECT uid FROM credentials WHERE school = $1 AND class_reference = $2",
        school.as_ref(),
        class
    )
    .fetch_optional(&ctx.postgres)
    .await?
    {
        Some(record) => record.uid,
        None => return Ok(None),
    };

    let session = get(user, ctx).await?.ok_or(AppError::InternalError)?;
    Ok(Some(session))
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
            get(ident.claims.sub, &ctx)
                .await?
                .ok_or(AppError::MissingCredentials)
        }
        .boxed_local()
    }
}
