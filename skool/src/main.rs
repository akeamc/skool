use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use auth1_sdk::KeyStore;
use clap::Parser;
use dotenv::dotenv;
use skool::{routes, Config};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::parse();

    let key_store = KeyStore::default();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");
    let db = web::Data::new(db);

    let config = web::Data::new(config);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(db.clone())
            .app_data(key_store.clone())
            .app_data(config.clone())
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
