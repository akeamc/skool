use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use auth1_sdk::KeyStore;
use clap::Parser;
use dotenv::dotenv;
use skool::{routes, ApiContext, Config};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = Config::parse();

    let _guard = sentry::init(sentry::ClientOptions {
        dsn: config.sentry_dsn.clone(),
        environment: config.sentry_environment.clone().map(Into::into),
        traces_sample_rate: 1.0,
        in_app_include: vec!["skolplattformen"],
        ..Default::default()
    });

    tracing_subscriber::fmt::init();

    let key_store = KeyStore::default();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("migrations failed");

    let redis = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("failed to create redis pool");

    let ctx = web::Data::new(ApiContext {
        postgres: db,
        redis,
        config,
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(sentry_actix::Sentry::new())
            .wrap(cors)
            .app_data(ctx.clone())
            .app_data(key_store.clone())
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
