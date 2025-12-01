#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::data::Username;
use crate::server::auth::create_oauth_url;
use crate::ui::views::{Float, Login, PageNotFound, TwitchOauthCallback, TwitchOauthCallbackQuery};

pub static SESSION_USERNAME: GlobalSignal<Option<Username>> = Signal::global(|| None);
pub static GLOBAL_CSS: Asset = asset!("/assets/css/index.scss");

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Login,
    #[route("/callback/twitch?:..query")]
    TwitchOauthCallback { query: TwitchOauthCallbackQuery },
    #[route("/float/:channel_name")]
    Float { channel_name: String },
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(None::<Username>));

    rsx! {
        Router::<Route> {}
    }
}
