use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Response,
    response::{IntoResponse, Redirect},
};

use crate::{
    domain::UserSession,
    twitch::{TwitchCredentials, TwitchUser},
};

#[derive(Debug)]
pub struct LoginRedirect(pub String);

impl LoginRedirect {
    pub fn new(user: Option<&TwitchUser>, credentials: Arc<TwitchCredentials>) -> Self {
        match user {
            Some(u) => Self(format!("/read.html#{}", u.login)),
            None => Self(credentials.into_auth_url("user:read:email")),
        }
    }
}

impl IntoResponse for LoginRedirect {
    fn into_response(self) -> Response<Body> {
        Redirect::temporary(&self.0).into_response()
    }
}

#[axum::debug_handler]
pub async fn get(
    State(credentials): State<Arc<TwitchCredentials>>,
    session: UserSession,
) -> impl IntoResponse {
    let redirect = LoginRedirect::new(session.user.as_ref(), credentials);
    tracing::debug!(?redirect, "serving auth url");
    redirect
}
