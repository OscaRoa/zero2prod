use actix_web::{HttpResponse, post, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>, connection_pool: web::Data<PgPool>) -> HttpResponse {
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
    .execute(connection_pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("REQUEST ID {request_id}: New subscriber saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("REQUEST ID {request_id}: Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
