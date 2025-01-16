use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use tokio::sync::RwLock;

use crate::{
    domain::{ApplicationController, UserSession},
    twitch::AuthCallbackQuery,
};

pub async fn route(
    State(controller): State<Arc<RwLock<ApplicationController>>>,
    session: UserSession,
    query: Query<AuthCallbackQuery>,
) -> Result<Redirect, Redirect> {
    let mut controller_guard = controller.write().await;
    let (user, tokens) = controller_guard
        .register_chat_daemon(&query.code)
        .await
        .inspect_err(|error| tracing::error!(?error, "unable to register chat daemon"))
        .map_err(|_| Redirect::temporary("/"))?;
    UserSession::update_user(&session.session, Some(&user))
        .await
        .inspect_err(|error| tracing::error!(?error, "unable to update user session"))
        .map_err(|_| Redirect::temporary("/"))?;
    UserSession::update_tokens(&session.session, Some(&tokens))
        .await
        .inspect_err(|error| tracing::error!(?error, "unable to update user tokens"))
        .map_err(|_| Redirect::temporary("/"))?;
    tracing::debug!(username = &user.login, "logged user in");
    Ok(Redirect::temporary(&format!("/float.html#{}", user.login)))
}
