use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Response,
    response::{IntoResponse, Redirect},
};

use crate::{
    domain::{PostloginUrl, UserSession},
    twitch::{TwitchAuthenticator, TwitchUser},
};

#[derive(Debug)]
pub struct LoginRedirect(pub String);

impl LoginRedirect {
    pub fn new(
        user: Option<&TwitchUser>,
        postlogin_url: impl std::fmt::Display,
        authenticator: Arc<Box<dyn TwitchAuthenticator>>,
    ) -> Self {
        match user {
            Some(u) => Self(format!("{}/{}", postlogin_url, u.login)),
            None => Self(authenticator.create_auth_url("user:read:email")),
        }
    }
}

impl IntoResponse for LoginRedirect {
    fn into_response(self) -> Response<Body> {
        Redirect::temporary(&self.0).into_response()
    }
}

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn get(
    State(authenticator): State<Arc<Box<dyn TwitchAuthenticator>>>,
    State(PostloginUrl(postlogin_url)): State<PostloginUrl>,
    session: UserSession,
) -> impl IntoResponse {
    let redirect = LoginRedirect::new(session.user.as_ref(), postlogin_url, authenticator);
    // TODO: redact this
    tracing::debug!(?redirect, "serving auth url");
    redirect
}
