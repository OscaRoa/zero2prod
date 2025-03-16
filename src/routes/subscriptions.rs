use actix_web::post;
use crate::HttpResponse;
use crate::web;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
