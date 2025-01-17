use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Redirect,
};

use crate::{
    domain::UserSession,
    twitch::{AuthCallbackQuery, TwitchCredentials},
};

pub async fn get(
    State(credentials): State<Arc<TwitchCredentials>>,
    session: UserSession,
    query: Query<AuthCallbackQuery>,
) -> Result<Redirect, Redirect> {
    let user_login = UserSession::update(&query, &session.session, &credentials)
        .await
        .inspect_err(|error| tracing::error!(?error, "unable to register user from auth callback"))
        .map_err(|_| Redirect::temporary("/"))?;
    Ok(Redirect::temporary(&format!("/read.html#{}", user_login)))
}
