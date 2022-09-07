use actix_web::{web, FromRequest, HttpRequest, HttpResponse};
use auth1_sdk::Identity;
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    crypt::{decrypt_bytes, encrypt_bytes},
    error::{AppError, AppResult},
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "service", rename_all = "snake_case")]
pub enum Credentials {
    Skolplattformen { username: String, password: String },
}

#[derive(Serialize)]
#[serde(tag = "service", rename_all = "snake_case")]
pub enum PublicCredentials {
    Skolplattformen { username: String },
}

impl From<Credentials> for PublicCredentials {
    fn from(c: Credentials) -> Self {
        match c {
            Credentials::Skolplattformen {
                username,
                password: _,
            } => PublicCredentials::Skolplattformen { username },
        }
    }
}

async fn save_credentials(
    identity: Identity,
    creds: web::Json<Credentials>,
    config: web::Data<crate::Config>,
    db: web::Data<PgPool>,
) -> AppResult<HttpResponse> {
    let creds = creds.into_inner();

    match &creds {
        Credentials::Skolplattformen { username, password } => {
            let _ = skolplattformen::schedule::start_session(username, password).await?;
        }
    }

    let d = encrypt_bytes(&creds, &config.aes_key)?;

    sqlx::query!(
        r#"
          INSERT INTO credentials (uid, data) VALUES ($1, $2)
          ON CONFLICT (uid) DO UPDATE
            SET data = EXCLUDED.data
        "#,
        identity.claims.sub,
        d
    )
    .execute(db.as_ref())
    .await?;

    Ok(HttpResponse::Ok().finish())
}

async fn get_credentials(creds: Credentials) -> HttpResponse {
    HttpResponse::Ok().json(PublicCredentials::from(creds))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::post().to(save_credentials))
            .route(web::get().to(get_credentials)),
    );
}

impl FromRequest for Credentials {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
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
                "SELECT data FROM credentials WHERE uid = $1",
                ident.claims.sub
            )
            .fetch_optional(db.as_ref())
            .await?
            .ok_or_else(|| AppError::BadRequest("no credentials found".into()))?;

            decrypt_bytes(&record.data, &key).map_err(Into::into)
        }
        .boxed_local()
    }
}
