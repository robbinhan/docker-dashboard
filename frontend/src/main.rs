use std::sync::Arc;
use std::env;

use dioxus::prelude::*;
use reqwest;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeZone, Utc};
use web_sys::console;
use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    message: String,
}

fn get_api_url(path: &str) -> String {
    let base_url = env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    format!("{}{}", base_url, path)
}

#[derive(Serialize, Deserialize, Debug,Clone)]
struct Container {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Names")]
    names: Vec<String>,
    #[serde(rename = "Image")]
    image: String,
    #[serde(rename = "Status")]
    status: String,
     #[serde(rename = "Created")]
    created: i64,
    #[serde(rename = "Service")]
    service: String,
    // Add more fields according to your JSON structure

}


#[derive(Serialize, Deserialize, Debug,Clone)]
struct ApiResponse {
    message: String,
    docker_info: Option<serde_json::Value>,
    // containers: Option<serde_json::Value>,
    containers: Option<Vec<Container>>,
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    #[route("/docker-info")]
    DockerInfo {},
    #[route("/containers")]
    Containers {},
    // #[route("/login")]
    // Login {}
}

// #[component]
// fn Login() -> Element {
//     let username = use_signal(|| String::new());
//     let password = use_signal(|| String::new());
//     let error = use_signal(|| String::new());
//     let navigator = use_navigator();

//     let handle_login = move |_| async move {
//         let user = User {
//             username: username.get().to_string(),
//             password: password.get().to_string(),
//         };

//         match reqwest::Client::new()
//             .post(get_api_url("/auth/login"))
//             .json(&user)
//             .send()
//             .await {
//                 Ok(response) => {
//                     if response.status().is_success() {
//                         if let Ok(login_response) = response.json::<LoginResponse>().await {
//                             // 存储token
//                             web_sys::window()
//                                 .unwrap()
//                                 .local_storage()
//                                 .unwrap()
//                                 .unwrap()
//                                 .set_item("token", &login_response.token)
//                                 .unwrap();
//                             navigator.push(Route::DockerInfo {});
//                         }
//                     } else {
//                         error.set("Invalid credentials".to_string());
//                     }
//                 }
//                 Err(_) => error.set("Login failed".to_string()),
//         }
//     };

//     rsx! {
//         div { class: "login-container",
//             h2 { "Login" }
//             div { class: "login-form",
//                 input {
//                     placeholder: "Username",
//                     value: username.get(),
//                     oninput: move |e| username.set(e.value.clone())
//                 }
//                 input {
//                     r#type: "password",
//                     placeholder: "Password",
//                     value: password.get(),
//                     oninput: move |e| password.set(e.value.clone())
//                 }
//                 button { onclick: handle_login, "Login" }
//                 if !error.get().is_empty() {
//                     p { class: "error", "{error}" }
//                 }
//             }
//         }
//     }
// }

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const CONTAINERS_CSS: Asset = asset!("/assets/styling/containers.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dotenv().ok();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: CONTAINERS_CSS }
        document::Link { rel: "stylesheet", href: "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css" }
        Router::<Route> {}
    }
}



#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::DockerInfo {},
                "Docker Info"
            }
            Link {
                to: Route::Containers {},
                "Containers"
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
pub fn DockerInfo() -> Element {
    let mut contents = use_signal(|| "".to_string());
    let get_docker_info = move |_| async move {
        let response = reqwest::get(get_api_url("/docker_info"))
            .await
            .unwrap()
            .json::<ApiResponse>()
            .await
            .unwrap();

        let message = match &response.docker_info {
            Some(info) => format!("{}", serde_json::to_string_pretty(info).unwrap_or("None".to_string())),
            None => "None".to_string()
        };
        contents.set(message);
    };

    rsx! {
        div {
            "Docker Info: "
            pre {
                "{contents}"
            }
        }
        button {
            onclick: get_docker_info,
            "Get Docker Info"
        }
    }
}

#[component]
pub fn Containers() -> Element {
    // let mut containers = use_signal(|| None as Option<Vec<Container>>);
    let mut get_containers = use_resource(move|| async move {
        let  response = reqwest::get(get_api_url("/containers"))
            .await
            .unwrap()
            .json::<ApiResponse>()
            .await
            .unwrap();

            let aaa = response.containers.map(|a| { 
                return a.iter().map(|x| {
                    // let datetime: DateTime<Utc> = DateTime::from_timestamp(x.created, 0).unwrap();
                    return Container{
                    id: x.id.chars().take(12).collect::<String>(),
                    // created_datetime: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                    ..x.clone()
                    };
                }).collect::<Vec<Container>>();
                
            });
            // containers.set(aaa);
            return aaa;
    });

    // for bb in get_containers.read_unchecked().as_ref().unwrap().iter() {
    //     for bbb in bb.iter() {
    //         println!("{}",bbb.id);
    //     }
    // }

    let  start_container = move |id:String| async move {
        let _ = reqwest::Client::new()
            .post(get_api_url(&format!("/container/{}/start", id)))
            .send()
            .await;
        get_containers.restart();
    };

    let  stop_container = move |id:String| async move {
        let _ = reqwest::Client::new()
            .post(format!("http://127.0.0.1:8081/container/{}/stop", id))
            .send()
            .await;
        get_containers.restart();
    };

    rsx! {
        div {
            class: "container-list",
            h2 { "Docker Containers" }

            match &*get_containers.read_unchecked() {
                Some(ccc) => rsx! {
                    table {
                        class: "container-table",
                        thead {
                            tr {
                                th { "ID" }
                                th { "Service" }
                                th { "Name" }
                                th { "Image" }
                                th { "Status" }
                                th { "Created" }
                                th { "Operater" }
                            }
                        }
                        tbody {
                            for c in ccc.as_ref().unwrap().iter() {
                                {
                                    let  c_id = c.id.clone();
                                    let  c_id2 = c.id.clone();
                                    let datetime: DateTime<Utc> = DateTime::from_timestamp(c.created, 0).unwrap();
                                    let created_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                                    // let  c_id3 = c.id.clone();
                                    rsx! {
                                        tr {
                                            td { "{c.id}" }
                                            td { "{c.service}" }
                                            td { "{c.names[0]}" }
                                            td { "{c.image}" }
                                            td { "{c.status}" }
                                            td { "{created_datetime}" }
                                            td { 
                                                div { class: "operation-buttons",
                                                    button { 
                                                        onclick: move |_| start_container(c_id.to_string()) ,
                                                        id: "button-start",
                                                        class: "operation-button",
                                                        name: "Start",
                                                        i { class: "bi bi-play-fill" }
                                                        " Start"
                                                    },
                                                    button { 
                                                        onclick: move |_| stop_container(c_id2.to_string()) ,
                                                        class: "operation-button",
                                                        name: "Stop",
                                                        i { class: "bi bi-stop-fill" }
                                                        " Stop"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { "Loading dogs..." }
                },
            }
        }
    }
}
