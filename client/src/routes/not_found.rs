#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::components::BackDrop;

static CSS: Asset = asset!("/assets/css/index.scss");

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    tracing::warn!(?route, "route not found");
    rsx! {
        document::Stylesheet { href: CSS }
        BackDrop {}
        section { class: "login-wrapper",
            div { class: "inner",
                h1 { "Page not found" }
                p { "We are terribly sorry, but the page you requested doesn't exist." }
            }
        }
    }
}
