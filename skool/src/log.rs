use actix_web::{dev::ServiceResponse, http::header, ResponseError};
use reqwest::StatusCode;

use tracing::{field, info, span, Level, Span};
use tracing_actix_web::RootSpanBuilder;
use uuid::Uuid;

pub struct SkoolRootSpanBuilder {}

#[derive(Clone, Copy, Debug)]
struct RequestId(Uuid);

impl Default for RequestId {
    fn default() -> Self {
        Self::generate()
    }
}

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

impl RootSpanBuilder for SkoolRootSpanBuilder {
    fn on_request_start(request: &actix_web::dev::ServiceRequest) -> tracing::Span {
        let user_agent = request
            .headers()
            .get(header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        let span = span!(
          Level::INFO,
          "req",
          request_id = %RequestId::generate(),
          trace_id = field::Empty,
          http.method = %request.method(),
          http.path = request.path(),
          http.status_code = field::Empty,
          http.user_agent = user_agent,
          otel.kind = "server",
          otel.status_code = field::Empty,
          exception.message = field::Empty,
          exception.details = field::Empty
        );

        if let Some(trace_id) = request
            .headers()
            .get("ot-tracer-traceid")
            .and_then(|v| v.to_str().ok())
        {
            span.record("trace_id", &trace_id);
        }

        span
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, actix_web::Error>) {
        match &outcome {
            Ok(response) => {
                if let Some(error) = response.response().error() {
                    handle_error(&span, response.status(), error.as_response_error())
                } else {
                    let code: i32 = response.response().status().as_u16().into();
                    span.record("http.status_code", &code);
                    span.record("otel.status_code", &"OK");
                }
            }
            Err(error) => {
                let response_error = error.as_response_error();
                handle_error(&span, response_error.status_code(), response_error);
            }
        }

        let _guard = span.enter();

        info!("sent response");
    }
}

fn handle_error(span: &Span, status_code: StatusCode, response_error: &dyn ResponseError) {
    let display = format!("{}", response_error);
    let debug = format!("{:?}", response_error);
    span.record("exception.message", &tracing::field::display(display));
    span.record("exception.details", &tracing::field::display(debug));
    let code: i32 = status_code.as_u16().into();

    span.record("http.status_code", &code);

    if status_code.is_client_error() {
        span.record("otel.status_code", &"OK");
    } else {
        span.record("otel.status_code", &"ERROR");
    }
}
