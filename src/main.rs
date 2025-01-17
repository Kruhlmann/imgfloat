use std::env;

use dotenvy::dotenv;
use imgfloat::domain::ChannelController;
use imgfloat::twitch::TwitchCredentials;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::new(
            "imgfloat=trace,tokio_tungstenite=info,tower_http=info,tower_sessions=info,hyper=info,tungstenite=debug,axum::rejection=trace",
        ))
        .try_init()
        .unwrap();
    let git_sha = include_str!("../.git/refs/heads/master").trim();
    tracing::debug!(?git_sha, "version");
    let twitch_credentials = TwitchCredentials {
        client_id: env::var("TWITCH_CLIENT_ID").expect("Client ID missing"),
        client_secret: env::var("TWITCH_CLIENT_SECRET").expect("Client secret missing"),
        redirect_uri: env::var("TWITCH_REDIRECT_URI").expect("Redirect URI missing"),
    };
    let controller = ChannelController::new();
    let (static_dir, not_found_page) = if cfg!(debug_assertions) {
        ("./client", "./client/index.html")
    } else {
        ("/var/www/imgfloat", "/var/www/imgfloat/index.html")
    };
    tracing::debug!(?static_dir, ?not_found_page, "static assets");
    imgfloat::run(twitch_credentials, controller, static_dir, not_found_page).await;
}
