use crate::domain::{SubscriptionToken, TokenError};
use crate::startup::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct ConfirmParameters {
    token: String,
}

impl TryFrom<Query<ConfirmParameters>> for SubscriptionToken {
    type Error = TokenError;

    fn try_from(value: Query<ConfirmParameters>) -> Result<Self, Self::Error> {
        let token = SubscriptionToken::parse(&value.token)?;

        Ok(Self(token.0))
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, parameters))]
pub async fn confirm(State(state): State<AppState>, parameters: Query<ConfirmParameters>) -> StatusCode {
    let token: SubscriptionToken = match parameters.try_into() {
        Ok(token) => token,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let subscriber_info = match get_subscriber_info_from_token(&state.db, token.as_ref()).await {
        Ok(info) => info,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match subscriber_info {
        // Non-existing token!
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_info) => {
            let status = subscriber_info.1;
            if status == "confirmed" {
                return StatusCode::OK;
            }
            if confirm_subscriber(&state.db, subscriber_info.0).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute subscriptions query: {:?}", e);
        e
    })?;

    sqlx::query!(
        r#"UPDATE subscription_tokens SET status = 'confirmed' WHERE subscriber_id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute subscription_tokens query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_info_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id, status FROM subscription_tokens \
        WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| (r.subscriber_id, r.status)))
}
