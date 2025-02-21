use std::{str::FromStr, sync::Arc};
use std::env;

use dioxus::prelude::*;
use reqwest;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeZone, Utc};
// use web_sys::console;
// use dotenv::dotenv;

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
    if let Some(stored_url) = web_sys::window()
    .unwrap()
    .local_storage()
    .unwrap()
    .and_then(|ls| ls.get_item("api_base_url").unwrap()){
        format!("{}{}", stored_url, path)
    }else {
        format!("{}", path)
    }
    // let base_url = env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    // format!("{}{}", base_url, path)
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
    #[route("/docker-info")]
    DockerInfo {},
    #[route("/containers")]
    Containers {},
    #[route("/")]
    #[route("/login")]
    Login {},
    // #[route("/settings")]
    // Settings {}
}

#[component]
fn Settings(base_url_signal: Signal<String>) -> Element {
    rsx! {
        div { class: "settings-container",
            // h2 { "Settings" }
            div { class: "row g-3 align-items-center",
                div{ class: "col-auto",
                    label { "Base URL:" }
                }
                div{ class: "col-auto",
                    input {
                        class: "form-control",
                        r#type: "url",
                        placeholder: "http://localhost:8081",
                        value: "{base_url_signal}",
                        oninput: move |e| base_url_signal.set(e.value().clone())
                    }
                }
                // button { onclick: save_base_url, "Save", class: "btn btn-dark"}
                // if !error().is_empty() {
                //     p { class: "error", "{error}" }
                // }
            }
        }
    }
}


#[component]
fn Login() -> Element {
    let mut base_url_signal = use_signal(|| String::from("http://localhost:8081"));
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| String::new());
    let navigator = use_navigator();

    // Load base URL from local storage on component mount
    use_effect(move || {
        if let Some(stored_url) = web_sys::window()
            .unwrap()
            .local_storage()
            .unwrap()
            .and_then(|ls| ls.get_item("api_base_url").unwrap())
        {
            base_url_signal.set(stored_url);
        }
        // async {}
    });
    


    let mut save_base_url = move |_| {
        let base_url = base_url_signal.to_string();
        if base_url.is_empty() {
            error.set("Base URL cannot be empty".to_string());
        } else {
            web_sys::window()
                .unwrap()
                .local_storage()
                .unwrap()
                .unwrap()
                .set_item("api_base_url", &base_url)
                .unwrap();
            error.set("Base URL saved successfully".to_string());
        }
    };

    let handle_login = move |evt| async move {
        save_base_url(evt);
        let user = User {
            username: username(),
            password: password(),
        };

        match reqwest::Client::new()
            .post(get_api_url("/auth/login"))
            .json(&user)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(login_response) = response.json::<LoginResponse>().await {
                            // 存储token
                            web_sys::window()
                                .unwrap()
                                .local_storage().unwrap().unwrap()
                                .set_item("token", &login_response.token)
                                .unwrap();
                            navigator.push(Route::DockerInfo {});
                        }
                    } else {
                        error.set("Invalid credentials".to_string());
                    }
                }
                Err(_) => error.set("Login failed".to_string()),
        }
    };

    rsx! {
        div { class: "login-container",
            h2 { "Login" }
            Settings {base_url_signal:base_url_signal}
            div { class: "login-form",
                input {
                    class: "form-control",
                    placeholder: "Username",
                    value: "{username}",
                    oninput: move |e| username.set(e.value().clone())
                }
                input {
                    class: "form-control",
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value().clone())
                }
                button { onclick: handle_login, class: "btn btn-primary", "Login" }
                if !error().is_empty() {
                    p { class: "error", "{error}" }
                }
            }
        }
    }
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const CONTAINERS_CSS: Asset = asset!("/assets/styling/containers.css");
// const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    // dotenv().ok();
    dioxus::launch(App);
}

#[component]
fn Navbar() -> Element {
    let mut show_drawer = use_signal(|| true);

    let toggle_drawer = move |_| {
        show_drawer.set(!show_drawer());
    };

    rsx! {
        div {
            id: "navbar",
            button {
                onclick: toggle_drawer,
                "Drawer"
            }
            if show_drawer() {
                div {
                    class: "drawer",
                    Link {
                        to: Route::DockerInfo {},
                        "Docker Info"
                    }
                    Link {
                        to: Route::Containers {},
                        "Containers"
                    }
                    // Link {
                    //     to: Route::Login {},
                    //     "Login"
                    // }
                    // Link {
                    //     to: Route::Settings {},
                    //     "Settings"
                    // }
                }
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: CONTAINERS_CSS }
        document::Link { rel: "stylesheet", href: "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css" }
        document::Link {rel : "stylesheet", href:"https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css"}
        Router::<Route> {}
    }
}

#[component]
pub fn DockerInfo() -> Element {
    let mut contents = use_signal(|| "".to_string());
    let get_docker_info = move |_| async move {
         // 获取token
         let token = web_sys::window()
         .unwrap()
         .local_storage().unwrap().unwrap()
         .get_item("token")
         .unwrap().unwrap();
        let response = reqwest::Client::new().get(get_api_url("/docker_info")).bearer_auth(token).send()
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
            class:"container",
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
}

#[component]
pub fn Containers() -> Element {
    // let mut containers = use_signal(|| None as Option<Vec<Container>>);
    let mut get_containers = use_resource(move|| async move {
        // 获取token
        let token = web_sys::window()
        .unwrap()
        .local_storage().unwrap().unwrap()
        .get_item("token")
        .unwrap().unwrap();
        let  response = reqwest::Client::new().get(get_api_url("/containers")).bearer_auth(token).send()
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

    let start_container = move |id:String| {
            // 获取token
            let token = web_sys::window()
            .unwrap()
            .local_storage().unwrap().unwrap()
            .get_item("token")
            .unwrap().unwrap();
            return async move {
                let _ = reqwest::Client::new()
                    .post(get_api_url(&format!("/container/{}/start", id))).bearer_auth(token)
                    .send()
                    .await;
                get_containers.restart();
        }
    };

    // async {
    //     let a = start_container("aa".to_string());
    //     a.await;
    // };


    let  stop_container = move |id:String| async move {
        // 获取token
        let token = web_sys::window()
        .unwrap()
        .local_storage().unwrap().unwrap()
        .get_item("token")
        .unwrap().unwrap();
        let _ = reqwest::Client::new()
            .post(format!("http://127.0.0.1:8081/container/{}/stop", id)).bearer_auth(token)
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
                                                        onclick: move |_| start_container(c_id.clone()) ,
                                                        id: "button-start",
                                                        class: "btn btn-primary",
                                                        name: "Start",
                                                        i { class: "bi bi-play-fill" }
                                                        " Start"
                                                    },
                                                    button {
                                                        onclick: move |_| stop_container(c_id2.clone()) ,
                                                        class: "btn btn-danger",
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
