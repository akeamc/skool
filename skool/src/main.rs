use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use auth1_sdk::KeyStore;
use clap::Parser;
use dotenv::dotenv;
use skool::{routes, ApiContext, Config};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = Config::parse();

    let _guard = sentry::init(sentry::ClientOptions {
        dsn: config.sentry_dsn.clone(),
        environment: config.sentry_environment.clone().map(Into::into),
        traces_sample_rate: 1.0,
        in_app_include: vec!["skolplattformen"],
        ..Default::default()
    });

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(EnvFilter::from_default_env()))
        .with(sentry_tracing::layer())
        .init();

    let key_store = KeyStore::default();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let redis = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    let ctx = web::Data::new(ApiContext {
        postgres: db,
        redis,
        config,
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(sentry_actix::Sentry::with_transaction())
            .wrap(cors)
            .app_data(ctx.clone())
            .app_data(key_store.clone())
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await?;

    Ok(())
}
