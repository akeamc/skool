use aes_gcm_siv::{Aes256GcmSiv, Key};
use auth1_sdk::Identity;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use deadpool_redis::redis::{self, aio::ConnectionLike};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    class::SchoolHash,
    credentials,
    crypt::{decrypt_bytes, encrypt_bytes},
    error::AppError,
    AppState, Result,
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

#[instrument(name = "get_session", skip(ctx))]
pub async fn get(owner: Uuid, ctx: &AppState) -> Result<Option<Session>> {
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
pub async fn in_class(school: &SchoolHash, class: &str, ctx: &AppState) -> Result<Option<Session>> {
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

#[async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ident = Identity::from_request_parts(parts, state).await?;

        get(ident.id(), state)
            .await?
            .ok_or(AppError::MissingCredentials)
    }
}
