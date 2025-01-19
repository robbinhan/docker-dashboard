use std::sync::Arc;

use dioxus::prelude::*;
use reqwest;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use web_sys::console;

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
    let mut containers = use_signal(|| None as Option<Vec<Container>>);
    let get_containers = move |_| async move {
        let response = reqwest::get("http://127.0.0.1:8081/containers")
            .await
            .unwrap()
            .json::<ApiResponse>()
            .await
            .unwrap();


        // match response.containers {
        //     Some(a) => {
        //         console::log_1(&format!("{:?}",a.len()).into());
        //         for aa in a.iter() {
        //             console::log_1(&format!("{:?}",aa.id).into());
                    
        //         }
        //     }
        //     None => todo!()
        // }

            let aaa = response.containers.map(|a| { 
                // c.

            //    let mut a =  c;
            //    console::log_1(&format!("{:?}",a.len()).into());
            //    for aa in a.iter() {
            //     console::log_1(&format!("{:?}",aa.id).into());
            //    }
               
                console::log_1(&format!("{:?}",a).into());

               return a.clone();
            
            });
            containers.set(aaa);

        // containers.set(response.containers.clone().map(|c| c.as_array().unwrap().clone()));
    };

    // containers.read();


    // let mut containers: Resource<Option<Vec<Container>>> = use_resource(|| async move {
    //     reqwest::get("http://127.0.0.1:8081/containers")
    //     .await
    //     .unwrap()
    //     .json::<ApiResponse>()
    //     .await
    //     .unwrap()
    //     .containers

    // });


    rsx! {
        div {
            class: "container-list",
            h2 { "Docker Containers" }
            button {
                onclick: get_containers,
                "Refresh Containers"
            }
            if let Some(ccc) = containers() {
                table {
                    class: "container-table",
                    thead {
                        tr {
                            th { "ID" }
                            th { "Name" }
                            th { "Image" }
                            th { "Status" }
                            th { "Created" }
                        }
                    }
                    tbody {
                        for c in ccc.iter() {
                           
                            tr {
                                td { "{c.id}" }
                                td { "{c.names[0]}" }
                                td { "{c.image}" }
                                td { "{c.status}" }
                                td { "{c.created}" }
                             }
                        }
                    }
                }
            }
            
        }
    }
}
