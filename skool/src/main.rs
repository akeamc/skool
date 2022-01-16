use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};

use dotenv::dotenv;

use skool::routes;
use skool_cookie::CookieConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let conf = CookieConfig {
        key: *b"bruhbruhbruhbruhbruhbruhbruhbruh",
    };

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(conf))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
