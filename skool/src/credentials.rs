use actix_web::{dev, web, FromRequest, HttpRequest};
use aes_gcm_siv::{Aes256GcmSiv, Key};
use auth1_sdk::Identity;
use chrono::{DateTime, Utc};
use futures::{future::LocalBoxFuture, FutureExt};
use sentry::types::Uuid;
use serde::{Deserialize, Serialize};

use sqlx::{PgExecutor, PgPool};
use tracing::error;

use crate::{crypt::decrypt_bytes, error::AppError, session::Session, Result};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "service", rename_all = "snake_case")]
pub enum Kind {
    Skolplattformen { username: String, password: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "service", rename_all = "snake_case")]
pub enum PublicKind {
    Skolplattformen { username: String },
}

impl From<Kind> for PublicKind {
    fn from(k: Kind) -> Self {
        match k {
            Kind::Skolplattformen {
                username,
                password: _,
            } => Self::Skolplattformen { username },
        }
    }
}

#[derive(Debug)]
pub struct Credentials {
    pub updated_at: DateTime<Utc>,
    pub kind: Kind,
}

#[derive(Debug, Serialize)]
pub struct PublicCredentials {
    pub updated_at: DateTime<Utc>,
    #[serde(flatten)]
    pub kind: PublicKind,
}

impl From<Credentials> for PublicCredentials {
    fn from(c: Credentials) -> Self {
        let Credentials { updated_at, kind } = c;

        Self {
            updated_at,
            kind: kind.into(),
        }
    }
}

impl Credentials {
    pub async fn into_session(self) -> Result<Session> {
        match self.kind {
            Kind::Skolplattformen { username, password } => {
                let session =
                    skolplattformen::schedule::start_session(&username, &password).await?;
                Ok(Session::Skolplattformen(session))
            }
        }
    }

    pub async fn get(user: Uuid, db: impl PgExecutor<'_>, key: Key<Aes256GcmSiv>) -> Result<Self> {
        let record = sqlx::query!(
            "SELECT updated_at, data FROM credentials WHERE uid = $1",
            user
        )
        .fetch_optional(db)
        .await?
        .ok_or(AppError::MissingCredentials)?;

        let kind: Kind = decrypt_bytes(&record.data, &key).map_err(|e| {
            error!("decrypt error: {e}");
            AppError::MissingCredentials
        })?;

        Ok(Self {
            updated_at: record.updated_at,
            kind,
        })
    }
}

impl FromRequest for Credentials {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let db = req
            .app_data::<web::Data<PgPool>>()
            .expect("web::Data<PgPool> missing in app_data")
            .clone();
        let key = req
            .app_data::<web::Data<crate::Config>>()
            .expect("web::Data<skool::Config> missing in app_data")
            .aes_key;
        let ident = Identity::from_request(req, payload);

        async move {
            let ident = ident.await?;
            let credentials = Self::get(ident.claims.sub, db.as_ref(), key).await?;

            Ok(credentials)
        }
        .boxed_local()
    }
}
