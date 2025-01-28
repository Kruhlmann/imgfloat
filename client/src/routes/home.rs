#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::components::BackDrop;

static CSS: Asset = asset!("/assets/css/index.scss");

#[component]
pub fn Home() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        BackDrop {}
        section { class: "login-wrapper",
            div { class: "inner",
                h1 { "imgfloat" }
                a { class: "twitch-sign-in", href: "/auth/login", "Sign in with Twitch" }
            }
        }
    }
}
