use actix_web::{
    web::{self, Payload},
    FromRequest, HttpRequest, HttpResponse,
};
use auth1_sdk::Identity;
use tracing::error;

use crate::{
    class,
    credentials::{self, Credentials, PublicCredentials},
    crypt::encrypt_bytes,
    error::AppError,
    session::{self, Session},
    ApiContext, Result,
};

async fn save_credentials(
    identity: Identity,
    creds: web::Json<credentials::Private>,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let creds = creds.into_inner();
    let d = encrypt_bytes(&creds, ctx.aes_key())?;
    let session = Session::create(&creds).await?;
    let mut tx = ctx.postgres.begin().await?;

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
    .fetch_one(&mut tx)
    .await?;

    let mut redis = ctx.redis.get().await?;
    session::save_to_cache(&session, identity.claims.sub, ctx.aes_key(), &mut redis).await?;

    let (school, class) = class::from_session(session).await.map_or_else(
        |e| {
            error!(error = %e, "snoop failed");
            (None, None)
        },
        |c| (Some(c.school), Some(c.reference)),
    );

    tx.commit().await?;

    let creds = PublicCredentials {
        public: creds.into(),
        updated_at: record.updated_at,
        school,
        class,
    };

    Ok(HttpResponse::Created().json(creds))
}

async fn get_credentials(req: HttpRequest, payload: Payload) -> Result<HttpResponse> {
    let mut payload = payload.into_inner();

    match Credentials::from_request(&req, &mut payload).await {
        Ok(c) => Ok(HttpResponse::Ok().json(PublicCredentials::from(c))),
        Err(AppError::MissingCredentials) => Err(AppError::NotFound("no credentials set")),
        Err(e) => Err(e),
    }
}

async fn delete_credentials(
    identity: Identity,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    sqlx::query!(
        "DELETE FROM credentials WHERE uid = $1",
        identity.claims.sub
    )
    .execute(&ctx.postgres)
    .await?;

    session::purge(&mut ctx.redis.get().await?, identity.claims.sub).await?;

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
