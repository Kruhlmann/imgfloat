use std::sync::Arc;

use axum::{extract::State, response::Redirect};

use crate::{domain::UserSession, twitch::TwitchCredentials};

pub async fn get(
    State(credentials): State<Arc<TwitchCredentials>>,
    session: UserSession,
) -> impl axum::response::IntoResponse {
    if let Some(user) = session.user().await {
        return Redirect::temporary(&format!("/read.html#{}", user.login));
    }
    let auth_url = format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=chat:read",
        credentials.client_id, credentials.redirect_uri
    );
    Redirect::temporary(&auth_url)
}
