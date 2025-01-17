#![feature(duration_constructors)]
use std::{net::SocketAddr, sync::Arc};

use axum::{extract::FromRef, routing::get, Router};
use domain::{middleware::log_requests, ApplicationController};
use time::Duration;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};
use twitch::TwitchCredentials;

pub mod domain;
pub mod routes;
pub mod twitch;

#[derive(Clone)]
pub struct AppState {
    controller: Arc<ApplicationController>,
    credentials: Arc<TwitchCredentials>,
}

impl FromRef<AppState> for Arc<ApplicationController> {
    fn from_ref(app_state: &AppState) -> Arc<ApplicationController> {
        Arc::clone(&app_state.controller)
    }
}

impl FromRef<AppState> for Arc<TwitchCredentials> {
    fn from_ref(app_state: &AppState) -> Arc<TwitchCredentials> {
        Arc::clone(&app_state.credentials)
    }
}

pub async fn run(
    twitch_credentials: TwitchCredentials,
    controller: ApplicationController,
    static_dir: &str,
    not_found_page: &str,
) {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));
    let app_state = AppState {
        controller: Arc::new(controller),
        credentials: Arc::new(twitch_credentials),
    };
    let static_dir = ServeDir::new(static_dir).not_found_service(ServeFile::new(not_found_page));
    let app = Router::new()
        .route("/api/whoami", get(routes::api::whoami::get))
        .route("/auth/login", get(routes::auth::login::get))
        .route("/auth/logout", get(routes::auth::logout::get))
        .route("/auth/callback", get(routes::auth::callback::get))
        .route("/ws/read/:username", get(routes::ws::read::get))
        .route("/ws/write/:username", get(routes::ws::write::get))
        .fallback_service(static_dir)
        .with_state(app_state)
        .layer(axum::middleware::from_fn(log_requests))
        .layer(session_layer);
    let address = "0.0.0.0:3000";
    tracing::info!(?address, "binding socket");
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
