#![allow(non_snake_case)]
use dioxus::prelude::*;

#[component]
pub fn PageNotFound(route: Vec<String>) -> Element {
    let msg = format!("Page not found: /{}", route.join("/"));
    rsx! {
        div { "{msg}" }
    }
}
