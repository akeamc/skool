pub mod credentials;
pub mod schedule;

use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};
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

async fn get_health() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .json(Health::default())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(web::scope("/schedule").configure(schedule::config))
        .service(web::scope("/credentials").configure(credentials::config));
}
