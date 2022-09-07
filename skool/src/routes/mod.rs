pub mod credentials;
pub mod schedule;

use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse,
};

async fn get_health() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .body("OK")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(get_health)))
        .service(web::scope("/schedule").configure(schedule::config))
        .service(web::scope("/credentials").configure(credentials::config));
}
