use crate::configuration::AppState;
use crate::routes::{health_check, subscribe};
use crate::telemetry::MakeSpanWithRequestId;
use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use tokio::net::TcpListener as TokioTcpListener;
use tower_http::trace::TraceLayer;

pub fn run(listener: TokioTcpListener, state: AppState) -> Serve<TokioTcpListener, Router, Router> {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(MakeSpanWithRequestId))
        .with_state(state.db);
    axum::serve(listener, app)
}
