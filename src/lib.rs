#![allow(non_snake_case)]
use dioxus::prelude::*;

#[cfg(feature = "server")]
pub mod domain;
#[cfg(feature = "server")]
pub mod models;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub mod twitch;

#[component]
pub fn App() -> Element {
    let mut meaning = use_signal(|| None);

    rsx! {
        h1 { "Meaning of life: {meaning:?}" }
        button {
            onclick: move |_| async move {
                if let Ok(data) = get_meaning("life the universe and everything".into()).await {
                    meaning.set(data);
                }
            },
            "Run a server function"
        }
    }
}

#[server]
async fn get_meaning(of: String) -> Result<Option<u32>, ServerFnError> {
    Ok(of.contains("life").then(|| 43))
}
