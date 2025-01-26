use actix_web::{web, App, HttpServer, Responder, error::ResponseError, http::StatusCode, http::header};
use dotenv::dotenv;
use std::env;
use actix_cors::Cors;
use bollard::{Docker, API_DEFAULT_VERSION, models::{SystemInfo}};
use bollard::container::ListContainersOptions;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::error::Error as StdError;
use lazy_static::lazy_static;
use serde_json;

lazy_static! {
    static ref DOCKER: Docker = {
        let docker_host = std::env::var("DOCKER_HOST").unwrap_or("unix:///var/run/docker.sock".to_string());
        match docker_host.starts_with("unix://") {
            true => Docker::connect_with_socket(
                &docker_host[7..],
                120,
                API_DEFAULT_VERSION,
            ).expect("Failed to connect to Docker socket"),
            false => Docker::connect_with_http(
                &docker_host,
                5,
                API_DEFAULT_VERSION,
            ).expect("Failed to connect to Docker HTTP")
        }
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    message: String,
    docker_info: Option<SystemInfo>,
    containers: Option<serde_json::Value>,
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

async fn hello() -> impl Responder {
    web::Json(ApiResponse {
        message: "Hello from backend!".to_string(),
        docker_info: None,
        containers: None,
    })
}

async fn start_container(id: web::Path<String>) ->  impl Responder{
    println!("starting: {}",id);
    DOCKER.start_container::<String>(&id, None).await.map_err(MyError)?;
    Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse {
        message: format!("Container {} started", id),
        docker_info: None,
        containers: None,
    }))
}

async fn stop_container(id: web::Path<String>) ->  impl Responder{
    println!("stoping:{}",id);
    DOCKER.stop_container(&id, None).await.map_err(MyError)?;
    Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse {
        message: format!("Container {} stopped", id),
        docker_info: None,
        containers: None,
    }))
}

async fn restart_container(id: web::Path<String>) -> Result<impl Responder, actix_web::Error> {
    DOCKER.restart_container(&id, None).await.map_err(MyError)?;
    Ok(web::Json(ApiResponse {
        message: format!("Container {} restarted", id),
        docker_info: None,
        containers: None,
    }))
}

async fn docker_info() -> impl Responder {
    let info = DOCKER.info().await.map_err(MyError)?;
    Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse{
        message: "Docker Info".to_string(),
        docker_info: Some(info),
        containers: None,
    }))
}

async fn get_containers() -> impl Responder {
    let options = ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    };
    
    let containers = DOCKER.list_containers(Some(options)).await.map_err(MyError)?;
    
    Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse{
        message: "Containers List".to_string(),
        docker_info: None,
        containers: Some(serde_json::to_value(containers)?),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
        
    HttpServer::new(|| {
        let cors = Cors::default()
              .allowed_origin("http://127.0.0.1:8080")
              .allowed_methods(vec!["GET", "POST"])
              .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
               .max_age(3600);
        App::new()
            .wrap(cors)
            .route("/", web::get().to(hello))
            .route("/docker_info", web::get().to(docker_info))
            .route("/containers", web::get().to(get_containers))
            .route("/container/{id}/start", web::post().to(start_container))
            .route("/container/{id}/stop", web::post().to(stop_container)) 
            .route("/container/{id}/restart", web::post().to(restart_container))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
