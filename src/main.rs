use reqwest::Url;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use zero2prod::email_client::EmailClient;
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

    let state = AppState { db: connection_pool };

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address).await?;

    run(listener, state, email_client).await
}
