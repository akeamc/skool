use auth1_sdk::Identity;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use crate::{
    class::{self, add_to_class, Class},
    session::Session,
    AppState, Result,
};

async fn list(
    ident: Identity,
    session: Session,
    State(ctx): State<AppState>,
) -> Result<impl IntoResponse> {
    let mut tx = ctx.postgres.begin().await?;

    let my_class = class::from_session(session).await?;

    add_to_class(&my_class, ident.id(), &mut tx).await?;

    let classes: Vec<Class> =
        sqlx::query_as("SELECT school, reference, name FROM classes WHERE school = $1")
            .bind(my_class.school.as_ref())
            .fetch_all(&mut tx)
            .await?;

    tx.commit().await?;

    Ok(Json(classes))
}

pub fn routes() -> Router<AppState> {
    Router::<_>::new().route("/", get(list))
}
