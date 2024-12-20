use std::{env, sync::Arc};

use axum::Router;
use domain::ApplicationController;
use dotenvy::dotenv;
use time::Duration;
use tokio::sync::RwLock;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, cookie::SameSite};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod domain;
mod error;
mod routes;
mod twitch;

pub type SharedState = Arc<RwLock<ApplicationController>>;

pub async fn run() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();
    let client_id = env::var("TWITCH_CLIENT_ID").expect("Client ID missing");
    let client_secret = env::var("TWITCH_CLIENT_SECRET").expect("Client secret missing");
    let redirect_uri = env::var("TWITCH_REDIRECT_URI").expect("Redirect URI missing");
    let controller = ApplicationController::new(client_id, client_secret, redirect_uri);
    let controller_state = Arc::new(RwLock::new(controller));
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let localstate = Arc::clone(&controller_state);
    tokio::spawn(async move {
        loop {
            localstate.read().await.tick().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    let app = Router::new()
        .route(
            "/auth/login",
            axum::routing::get(routes::auth::login::route),
        )
        .route(
            "/auth/callback",
            axum::routing::get(routes::auth::callback::route),
        )
        .with_state(controller_state)
        .layer(session_layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
