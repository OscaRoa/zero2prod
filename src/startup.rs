use crate::configuration::AppState;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use crate::telemetry::MakeSpanWithRequestId;
use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use tokio::net::TcpListener as TokioTcpListener;
use tower_http::trace::TraceLayer;

pub fn run(
    listener: TokioTcpListener,
    state: AppState,
    email_client: EmailClient,
) -> Serve<TokioTcpListener, Router, Router> {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(MakeSpanWithRequestId))
        .with_state(state.db)
        .with_state(email_client);

    axum::serve(listener, app)
}
