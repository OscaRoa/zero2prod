use std::net::TcpListener;
use zero2prod::run;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    run(TcpListener::bind("127.0.0.1:8000").unwrap())?.await
}
