use auth1_sdk::Identity;
use axum::{extract::State, response::IntoResponse, routing::put, Json, Router};
use reqwest::StatusCode;
use tracing::error;

use crate::{
    class::{self, add_to_class},
    credentials::{self, Credentials, PublicCredentials},
    crypt::encrypt_bytes,
    error::AppError,
    session::{self, Session},
    AppState, Result,
};

async fn save_credentials(
    identity: Identity,
    State(ctx): State<AppState>,
    Json(creds): Json<credentials::Private>,
) -> Result<impl IntoResponse> {
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
        identity.id(),
        d,
    )
    .fetch_one(&mut tx)
    .await?;

    let mut redis = ctx.redis.get().await?;
    session::save_to_cache(&session, identity.id(), ctx.aes_key(), &mut redis).await?;

    let (school, class) = match class::from_session(session).await {
        Err(e) => {
            error!(error = %e, "snoop failed");
            (None, None)
        }
        Ok(class) => {
            add_to_class(&class, identity.id(), &mut tx).await?;
            (Some(class.school), Some(class.reference))
        }
    };

    tx.commit().await?;

    let creds = PublicCredentials {
        public: creds.into(),
        updated_at: record.updated_at,
        school,
        class,
    };

    Ok((StatusCode::CREATED, Json(creds)))
}

async fn get_credentials(credentials: Result<Credentials>) -> Result<impl IntoResponse> {
    match credentials {
        Ok(c) => Ok((
            [("cache-control", "no-cache")],
            Json(PublicCredentials::from(c)),
        )),
        Err(AppError::MissingCredentials) => Err(AppError::NotFound("no credentials set")),
        Err(e) => Err(e),
    }
}

async fn delete_credentials(
    identity: Identity,
    State(ctx): State<AppState>,
) -> Result<impl IntoResponse> {
    sqlx::query!(
        "DELETE FROM credentials WHERE uid = $1",
        identity.claims.sub
    )
    .execute(&ctx.postgres)
    .await?;

    session::purge(&mut ctx.redis.get().await?, identity.claims.sub).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router<AppState> {
    Router::<_>::new().route(
        "/",
        put(save_credentials)
            .get(get_credentials)
            .delete(delete_credentials),
    )
}
