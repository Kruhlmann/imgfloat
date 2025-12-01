#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::ui::app::GLOBAL_CSS;

pub fn FloatControls() -> Element {
    rsx! {
        div { "class": "float-controls",
            button { "class": "close-button", "X" }
            button { "Upload asset" }
        }
    }
}

#[component]
pub fn Float(channel_name: String) -> Element {
    rsx! {
        document::Stylesheet { href: GLOBAL_CSS }
        document::Stylesheet { href: asset!("/assets/css/float.scss") }
        div { "class": "float-wrapper",
            iframe { src: format!("https://player.twitch.tv/?channel={channel_name}&parent=localhost") }
            div { "class": "cover" }
            canvas { "id": "float-canvas" }
            FloatControls { },
        }
    }
}
