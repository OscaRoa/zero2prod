use axum::{Form, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::PgPool;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection_pool),
    fields(
        subscriber_email= %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    State(connection_pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> Result<String, (StatusCode, String)> {
    match insert_subscriber(&connection_pool, &form).await {
        Ok(_) => Ok(StatusCode::OK.as_str().to_owned()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[tracing::instrument(name = "Saving new subscriber in DB", skip(connection_pool, form))]
async fn insert_subscriber(connection_pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v7(Timestamp::now(ContextV7::new())),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
