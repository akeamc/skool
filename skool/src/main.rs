use auth1_sdk::KeyStore;
use axum::Extension;
use clap::Parser;
use dotenv::dotenv;
use opentelemetry::{
    sdk::{propagation::TraceContextPropagator, trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use skool::{routes::app, AppState, Config};
use sqlx::postgres::PgPoolOptions;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_telemetry(otlp_endpoint: impl Into<String>) -> anyhow::Result<()> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace::config().with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                env!("CARGO_PKG_NAME"),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ])))
        .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = Config::parse();

    init_telemetry(config.otlp_endpoint.clone())?;

    let key_store = KeyStore::default();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let redis = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    let ctx = AppState {
        postgres: db,
        redis,
        config,
    };

    let app = app().with_state(ctx.clone()).layer(Extension(key_store));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
