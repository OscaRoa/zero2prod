pub mod configuration;
pub mod routes;
pub mod startup;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;
use routes::{health_check, subscribe};


pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(health_check).service(subscribe))
        .listen(listener)?
        .run();

    Ok(server)
}
