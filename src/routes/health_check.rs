use actix_web::{get, HttpResponse};

#[get("/health-check")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}