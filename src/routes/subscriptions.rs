use crate::domain::{EmailAddress, NewSubscriber, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::AppState;
use axum::{Form, extract::State, http::StatusCode};
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use sqlx::{Executor, Postgres, Transaction};
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
    Form(form): Form<FormData>,
) -> Result<StatusCode, (StatusCode, String)> {
    let new_subscriber: NewSubscriber = match form.try_into() {
        Ok(form) => form,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    let mut transaction = match state.db.begin().await {
        Ok(transaction) => transaction,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };
    let subscription_token = generate_subscription_token();

    match store_token(&mut transaction, subscriber_id, &subscription_token).await {
        Ok(_) => (),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }

    match transaction.commit().await {
        Ok(_) => (),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }

    match send_confirmation_email(
        &state.email_client,
        new_subscriber.email,
        state.base_url,
        &subscription_token,
    )
    .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
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

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

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

fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    dbg!(subscriber_id, subscription_token);

    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
