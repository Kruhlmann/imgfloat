use std::{net::SocketAddr, sync::Arc};

use axum::{routing::get, Router};
use domain::ApplicationController;
use time::Duration;
use tokio::sync::RwLock;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};

mod domain;
mod routes;
mod twitch;

pub async fn run(client_id: String, client_secret: String, redirect_uri: String) {
    let controller = ApplicationController::new(client_id, client_secret, redirect_uri);
    let controller_state = Arc::new(RwLock::new(controller));
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));
    let tracing_layer =
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true));
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

    let static_dir =
        ServeDir::new("src/client").not_found_service(ServeFile::new("src/client/index.html"));
    let app = Router::new()
        .route("/auth/login", get(routes::auth::login::route))
        .route("/auth/logout", get(routes::auth::logout::route))
        .route("/auth/callback", get(routes::auth::callback::route))
        .route("/whoami", get(routes::whoami::route))
        .route("/ws/chat/:username", get(routes::ws::chat::route))
        .fallback_service(static_dir)
        .with_state(controller_state)
        .layer(session_layer)
        .layer(tracing_layer);
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
