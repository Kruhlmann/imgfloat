use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use tower_sessions::Session;

use crate::twitch::{AuthCallbackSuccessQuery, TwitchAuthenticator, TwitchUser};

#[derive(Debug)]
pub enum UserSessionError {
    TokenRequestFailed,
    UserInfoRequestFailed,
    NoSuchUser,
    SessionUnavailable(tower_sessions::session::Error),
}

#[derive(Clone, Debug)]
pub struct UserSession {
    pub session: Session,
    pub user: Option<TwitchUser>,
}

impl UserSession {
    const SESSION_USER_KEY: &'static str = "session.user";

    pub async fn update(
        query: &AuthCallbackSuccessQuery,
        session: &Session,
        authenticator: Arc<Box<dyn TwitchAuthenticator>>,
    ) -> Result<String, UserSessionError> {
        let tokens = authenticator
            .get_tokens(&query.code)
            .await
            .inspect_err(|error| tracing::error!(?error, "token error"))
            .map_err(|_| UserSessionError::TokenRequestFailed)?;
        let user = authenticator
            .get_user(&tokens)
            .await
            .inspect_err(|error| tracing::error!(?error, "user api error"))
            .map_err(|_| UserSessionError::UserInfoRequestFailed)?;

        tracing::debug!(username = &user.login, "logged user in");
        let user_login = user.login.clone();

        session
            .insert(Self::SESSION_USER_KEY, &user)
            .await
            .inspect_err(|error| tracing::error!(?error, "session insert error"))
            .map_err(UserSessionError::SessionUnavailable)?;

        Ok(user_login)
    }

    async fn get_user_from_session(session: &Session) -> Option<TwitchUser> {
        session.get(Self::SESSION_USER_KEY).await.unwrap_or(None)
    }

    pub async fn destroy(session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.delete().await
    }

    pub fn user(&self) -> Option<&TwitchUser> {
        self.user.as_ref()
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for UserSession
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, _state).await?;
        let user = Self::get_user_from_session(&session).await;

        Ok(Self { session, user })
    }
}
