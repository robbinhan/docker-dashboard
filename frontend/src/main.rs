use dioxus::prelude::*;
use reqwest;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    message: String,
    docker_info: Option<serde_json::Value>,
    containers: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/docker-info")]
    DockerInfo {},
    #[route("/containers")]
    Containers {}
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const CONTAINERS_CSS: Asset = asset!("/assets/styling/containers.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: CONTAINERS_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Hero {}
    }
}

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
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
        let response = reqwest::get("http://127.0.0.1:8081/docker_info")
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
    let mut containers = use_signal(|| None as Option<Vec<serde_json::Value>>);
    let get_containers = move |_| async move {
        let response = reqwest::get("http://127.0.0.1:8081/containers")
            .await
            .unwrap()
            .json::<ApiResponse>()
            .await
            .unwrap();

        containers.set(response.containers.map(|c| c.as_array().unwrap().clone()));
    };

    rsx! {
        div {
            class: "container-list",
            h2 { "Docker Containers" }
            button {
                onclick: get_containers,
                "Refresh Containers"
            }
            if let Some(containers) = containers() {
                table {
                    class: "container-table",
                    thead {
                        tr {
                            th { "ID" }
                            th { "Name" }
                            th { "Image" }
                            th { "Status" }
                            // th { "Created" }
                        }
                    }
                    tbody {
                        for container in containers {
                            tr {
                                // td { "{container["Id"].as_str().unwrap_or("").chars().take(12).collect::<String>()}" }
                                // td { "{container["Names"].as_array().unwrap_or(&vec![]).first().unwrap_or(&serde_json::Value::String("".to_string())).as_str().unwrap_or("")}" }
                                // td { "{container["Image"].as_str().unwrap_or("")}" }
                                // td { "{container["Status"].as_str().unwrap_or("")}" }
                                // td { "{container["Created"].as_i64().map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or("".to_string())}" }
                            }
                        }
                    }
                }
            } else {
                p { "No containers found" }
            }
        }
    }
}
