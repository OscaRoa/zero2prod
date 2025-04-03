use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use zero2prod::{
    configuration::{AppState, get_configuration},
    startup::run,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".to_owned(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to postgres");

    let state = AppState { db: connection_pool };

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).await.unwrap();

    run(listener, state).await
}
