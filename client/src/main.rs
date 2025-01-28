#![allow(non_snake_case)]

use client::routes::{Home, NotFound};

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    // #[route("/write/:username")]
    // Write { id: String },
    // #[route("/read/:username")]
    // Read { id: String },
    //
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
