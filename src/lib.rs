#![feature(duration_constructors)]
#![feature(trivial_bounds)]

#[cfg(feature = "server")]
pub mod domain;
// #[cfg(feature = "server")]
pub mod data;
pub mod models;
pub mod server;
pub mod twitch;
pub mod ui;

pub const COOKIE_SID_NAME: &'static str = "imgfloat-connect-sid";

#[cfg(feature = "server")]
const LOG_CONFIG: &'static [&'static str] = &[
    "imgfloat=trace",
    "tokio_tungstenite=info",
    "tower_http=info",
    "tower_sessions=info",
    "hyper=info",
    "tungstenite=debug",
    "axum::rejection=trace",
];

#[cfg(feature = "server")]
pub fn create_server_state() -> std::sync::Arc<crate::domain::AppState> {
    use domain::ChannelController;
    use domain::EnvVar;
    use domain::SqliteDbService;
    use dotenvy::dotenv;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
    use twitch::TwitchAuthenticator;
    use twitch::TwitchCredentials;
    use twitch::TwitchHttpAuthenticator;

    if cfg!(debug_assertions) {
        let git_sha = include_str!("../.git/refs/heads/master").trim();
        tracing::debug!(?git_sha, "version");
    }

    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::new(LOG_CONFIG.join(",")))
        .try_init()
        .unwrap();

    let client_id = EnvVar::new("TWITCH_CLIENT_ID").unwrap();
    let client_secret = EnvVar::new("TWITCH_CLIENT_SECRET").unwrap();
    let redirect_uri = EnvVar::new("TWITCH_REDIRECT_URI").unwrap();
    let database_url = EnvVar::new("DATABASE_URL").ensure_file().unwrap();
    let asset_dir = EnvVar::new("ASSET_DIRECTORY").ensure_directory().unwrap();
    tracing::debug!(?asset_dir, "dynamic assets");

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
    let database = crate::domain::SqliteDbService::new(&database_url)
        .inspect(|_| tracing::debug!(?database_url, "connected to database"))
        .inspect_err(|error| tracing::error!(?error, "error creating db connection"))
        .unwrap();

    let state = crate::domain::AppState::new(controller, twitch_authenticator, database, asset_dir);
    std::sync::Arc::new(state)
}

#[cfg(feature = "web")]
pub fn create_client_state() -> std::sync::Arc<Option<crate::data::Username>> {
    let username: Option<crate::data::Username> = None;
    std::sync::Arc::new(username)
}
