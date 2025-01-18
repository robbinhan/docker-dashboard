// backend/src/main.rs
use actix_web::{web, App, HttpServer, Responder};
use bollard::{Docker, API_DEFAULT_VERSION};

async fn hello() -> impl Responder {
    "Hello from backend!".to_string()
}

async fn docker_info() -> impl Responder {
    let docker = Docker::connect_with_socket("/var/run/docker.sock",120,API_DEFAULT_VERSION).unwrap();
    let info = docker.info().await.unwrap();

    format!("Docker info: {:?}", info).to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/docker_info", web::get().to(docker_info))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}