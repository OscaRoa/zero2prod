use actix_web::{post, web, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>, connection_pool: web::Data<PgPool>) -> HttpResponse {
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
        .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => { 
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
