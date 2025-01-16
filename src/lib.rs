#![feature(duration_constructors)]
use std::{net::SocketAddr, sync::Arc};

use axum::{routing::get, Router};
use domain::{middleware::log_requests, ApplicationController};
use time::Duration;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};

mod domain;
mod routes;
mod twitch;

pub async fn run(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    static_dir: &str,
    not_found_page: &str,
) {
    let controller = ApplicationController::new(client_id, client_secret, redirect_uri);
    let controller_state = Arc::new(RwLock::new(controller));
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));
    let localstate = Arc::clone(&controller_state);

    tokio::spawn(async move {
        loop {
            {
                let guard = localstate.read().await;
                guard.tick().await;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    let static_dir = ServeDir::new(static_dir).not_found_service(ServeFile::new(not_found_page));
    let app = Router::new()
        .route("/api/whoami", get(routes::api::whoami::route))
        .route("/auth/login", get(routes::auth::login::route))
        .route("/auth/logout", get(routes::auth::logout::route))
        .route("/auth/callback", get(routes::auth::callback::route))
        .route("/ws/chat/:username", get(routes::ws::chat::route))
        .fallback_service(static_dir)
        .with_state(controller_state)
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
