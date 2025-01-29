use std::sync::Arc;
use actix_web::{body::{BoxBody, MessageBody}, error::ResponseError, http::{header, StatusCode}, middleware::{from_fn, Logger, Next}, web, App, HttpServer, Responder};
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
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::time::{SystemTime, UNIX_EPOCH};
use env_logger::Env;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn create_jwt(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 24 * 3600; // Token有效期24小时

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )
}

fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    )?;
    Ok(token_data.claims)
}

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
    
    // 处理容器数据，添加service字段
    let mut container_data = Vec::new();
    for container in containers {
        let mut container_value = serde_json::to_value(&container)?;
        if let Some(labels) = container.labels {
            if let Some(project) = labels.get("com.docker.compose.project") {
                if let Some(obj) = container_value.as_object_mut() {
                    obj.insert("Service".to_string(), serde_json::Value::String(project.clone()));
                }
            } else {
                if let Some(obj) = container_value.as_object_mut() {
                    obj.insert("Service".to_string(), serde_json::Value::String("Unknown".to_string()));
                }
            }
        }
        container_data.push(container_value);
    }
    
    Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse{
        message: "Containers List".to_string(),
        docker_info: None,
        containers: Some(serde_json::Value::Array(container_data)),
    }))
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    message: String,
}

// async fn register(user: web::Json<User>) -> impl Responder {
//     let hashed = hash(user.password.as_bytes(), DEFAULT_COST).unwrap();
//     // TODO: 在实际应用中，这里应该将用户信息存储到数据库
//     Ok::<web::Json<ApiResponse>, actix_web::Error>(web::Json(ApiResponse {
//         message: "User registered successfully".to_string(),
//         docker_info: None,
//         containers: None,
//     }))
// }

async fn login(user: web::Json<User>) -> impl Responder {
    // TODO: 在实际应用中，这里应该从数据库验证用户信息
    // 这里仅作演示，使用固定的用户名和密码
    if user.username == "admin" && user.password == "password" {
        let token = create_jwt(&user.username).unwrap();
        Ok::<web::Json<LoginResponse>, actix_web::Error>(web::Json(LoginResponse {
            token,
            message: "Login successful".to_string(),
        }))
    } else {
        Err(actix_web::error::ErrorUnauthorized("Invalid credentials"))
    }
}

async fn auth_middleware(req: actix_web::dev::ServiceRequest, next: Next<BoxBody>) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let auth_header = req.headers().get("Authorization");
    match auth_header {
        Some(auth_str) => {
            let auth_str = auth_str.to_str().unwrap();
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                match verify_jwt(token) {
                    Ok(_) => next.call(req).await,
                    Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                }
            } else {
                Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"))
            }
        }
        None => Err(actix_web::error::ErrorUnauthorized("No authorization header"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

        
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            // .expose_headers(&["Authorization"]);

        // let auth = actix_web::middleware::Wrap::new(auth_middleware);

        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(cors)
            .service(
                web::scope("/auth")
                    // .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login))
            )
            .service(
                web::scope("")
                    .wrap(from_fn(auth_middleware))
                    .route("/", web::get().to(hello))
                    .route("/docker_info", web::get().to(docker_info))
                    .route("/containers", web::get().to(get_containers))
                    .route("/container/{id}/start", web::post().to(start_container))
                    .route("/container/{id}/stop", web::post().to(stop_container)) 
                    .route("/container/{id}/restart", web::post().to(restart_container))
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
