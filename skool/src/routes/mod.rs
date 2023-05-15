use axum::{response::IntoResponse, routing::get, Json, Router};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::AppState;

pub mod classes;
pub mod credentials;
pub mod schedule;

#[derive(Debug, Serialize)]
struct Health {
    version: &'static str,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub async fn get_health() -> impl IntoResponse {
    ([("cache-control", "no-cache")], Json(Health::default()))
}

// fn config_traced(cfg: &mut web::ServiceConfig) {
//     cfg.service(web::scope("/schedule").configure(schedule::config))
//         .service(web::scope("/credentials").configure(credentials::config))
//         .service(web::scope("/classes").configure(classes::config));
// }

// pub fn config(cfg: &mut web::ServiceConfig) {
//     cfg.service(web::resource("/health").route(web::get().to(get_health)))
//         .service(
//             // don't trace health checks
//             web::scope("")
//                 .wrap(RequestTracing::new())
//                 .configure(config_traced),
//         );
// }

pub fn app() -> Router<AppState> {
    Router::<_>::new()
        .nest("/schedule", schedule::routes())
        .nest("/credentials", credentials::routes())
        .nest("/classes", classes::routes())
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(get_health))
        .layer(CorsLayer::very_permissive())
}
