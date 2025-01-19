use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, UserSession},
    models::User,
    twitch::{AuthCallbackQuery, TwitchCredentials},
};

pub async fn get(
    State(credentials): State<Arc<TwitchCredentials>>,
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
    query: Query<AuthCallbackQuery>,
) -> Result<Redirect, Redirect> {
    let user_login = UserSession::update(&query, &session.session, &credentials)
        .await
        .inspect_err(|error| tracing::error!(?error, "unable to register user from auth callback"))
        .map_err(|_| Redirect::temporary("/"))?;
    let _ = database
        .write()
        .await
        .create_user(User::new(user_login.clone()));
    Ok(Redirect::temporary(&format!("/read.html#{}", user_login)))
}
