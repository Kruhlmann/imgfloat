use dotenvy::dotenv;
use imgfloat::domain::db::SqliteDbService;
use imgfloat::domain::{ChannelController, EnvVar};
use imgfloat::twitch::{TwitchAuthenticator, TwitchCredentials, TwitchHttpAuthenticator};
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const LOG_CONFIG: &'static [&'static str] = &[
    "imgfloat=trace",
    "tokio_tungstenite=info",
    "tower_http=info",
    "tower_sessions=info",
    "hyper=info",
    "tungstenite=debug",
    "axum::rejection=trace",
];

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::new(LOG_CONFIG.join(",")))
        .try_init()
        .unwrap();

    if cfg!(debug_assertions) {
        let git_sha = include_str!("../../.git/refs/heads/master").trim();
        tracing::debug!(?git_sha, "version");
    }

    let http_host = EnvVar::new("HTTP_HOST")
        .with_default_value("127.0.0.1")
        .unwrap();
    let http_port = EnvVar::new("HTTP_PORT").with_default_value("3000").unwrap();
    let client_id = EnvVar::new("TWITCH_CLIENT_ID").unwrap();
    let client_secret = EnvVar::new("TWITCH_CLIENT_SECRET").unwrap();
    let redirect_uri = EnvVar::new("TWITCH_REDIRECT_URI").unwrap();
    let database_url = EnvVar::new("DATABASE_URL").ensure_file().unwrap();
    let asset_dir = EnvVar::new("ASSET_DIRECTORY").ensure_directory().unwrap();
    let static_dir = EnvVar::new("STATIC_DIRECTORY").ensure_directory().unwrap();
    let not_found_page = EnvVar::new("NOT_FOUND_PAGE").ensure_file().unwrap();

    let twitch_credentials = TwitchCredentials {
        client_id,
        client_secret,
        redirect_uri,
    };
    let twitch_authenticator: Box<dyn TwitchAuthenticator> =
        Box::new(TwitchHttpAuthenticator::new(
            "https://id.twitch.tv/oauth2",
            "https://api.twitch.tv/helix",
            twitch_credentials,
        ));
    let controller = ChannelController::new();
    let db_service = SqliteDbService::new(&database_url)
        .inspect(|_| tracing::debug!(?database_url, "connected to database"))
        .inspect_err(|error| tracing::error!(?error, "error creating db connection"))
        .unwrap();
    let database: RwLock<SqliteDbService> = RwLock::new(db_service);
    tracing::debug!(?static_dir, ?not_found_page, "static assets");
    tracing::debug!(?asset_dir, "dynamic assets");
    imgfloat::run(
        twitch_authenticator,
        controller,
        database,
        asset_dir,
        static_dir,
        not_found_page,
        http_host,
        http_port,
    )
    .await;
}
