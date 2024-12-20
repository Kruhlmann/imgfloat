use std::sync::Arc;

use axum::{extract::State, response::Redirect};
use tokio::sync::RwLock;

use crate::domain::{ApplicationController, UserSession};

pub async fn route(
    State(controller): State<Arc<RwLock<ApplicationController>>>,
    session: UserSession,
) -> impl axum::response::IntoResponse {
    if session.is_logged_in() {
        return Redirect::temporary("google.com");
    }
    let controller_guard = controller.read().await;
    let auth_url = format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=chat:read",
        controller_guard.client_id, controller_guard.redirect_uri
    );
    Redirect::temporary(&auth_url)
}
