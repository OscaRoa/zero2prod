use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{
    configuration::{AppState, get_configuration},
    startup::run,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to postgres");

    let state = AppState { db: connection_pool };

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).await?;

    run(listener, state).await
}
