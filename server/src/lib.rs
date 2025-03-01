#![feature(duration_constructors)]
use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post, put},
    Router,
};
use domain::{db::SqliteDbService, middleware::log_requests, AppState, ChannelController};
use time::Duration;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};
use twitch::TwitchAuthenticator;

pub mod domain;
pub mod models;
pub mod routes;
pub mod twitch;

pub async fn run(
    twitch_authenticator: Box<dyn TwitchAuthenticator>,
    controller: ChannelController,
    database: RwLock<SqliteDbService>,
    asset_dir: String,
    static_dir: String,
    not_found_page: String,
    host: impl std::fmt::Display,
    port: impl std::fmt::Display,
) {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(cfg!(debug_assertions))
        .with_same_site(SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));
    let app_state = AppState::new(
        Arc::new(controller),
        Arc::new(twitch_authenticator),
        Arc::new(database),
        asset_dir,
    );
    let static_dir = ServeDir::new(static_dir).not_found_service(ServeFile::new(not_found_page));
    let app = Router::new()
        .route("/api/whoami", get(routes::api::whoami::get))
        .route("/api/assets/:username", get(routes::api::asset::get))
        .route("/api/assets/:username", post(routes::api::asset::post))
        .route(
            "/api/assets/:username/:filename",
            get(routes::api::asset::file),
        )
        .route("/api/settings", get(routes::api::settings::get))
        .route("/api/settings", put(routes::api::settings::put))
        .route("/auth/login", get(routes::auth::login::get))
        .route("/auth/logout", get(routes::auth::logout::get))
        .route("/auth/callback", get(routes::auth::callback::get))
        .route("/ws/read/:username", get(routes::ws::read::get))
        .route("/ws/write/:username", get(routes::ws::write::get))
        .fallback_service(static_dir)
        .with_state(app_state)
        .layer(axum::middleware::from_fn(log_requests))
        .layer(session_layer);
    let address = format!("{host}:{port}");
    tracing::info!(?address, "binding socket");
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
