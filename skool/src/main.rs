use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};

use dotenv::dotenv;

use skool::routes;
use skool_cookie::CookieConfig;
use tracing::{info, span, Level};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use uuid::Uuid;

struct CustomRootSpanBuilder {}

#[derive(Clone, Copy, Debug)]
struct RequestId(Uuid);

impl RequestId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::ops::Deref for RequestId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &actix_web::dev::ServiceRequest) -> tracing::Span {
        let request_id = RequestId::generate(); // todo: get this from some header

        let span = span!(Level::INFO, "req", id = %request_id);

        {
            let _guard = span.enter();

            let user_agent = request
                .headers()
                .get(header::USER_AGENT)
                .map(|h| h.to_str().ok())
                .flatten()
                .unwrap_or("");

            info!("\"{} {}\" {}", request.method(), request.path(), user_agent);
        }

        span
    }

    fn on_request_end<B>(
        span: tracing::Span,
        outcome: &Result<actix_web::dev::ServiceResponse<B>, actix_web::Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

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
            .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(conf))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
