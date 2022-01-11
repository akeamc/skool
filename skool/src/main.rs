use actix_web::{web, App, HttpServer, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

async fn login(info: web::Json<LoginInfo>) -> impl Responder {
    "bruh"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/login", web::post().to(login)))
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
