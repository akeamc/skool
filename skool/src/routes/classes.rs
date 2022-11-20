use actix_web::{web, HttpResponse};
use auth1_sdk::Identity;

use crate::{
    class::{self, Class},
    session::Session,
    ApiContext, Result,
};

async fn list(
    ident: Identity,
    session: Session,
    ctx: web::Data<ApiContext>,
) -> Result<HttpResponse> {
    let mut tx = ctx.postgres.begin().await?;

    let my_class = class::from_session(session).await?;

    sqlx::query!(
        r#"
          INSERT INTO classes (school, reference, name) VALUES ($1, $2, $3)
          ON CONFLICT ON CONSTRAINT classes_pkey DO UPDATE
            SET name = EXCLUDED.name
        "#,
        my_class.school.as_ref(),
        my_class.reference,
        my_class.name
    )
    .execute(&mut tx)
    .await?;

    sqlx::query!(
        "UPDATE credentials SET (school, class_reference) = ($1, $2) WHERE uid = $3",
        my_class.school.as_ref(),
        my_class.reference,
        ident.claims.sub
    )
    .execute(&mut tx)
    .await?;

    let classes: Vec<Class> =
        sqlx::query_as("SELECT school, reference, name FROM classes WHERE school = $1")
            .bind(my_class.school.as_ref())
            .fetch_all(&mut tx)
            .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(classes))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list)));
}
