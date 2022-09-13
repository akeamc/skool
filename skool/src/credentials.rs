use actix_web::{dev, web, FromRequest, HttpRequest};
use auth1_sdk::Identity;
use chrono::{DateTime, Utc};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use skolplattformen::schedule::AuthorizedClient;
use sqlx::PgPool;
use tracing::error;

use crate::{crypt::decrypt_bytes, error::AppError, Result};

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
    pub async fn into_client(self) -> Result<AuthorizedClient> {
        let session = match self.kind {
            Kind::Skolplattformen { username, password } => {
                skolplattformen::schedule::start_session(&username, &password).await?
            }
        };

        session.try_into_client().map_err(Into::into)
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
            let record = sqlx::query!(
                "SELECT updated_at, data FROM credentials WHERE uid = $1",
                ident.claims.sub
            )
            .fetch_optional(db.as_ref())
            .await?
            .ok_or(AppError::MissingCredentials)?;

            let inner: Kind = decrypt_bytes(&record.data, &key).map_err(|e| {
                error!("decrypt error: {e}");
                AppError::MissingCredentials
            })?;

            Ok(Credentials {
                updated_at: record.updated_at,
                kind: inner,
            })
        }
        .boxed_local()
    }
}
