use std::{error::Error, fmt::Display};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use tower_sessions::Session;

use crate::twitch::{
    AuthCallbackSuccessQuery, TwitchApiResponse, TwitchCredentials, TwitchUser, TwitchUserTokens,
};

#[derive(Debug)]
pub enum UserSessionError {
    TokenRequestFailed(reqwest::Error),
    InvalidTokenResponse(reqwest::Error),
    UserInfoRequestFailed(reqwest::Error),
    InvalidUserInfoResponse(reqwest::Error),
    NoSuchUser,
    SessionUnavailable(tower_sessions::session::Error),
}

impl Error for UserSessionError {}

impl Display for UserSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenRequestFailed(e) => write!(f, "{:?}: {e}", self),
            Self::InvalidTokenResponse(e) => write!(f, "{:?}: {e}", self),
            Self::UserInfoRequestFailed(e) => write!(f, "{:?}: {e}", self),
            Self::InvalidUserInfoResponse(e) => write!(f, "{:?}: {e}", self),
            Self::SessionUnavailable(e) => write!(f, "{:?}: {e}", self),
            Self::NoSuchUser => write!(f, "{:?}", self),
        }
    }
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
        credentials: &TwitchCredentials,
    ) -> Result<String, UserSessionError> {
        let client = reqwest::Client::new();
        let tokens = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", &credentials.client_id),
                ("client_secret", &credentials.client_secret),
                ("code", &query.code),
                ("grant_type", &"authorization_code".to_owned()),
                ("redirect_uri", &credentials.redirect_uri),
            ])
            .send()
            .await
            .inspect_err(|error| tracing::error!(?error, "token request error"))
            .map_err(UserSessionError::TokenRequestFailed)?
            .json::<TwitchUserTokens>()
            .await
            .inspect_err(|error| tracing::error!(?error, "invalid token response"))
            .map_err(UserSessionError::InvalidTokenResponse)?;

        let user = client
            .get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", tokens.access_token))
            .header("Client-Id", &credentials.client_id)
            .send()
            .await
            .inspect_err(|error| tracing::error!(?error, "user info request error"))
            .map_err(UserSessionError::UserInfoRequestFailed)?
            .json::<TwitchApiResponse<Vec<TwitchUser>>>()
            .await
            .inspect_err(|error| tracing::error!(?error, "invalid user info response"))
            .map_err(UserSessionError::InvalidUserInfoResponse)?
            .data
            .into_iter()
            .next()
            .ok_or(UserSessionError::NoSuchUser)
            .inspect_err(|error| tracing::error!(?error, "user not found"))?;

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
