use auth1_sdk::Identity;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use reqwest::StatusCode;

use crate::{
    error::AppError,
    share::{self, Link},
    AppState, Result,
};

async fn list(identity: Identity, State(ctx): State<AppState>) -> Result<impl IntoResponse> {
    let links: Vec<Link> = sqlx::query_as("SELECT * FROM links WHERE owner = $1")
        .bind(identity.claims.sub)
        .fetch_all(&ctx.postgres)
        .await?;

    Ok(Json(links))
}

async fn create(
    identity: Identity,
    State(ctx): State<AppState>,
    Json(options): Json<share::Options>,
) -> Result<impl IntoResponse> {
    let link = Link::new(options);

    sqlx::query!(
        "INSERT INTO links (owner, id, expires_at, range) VALUES ($1, $2, $3, $4)",
        identity.claims.sub,
        link.id.as_ref(),
        link.options.expires_at,
        link.options.range.as_ref()
    )
    .execute(&ctx.postgres)
    .await?;

    Ok((StatusCode::CREATED, Json(link)))
}

async fn delete_link(
    identity: Identity,
    Path(id): Path<share::Id>,
    State(ctx): State<AppState>,
) -> Result<impl IntoResponse> {
    let res = sqlx::query!(
        "DELETE FROM links WHERE owner = $1 AND id = $2",
        identity.claims.sub,
        id.as_ref()
    )
    .execute(&ctx.postgres)
    .await?;

    if res.rows_affected() == 0 {
        Err(AppError::NotFound("link not found"))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub fn routes() -> Router<AppState> {
    Router::<_>::new()
        .route("/", get(list).post(create))
        .route("/{id}", delete(delete_link))
}
