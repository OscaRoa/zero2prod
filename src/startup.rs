use crate::configuration::AppState;
use crate::routes::{health_check, subscribe};
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use axum::serve::Serve;
use tokio::net::TcpListener as TokioTcpListener;

pub fn run(listener: TokioTcpListener, State(state): State<AppState>) -> Serve<TokioTcpListener, Router, Router> {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state.db.clone());
    axum::serve(listener, app)
}
