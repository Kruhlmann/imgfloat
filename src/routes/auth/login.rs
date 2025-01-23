use std::sync::Arc;

use axum::{extract::State, response::Redirect};

use crate::{domain::UserSession, twitch::TwitchCredentials};

#[axum::debug_handler]
pub async fn get(
    State(credentials): State<Arc<TwitchCredentials>>,
    session: UserSession,
) -> impl axum::response::IntoResponse {
    if let Some(user) = session.user().await {
        return Redirect::temporary(&format!("/read.html#{}", user.login));
    }
    let auth_url = format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=user:read:email",
        credentials.client_id, credentials.redirect_uri
    );
    tracing::debug!(?auth_url, "serving auth url");
    Redirect::temporary(&auth_url)
}
