// backend/src/main.rs
use actix_web::{web, App, HttpServer, Responder, error::ResponseError, http::StatusCode, http::header};
use actix_cors::Cors;
use bollard::{Docker, API_DEFAULT_VERSION, models::SystemInfo};
use serde::{Serialize, Deserialize};
use std::fmt;
use std::error::Error as StdError;


#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    message: String,
    docker_info: Option<SystemInfo>,
}

async fn hello() -> impl Responder {
    web::Json(ApiResponse {
        message: "Hello from backend!".to_string(),
        docker_info: None,
    })
}
#[derive(Debug)]
struct MyError(bollard::errors::Error);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl StdError for MyError {}

impl ResponseError for MyError {
    fn error_response(&self) -> actix_web::HttpResponse {
         actix_web::HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::json())
            .body(self.to_string())
    }
     fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

}

async fn docker_info() -> impl Responder {
    let docker_host = std::env::var("DOCKER_HOST").unwrap_or("unix:///var/run/docker.sock".to_string());
    let docker = match docker_host.starts_with("unix://") {
        true => {
            Docker::connect_with_socket(
                &docker_host[7..],
                120,
                API_DEFAULT_VERSION,
            ).map_err(MyError)?
        }
        false => {
            Docker::connect_with_http(
                &docker_host,
                5,
                API_DEFAULT_VERSION,
             ).map_err(MyError)?
        }
    };
    let info = docker.info().await.map_err(MyError)?;
    Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse{
        message: "Docker Info".to_string(),
        docker_info: Some(info),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
              .allowed_origin("http://127.0.0.1:8080") // 允许来自前端域的请求
              .allowed_methods(vec!["GET", "POST"]) // 允许的 HTTP 方法
              .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
               .max_age(3600);
        App::new()
            .wrap(cors)
            .route("/", web::get().to(hello))
            .route("/docker_info", web::get().to(docker_info))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}