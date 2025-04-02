use actix_web::{HttpResponse, get};
use axum::http::StatusCode;

#[get("/health-check")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn ax_health_check() -> StatusCode {
    StatusCode::OK
}
