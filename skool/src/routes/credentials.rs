use actix_web::{
    web::{self, Payload},
    FromRequest, HttpRequest, HttpResponse,
};
use auth1_sdk::Identity;
use sqlx::PgPool;

use crate::{
    credentials::{self, Credentials, PublicCredentials},
    crypt::encrypt_bytes,
    error::AppError,
    session, Result,
};

async fn save_credentials(
    identity: Identity,
    creds: web::Json<credentials::Kind>,
    config: web::Data<crate::Config>,
    db: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<HttpResponse> {
    let creds = creds.into_inner();

    match &creds {
        credentials::Kind::Skolplattformen { username, password } => {
            let _ = skolplattformen::schedule::start_session(username, password).await?;
        }
    }

    let d = encrypt_bytes(&creds, &config.aes_key)?;

    let record = sqlx::query!(
        r#"
          INSERT INTO credentials (uid, data, updated_at) VALUES ($1, $2, DEFAULT)
          ON CONFLICT (uid) DO UPDATE
            SET (data, updated_at) = (EXCLUDED.data, EXCLUDED.updated_at)
          RETURNING updated_at
        "#,
        identity.claims.sub,
        d,
    )
    .fetch_one(db.as_ref())
    .await?;

    session::purge(&mut redis.get().await?, identity.claims.sub).await?;

    let creds = PublicCredentials {
        kind: creds.into(),
        updated_at: record.updated_at,
    };

    Ok(HttpResponse::Created().json(creds))
}

async fn get_credentials(req: HttpRequest, payload: Payload) -> Result<HttpResponse> {
    let mut payload = payload.into_inner();

    match Credentials::from_request(&req, &mut payload).await {
        Ok(c) => Ok(HttpResponse::Ok().json(PublicCredentials::from(c))),
        Err(AppError::MissingCredentials) => Err(AppError::NotFound("no credentials set".into())),
        Err(e) => Err(e),
    }
}

async fn delete_credentials(
    identity: Identity,
    db: web::Data<PgPool>,
    redis: web::Data<deadpool_redis::Pool>,
) -> Result<HttpResponse> {
    sqlx::query!(
        "DELETE FROM credentials WHERE uid = $1",
        identity.claims.sub
    )
    .execute(db.as_ref())
    .await?;

    session::purge(&mut redis.get().await?, identity.claims.sub).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::put().to(save_credentials))
            .route(web::get().to(get_credentials))
            .route(web::delete().to(delete_credentials)),
    );
}
