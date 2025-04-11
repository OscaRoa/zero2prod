use crate::configuration::{AppState, Settings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use crate::telemetry::MakeSpanWithRequestId;
use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use reqwest::Url;
use sqlx::postgres::PgPoolOptions;
use std::io::Error;
use std::sync::Arc;
use tokio::net::{TcpListener as TokioTcpListener, TcpListener};
use tower_http::trace::TraceLayer;

type App = Serve<TcpListener, Router, Router>;

pub async fn build(configuration: Settings) -> Result<App, Error> {
    let connection_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.connect_options());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");

    let base_url = Url::parse(configuration.email_client.base_url.as_str()).expect("Failed to parse URL");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let state = AppState {
        db: connection_pool,
        email_client: Arc::new(email_client),
    };

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address).await?;

    Ok(run(listener, state))
}
pub fn run(listener: TokioTcpListener, state: AppState) -> App {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(MakeSpanWithRequestId))
        .with_state(state.db)
        .with_state(state.email_client);

    axum::serve(listener, app)
}
