use dotenvy::dotenv;
use imgfloat::domain::db::SqliteDbService;
use imgfloat::domain::{ChannelController, EnvVar};
use imgfloat::twitch::TwitchCredentials;
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
        let git_sha = include_str!("../.git/refs/heads/master").trim();
        tracing::debug!(?git_sha, "version");
    }

    let EnvVar(client_id) = EnvVar::new("TWITCH_CLIENT_ID");
    let EnvVar(client_secret) = EnvVar::new("TWITCH_CLIENT_SECRET");
    let EnvVar(redirect_uri) = EnvVar::new("TWITCH_REDIRECT_URI");
    let EnvVar(database_url) = EnvVar::new("DATABASE_URL").ensure_file();
    let EnvVar(asset_dir) = EnvVar::new("ASSET_DIRECTORY").ensure_directory();
    let EnvVar(static_dir) = EnvVar::new("STATIC_DIRECTORY").ensure_directory();
    let EnvVar(not_found_page) = EnvVar::new("NOT_FOUND_PAGE").ensure_file();

    let twitch_credentials = TwitchCredentials {
        client_id,
        client_secret,
        redirect_uri,
    };
    let controller = ChannelController::new();
    let db_service = SqliteDbService::new(&database_url)
        .inspect(|_| tracing::debug!(?database_url, "connected to database"))
        .inspect_err(|error| tracing::error!(?error, "error creating db connection"))
        .unwrap();
    let database: RwLock<SqliteDbService> = RwLock::new(db_service);
    tracing::debug!(?static_dir, ?not_found_page, "static assets");
    tracing::debug!(?asset_dir, "dynamic assets");
    imgfloat::run(
        twitch_credentials,
        controller,
        database,
        asset_dir,
        static_dir,
        not_found_page,
    )
    .await;
}
