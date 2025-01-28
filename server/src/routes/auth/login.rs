use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Response,
    response::{IntoResponse, Redirect},
};

use crate::{
    domain::UserSession,
    twitch::{TwitchAuthenticator, TwitchUser},
};

#[derive(Debug)]
pub struct LoginRedirect(pub String);

impl LoginRedirect {
    pub fn new(
        user: Option<&TwitchUser>,
        authenticator: Arc<Box<dyn TwitchAuthenticator>>,
    ) -> Self {
        match user {
            Some(u) => Self(format!("/read.html#{}", u.login)),
            None => Self(authenticator.create_auth_url("user:read:email")),
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
    State(authenticator): State<Arc<Box<dyn TwitchAuthenticator>>>,
    session: UserSession,
) -> impl IntoResponse {
    let redirect = LoginRedirect::new(session.user.as_ref(), authenticator);
    tracing::debug!(?redirect, "serving auth url");
    redirect
}
