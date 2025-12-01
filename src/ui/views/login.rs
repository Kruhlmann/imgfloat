#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_sdk::storage::*;

use crate::ui::app::GLOBAL_CSS;
use crate::{data::Username, server::auth::create_oauth_url, ui::components::Backdrop};

#[component]
pub fn Login() -> Element {
    let oauth_url = use_server_future(create_oauth_url)?;
    let session_username = use_singleton_persistent(|| None::<Username>);
    let display_name = use_memo(move || session_username().map(|u| u.to_string()));
    let oauth_url_value: Option<String> = oauth_url().and_then(|res| res.ok());

    let link_destination = match display_name() {
        Some(name) => format!("/float/{name}"),
        None => oauth_url_value.unwrap_or_else(|| "#".to_string()),
    };

    let link_display_text: String = if let Some(name) = display_name() {
        format!("Go to your float")
    } else {
        "Sign in with Twitch".to_string()
    };

    rsx! {
        document::Stylesheet { href: GLOBAL_CSS }
        Backdrop {}
        section { class: "login-wrapper",
            div { class: "inner",
                h1 {
                    match display_name() {
                        Some(name) => rsx!{ "Hi, {name}" },
                        None => rsx!{ "Please sign in" },
                    }
                }
                Link { class: "twitch-sign-in", to: link_destination.clone(), "{link_display_text}" }
            }
        }
    }
}
