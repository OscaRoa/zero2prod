use crate::domain::{EmailAddress, NewSubscriber, SubscriberName, SubscriptionToken};
use crate::email_client::EmailClient;
use crate::startup::AppState;
use anyhow::Context;
use axum::response::{IntoResponse, Response};
use axum::{Form, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::{Executor, Postgres, Transaction};
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(thiserror::Error)]
pub enum SubscriptionError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for SubscriptionError {
    fn into_response(self) -> Response {
        let status = match self {
            SubscriptionError::ValidationError(e) => {
                tracing::debug!("Validation Error: {e:?}");
                StatusCode::BAD_REQUEST
            }
            SubscriptionError::UnexpectedError(e) => {
                tracing::error!("Unexpected Error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        status.into_response()
    }
}

#[derive(serde::Deserialize)]
pub struct NewSubscriptionForm {
    email: String,
    name: String,
}

impl TryFrom<NewSubscriptionForm> for NewSubscriber {
    type Error = String;

    fn try_from(value: NewSubscriptionForm) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = EmailAddress::parse(value.email)?;

        Ok(Self { name, email })
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, state),
    fields(
        subscriber_email= %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form): Form<NewSubscriptionForm>,
) -> Result<StatusCode, SubscriptionError> {
    let new_subscriber = form.try_into().map_err(SubscriptionError::ValidationError)?;

    let mut transaction = state
        .db
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber in the database")?;

    let subscription_token = SubscriptionToken::new();

    store_token(&mut transaction, subscriber_id, subscription_token.as_ref())
        .await
        .context("Failed to store the confirmation token for a new subscriber")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    send_confirmation_email(
        &state.email_client,
        new_subscriber.email,
        state.base_url,
        subscription_token.as_ref(),
    )
    .await
    .context("Failed to send confirmation email")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Saving new subscriber in DB", skip(transaction, new_subscriber))]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v7(Timestamp::now(ContextV7::new()));

    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );

    transaction.execute(query).await?;

    Ok(subscriber_id)
}

#[tracing::instrument(name = "Sending confirmation email", skip(email_client, subscriber_email))]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber_email: EmailAddress,
    base_url: String,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = &format!("{base_url}/subscriptions/confirm?token={token}");
    let plain_body = format!("Welcome to our newsletter!\nVisit {confirmation_link} to confirm your subscription.");
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{confirmation_link}\">here</a> to confirm your subscription."
    );
    email_client
        .send_email(subscriber_email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    dbg!(subscriber_id, subscription_token);

    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(StoreTokenError)?;

    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database failure was encountered while trying to store a subscription token."
        )
    }
}

pub fn error_chain_fmt(e: &impl std::error::Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
