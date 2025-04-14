use axum::extract::MatchedPath;
use axum::http::Request;
use tower_http::trace::MakeSpan;
use tracing::subscriber::set_global_default;
use tracing::{Span, Subscriber, info_span};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};
use uuid::Uuid;

pub fn get_subscriber<Sink>(name: String, env_filter: String, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to set subscriber")
}

#[derive(Clone)]
pub struct MakeSpanWithRequestId;

impl<Body> MakeSpan<Body> for MakeSpanWithRequestId {
    fn make_span(&mut self, request: &Request<Body>) -> Span {
        let matched_path = request.extensions().get::<MatchedPath>().map(MatchedPath::as_str);
        info_span!(
            "http_request",
            method = ?request.method(),
            matched_path,
            request_id = Uuid::new_v4().to_string(),
        )
    }
}
