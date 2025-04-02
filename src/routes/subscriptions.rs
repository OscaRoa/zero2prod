use axum::{Form, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(connection_pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> Result<String, (StatusCode, String)> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding a new subscriber", %request_id, subscriber_email=%form.email, subscriber_name=%form.name);

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v7(Timestamp::now(ContextV7::new())),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(&connection_pool)
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("REQUEST ID {request_id}: New subscriber saved");
            Ok(StatusCode::OK.as_str().to_owned())
        }
        Err(e) => {
            tracing::error!("REQUEST ID {request_id}: Failed to execute query: {:?}", e);
            Err(internal_error(e))
        }
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
