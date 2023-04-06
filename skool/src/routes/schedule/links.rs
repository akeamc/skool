use actix_web::{
    web::{self, Json, Path},
    HttpResponse,
};
use auth1_sdk::Identity;

use crate::{
    error::AppError,
    share::{self, Link},
    ApiContext, Result,
};

async fn list(identity: Identity, ctx: web::Data<ApiContext>) -> Result<HttpResponse> {
    let links: Vec<Link> = sqlx::query_as("SELECT * FROM links WHERE owner = $1")
        .bind(identity.claims.sub)
        .fetch_all(&ctx.postgres)
        .await?;

    Ok(HttpResponse::Ok().json(links))
}

async fn create(
    identity: Identity,
    Json(options): Json<share::Options>,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
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

    Ok(HttpResponse::Created().json(link))
}

async fn delete(
    identity: Identity,
    path: Path<share::Id>,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
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
        Ok(HttpResponse::NoContent().finish())
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::get().to(list))
            .route(web::post().to(create)),
    )
    .service(web::resource("/{id}").route(web::delete().to(delete)));
}
