use std::env;

use dotenvy::dotenv;
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
    let client_id = env::var("TWITCH_CLIENT_ID").expect("Client ID missing");
    let client_secret = env::var("TWITCH_CLIENT_SECRET").expect("Client secret missing");
    let redirect_uri = env::var("TWITCH_REDIRECT_URI").expect("Redirect URI missing");
    imgfloat::run(client_id, client_secret, redirect_uri).await;
}
