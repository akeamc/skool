use aes_gcm_siv::{Aes256GcmSiv, Key};
use auth1_sdk::Identity;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use chrono::{DateTime, Utc};

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use sqlx::PgExecutor;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{class::SchoolHash, crypt::decrypt_bytes, error::AppError, AppState, Result};

fn serialize_secret<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "service", rename_all = "snake_case")]
pub enum Private {
    Skolplattformen {
        username: String,
        #[serde(serialize_with = "serialize_secret")]
        password: SecretString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "service", rename_all = "snake_case")]
pub enum Public {
    Skolplattformen { username: String },
}

impl From<Private> for Public {
    fn from(p: Private) -> Self {
        match p {
            Private::Skolplattformen {
                username,
                password: _,
            } => Self::Skolplattformen { username },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub updated_at: DateTime<Utc>,
    pub class: Option<String>,
    pub school: Option<SchoolHash>,
    pub private: Private,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicCredentials {
    pub updated_at: DateTime<Utc>,
    pub class: Option<String>,
    pub school: Option<SchoolHash>,
    #[serde(flatten)]
    pub public: Public,
}

impl From<Credentials> for PublicCredentials {
    fn from(c: Credentials) -> Self {
        let Credentials {
            updated_at,
            private,
            class,
            school,
        } = c;

        Self {
            updated_at,
            class,
            school,
            public: private.into(),
        }
    }
}

#[instrument(name = "get_credentials", skip(db, key))]
pub async fn get(
    user: Uuid,
    db: impl PgExecutor<'_>,
    key: &Key<Aes256GcmSiv>,
) -> Result<Option<Credentials>> {
    let record = match sqlx::query!(
        "SELECT updated_at, data, school, class_reference FROM credentials WHERE uid = $1",
        user
    )
    .fetch_optional(db)
    .await?
    {
        Some(record) => record,
        None => return Ok(None),
    };

    let class = record.class_reference;
    let school = record.school.and_then(|v| v.as_slice().try_into().ok());

    match decrypt_bytes(&record.data, key) {
        Ok(private) => Ok(Some(Credentials {
            updated_at: record.updated_at,
            class,
            school,
            private,
        })),
        Err(e) => {
            error!("decrypt failed: {e}");
            Ok(None)
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Credentials {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ident = Identity::from_request_parts(parts, state).await?;

        let credentials = get(ident.claims.sub, &state.postgres, state.aes_key())
            .await?
            .ok_or(AppError::MissingCredentials)?;

        Ok(credentials)
    }
}
