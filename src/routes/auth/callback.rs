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
        .inspect_err(|e| eprintln!("User already has running daemon: {e}"))
        .map_err(|_| Redirect::temporary("/err"))?;
    UserSession::update_user(&session.session, Some(&user))
        .await
        .inspect_err(|e| eprintln!("Unable to update tokens from callback: {e}"))
        .map_err(|_| Redirect::temporary("/err"))?;
    UserSession::update_tokens(&session.session, Some(&tokens))
        .await
        .inspect_err(|e| eprintln!("Unable to update tokens from callback: {e}"))
        .map_err(|_| Redirect::temporary("/err"))?;
    Ok(Redirect::temporary("/settings.html"))
}
