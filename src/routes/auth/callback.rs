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

// TODO: hide from trace log
#[axum::debug_handler(state = crate::domain::AppState)]
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
    let user = User::new(&user_login);
    let _ = database.write().await.create_user(&user);
    Ok(Redirect::temporary(&format!("/read.html#{}", user_login)))
}
