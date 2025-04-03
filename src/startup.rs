use crate::configuration::AppState;
use crate::routes::{health_check, subscribe};
use crate::telemetry::get_http_tracing_layer;
use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use tokio::net::TcpListener as TokioTcpListener;

pub fn run(listener: TokioTcpListener, state: AppState) -> Serve<TokioTcpListener, Router, Router> {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(get_http_tracing_layer())
        .with_state(state.db);
    axum::serve(listener, app)
}
