use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, UserSession},
    models::User,
    twitch::{AuthCallbackQuery, TwitchAuthenticator},
};

pub struct AuthCallbackRedirect(pub Redirect);

impl AuthCallbackRedirect {
    pub fn new() -> Self {
        Self(Redirect::temporary("/"))
    }

    pub fn new_with_user(user_login: &str) -> Self {
        Self(Redirect::temporary(&format!("/read.html#{}", user_login)))
    }
}

impl IntoResponse for AuthCallbackRedirect {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

// TODO: hide from trace log
#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn get(
    State(authenticator): State<Arc<Box<dyn TwitchAuthenticator>>>,
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
    query: Query<AuthCallbackQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if query.error.is_some() {
        tracing::error!(query = ?query.as_failure(), "twitch oauth error");
        return Err(AuthCallbackRedirect::new());
    }
    let user_login = UserSession::update(&query.as_success(), &session.session, authenticator)
        .await
        .inspect_err(|_| tracing::warn!(?query, ?session, "failed to update user session"))
        .map_err(|_| AuthCallbackRedirect::new())?;
    if database.read().await.get_user(&user_login).is_none() {
        let user = User::new(&user_login);
        let _ = database.write().await.create_user(&user);
    }
    return Ok(AuthCallbackRedirect::new_with_user(&user_login));
}
