use crate::routes::{ax_health_check, ax_subscribe, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::net::TcpListener;
use tokio::net::TcpListener as TokioTcpListener;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(health_check)
            .service(subscribe)
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn run_axum(_listener: TokioTcpListener, _db_pool: PgPool) {
    let _app = Router::new()
        .route("/health-check", get(ax_health_check))
        .route("/subscriptions", post(ax_subscribe));
}
