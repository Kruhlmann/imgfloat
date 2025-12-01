#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_sdk::storage::*;

use crate::{
    data::Username,
    server::auth::login,
    twitch::TwitchUser,
    ui::{app::SESSION_USERNAME, components::Backdrop},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TwitchOauthCallbackQuery {
    Success {
        code: String,
        scope: String,
    },
    Error {
        error: String,
        error_description: String,
    },
    Invalid,
}
impl std::fmt::Display for TwitchOauthCallbackQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Success { code, scope } => write!(f, "code={code}&scope={scope}"),
            Self::Error {
                error,
                error_description,
            } => write!(f, "error={error}&error_description={error_description}"),
            Self::Invalid => write!(f, ""),
        }
    }
}

impl From<&str> for TwitchOauthCallbackQuery {
    fn from(query: &str) -> Self {
        let mut code = None;
        let mut scope = None;
        let mut error = None;
        let mut error_description = None;
        form_urlencoded::parse(query.as_bytes()).for_each(|(key, value)| {
            if key == "code" {
                code = Some(value.clone().into());
            }
            if key == "scope" {
                scope = Some(value.clone().into());
            }
            if key == "error" {
                error = Some(value.clone().into());
            }
            if key == "error_description" {
                error_description = Some(value.clone().into());
            }
        });

        match (code, scope, error, error_description) {
            (Some(code), Some(scope), _, _) => Self::Success { code, scope },
            (_, _, Some(error), Some(error_description)) => Self::Error {
                error,
                error_description,
            },
            _ => Self::Invalid,
        }
    }
}

impl Default for TwitchOauthCallbackQuery {
    fn default() -> Self {
        Self::Invalid
    }
}

#[component]
pub fn TwitchOauthCallback(query: TwitchOauthCallbackQuery) -> Element {
    let mut session_result: Signal<Option<Result<TwitchUser, ()>>> = use_signal(|| None);
    let mut session_username = use_singleton_persistent(|| None::<Username>);

    rsx! {
        match query {
            TwitchOauthCallbackQuery::Error{ error, error_description }=> rsx!{
                span{"Error: {error} {error_description}"}
            },
            TwitchOauthCallbackQuery::Success { code, .. } => rsx!{
                span {
                    onvisible: move |_| {
                        let value = code.clone();
                        async move {
                            let login_result = login(value)
                                .await
                                .inspect(|user| tracing::info!(?user, "login succeeded"))
                                .inspect_err(|error| tracing::error!(?error, "login"))
                                .map_err(|_| ());
                            *session_result.write() = Some(login_result.clone());
                            *session_username.write() = login_result.map(|r| r.into()).ok()
                        }
                    },
                }
            },
            TwitchOauthCallbackQuery::Invalid => rsx!{span{"Invalid"}},
        }

        document::Stylesheet { href: asset!("/assets/css/twitch_oauth.scss") }
        Backdrop {},
        section { class: "login-wrapper",
            span { "{session_username:?}" }
            div { class: "inner",
                // SAFETY: Bounds-checked by dioxus at runtime
                // https://docs.rs/dioxus/latest/dioxus/prelude/trait.Readable.html#method.read_unchecked
                match &*session_result.read_unchecked() {
                    Some(Ok(user)) => rsx! {
                        Link { to: "/",
                            div { class: "twitch-sign-in",
                                img {src: "{user.profile_image_url}"}
                                span {"Log in as {user.login}"}
                            }
                        }
                    },
                    Some(Err(())) => rsx! {
                        h1 {"Error"}
                        div { class: "twitch-sign-in disabled", "Unable to sign in" }
                    },
                    None => rsx! {
                        h1 {"Logging in "}
                        div { class: "twitch-sign-in disabled", "Please wait" }
                    },
                }
            }
        }
    }
}
