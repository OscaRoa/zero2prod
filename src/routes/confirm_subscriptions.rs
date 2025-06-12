use crate::domain::SubscriptionToken;
use crate::routes::error_chain_fmt;
use crate::startup::AppState;
use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum ConfirmSubscriptionError {
    #[error("{0}")]
    ValidationError(String),

    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmSubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for ConfirmSubscriptionError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            ConfirmSubscriptionError::ValidationError(e) => {
                tracing::debug!("Validation Error: {e:?}");
                StatusCode::BAD_REQUEST
            }
            ConfirmSubscriptionError::UnknownToken => StatusCode::UNAUTHORIZED,
            ConfirmSubscriptionError::UnexpectedError(e) => {
                tracing::error!("Unexpected Error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        status.into_response()
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ConfirmParameters {
    token: String,
}

impl TryFrom<Query<ConfirmParameters>> for SubscriptionToken {
    type Error = String;

    fn try_from(value: Query<ConfirmParameters>) -> Result<Self, Self::Error> {
        let token = SubscriptionToken::parse(&value.token)?;

        Ok(Self(token.0))
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, parameters))]
pub async fn confirm(
    State(state): State<AppState>,
    parameters: Query<ConfirmParameters>,
) -> Result<StatusCode, ConfirmSubscriptionError> {
    let token: SubscriptionToken = parameters
        .try_into()
        .map_err(ConfirmSubscriptionError::ValidationError)?;

    let subscriber_info = get_subscriber_info_from_token(&state.db, token.as_ref())
        .await
        .context("Failed to get subscriber info from token")?
        .ok_or(ConfirmSubscriptionError::UnknownToken)?;

    if subscriber_info.1 == "confirmed" {
        return Ok(StatusCode::OK);
    }

    confirm_subscriber(&state.db, subscriber_info.0)
        .await
        .context("Failed to confirm subscriber")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"UPDATE subscription_tokens SET status = 'confirmed' WHERE subscriber_id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await?;
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
    .await?;

    Ok(result.map(|r| (r.subscriber_id, r.status)))
}
