use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use dotenv::dotenv;

use skool::{logging::SkoolRootSpanBuilder, routes, WebhookConfig};
use skool_crypto::CryptoConfig;
use structopt::StructOpt;

use tracing_actix_web::TracingLogger;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    crypto: CryptoConfig,

    #[structopt(flatten)]
    webhook: WebhookConfig,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(TracingLogger::<SkoolRootSpanBuilder>::new())
            .wrap(cors)
            .app_data(web::Data::new(opt.crypto.clone()))
            .app_data(web::Data::new(opt.webhook))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
