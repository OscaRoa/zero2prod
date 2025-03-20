use actix_web::{HttpResponse, get};

#[get("/health-check")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
