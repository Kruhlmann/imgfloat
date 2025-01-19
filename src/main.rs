use std::sync::Arc;

use dotenvy::dotenv;
use imgfloat::domain::db::{DbService, SqliteDbService};
use imgfloat::domain::{ChannelController, EnvVar};
use imgfloat::twitch::TwitchCredentials;
use tokio::sync::RwLock;
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

    let EnvVar(client_id) = EnvVar::new("TWITCH_CLIENT_ID");
    let EnvVar(client_secret) = EnvVar::new("TWITCH_CLIENT_SECRET");
    let EnvVar(redirect_uri) = EnvVar::new("TWITCH_REDIRECT_URI");
    let EnvVar(database_url) = EnvVar::new("DATABASE_URL");

    let twitch_credentials = TwitchCredentials {
        client_id,
        client_secret,
        redirect_uri,
    };
    let controller = ChannelController::new();
    let (static_dir, not_found_page) = if cfg!(debug_assertions) {
        ("./client", "./client/index.html")
    } else {
        ("/var/www/imgfloat", "/var/www/imgfloat/index.html")
    };
    let database: RwLock<SqliteDbService> = RwLock::new(SqliteDbService::new(&database_url));
    tracing::debug!(?static_dir, ?not_found_page, "static assets");
    imgfloat::run(
        twitch_credentials,
        controller,
        database,
        static_dir,
        not_found_page,
    )
    .await;
}
