pub mod classes;
pub mod credentials;
pub mod schedule;

use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};
use actix_web_opentelemetry::RequestTracing;
use serde::Serialize;

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

pub async fn get_health() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .json(Health::default())
}

fn config_traced(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/schedule").configure(schedule::config))
        .service(web::scope("/credentials").configure(credentials::config))
        .service(web::scope("/classes").configure(classes::config));
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(
            // don't trace health checks
            web::scope("")
                .wrap(RequestTracing::new())
                .configure(config_traced),
        );
}
