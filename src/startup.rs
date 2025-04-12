use crate::configuration::{AppState, DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, subscribe};
use crate::telemetry::MakeSpanWithRequestId;
use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use reqwest::Url;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::io::Error;
use std::sync::Arc;
use tokio::net::{TcpListener as TokioTcpListener, TcpListener};
use tower_http::trace::TraceLayer;

type AppServer = Serve<TcpListener, Router, Router>;

pub struct Application {
    port: u16,
    server: AppServer,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Application, Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let email_provider_url = Url::parse(configuration.email_client.base_url.as_str()).expect("Failed to parse URL");
        let email_client_timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            email_provider_url,
            sender_email,
            configuration.email_client.authorization_token,
            email_client_timeout,
        );

        let address = format!("{}:{}", configuration.application.host, configuration.application.port);
        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr()?.port();

        let state = AppState {
            db: connection_pool,
            email_client: Arc::new(email_client),
            base_url: configuration.application.base_url,
        };
        let server = run(listener, state);

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(db_settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(db_settings.connect_options())
}

fn run(listener: TokioTcpListener, state: AppState) -> AppServer {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions/confirm", get(confirm))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(MakeSpanWithRequestId))
        .with_state(state);

    axum::serve(listener, app)
}
