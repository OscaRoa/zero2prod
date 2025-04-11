use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use axum::{Form, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::PgPool;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self { name, email })
    }
}

#[allow(clippy::async_yields_async)]
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
    let new_subscriber: NewSubscriber = match form.try_into() {
        Ok(form) => form,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };
    match insert_subscriber(&connection_pool, &new_subscriber).await {
        Ok(_) => Ok(StatusCode::OK.as_str().to_owned()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[tracing::instrument(name = "Saving new subscriber in DB", skip(connection_pool, new_subscriber))]
async fn insert_subscriber(connection_pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v7(Timestamp::now(ContextV7::new())),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
