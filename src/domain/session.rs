use std::{error::Error, fmt::Display};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use tower_sessions::Session;

use crate::twitch::{AuthCallbackQuery, TwitchApiResponse, TwitchUser, TwitchUserTokens};
use crate::{domain::ApplicationController, twitch::TwitchCredentials};

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

#[derive(Debug)]
pub struct UserSession {
    pub session: Session,
}

impl UserSession {
    const SESSION_USER_KEY: &'static str = "session.user";

    pub async fn update(
        query: &AuthCallbackQuery,
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
            .map_err(UserSessionError::TokenRequestFailed)?
            .error_for_status()
            .map_err(UserSessionError::TokenRequestFailed)?
            .json::<TwitchUserTokens>()
            .await
            .map_err(UserSessionError::InvalidTokenResponse)?;
        let user = client
            .get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", tokens.access_token))
            .header("Client-Id", &credentials.client_id)
            .send()
            .await
            .map_err(UserSessionError::UserInfoRequestFailed)?
            .error_for_status()
            .map_err(UserSessionError::UserInfoRequestFailed)?
            .json::<TwitchApiResponse<Vec<TwitchUser>>>()
            .await
            .map_err(UserSessionError::InvalidUserInfoResponse)?
            .data
            .into_iter()
            .next()
            .ok_or(UserSessionError::NoSuchUser)?;
        tracing::debug!(username = &user.login, "logged user in");
        let user_login = user.login.clone();
        session
            .insert(Self::SESSION_USER_KEY, user)
            .await
            .inspect_err(|error| tracing::error!(?error, "session insert error"))
            .map_err(|error| UserSessionError::SessionUnavailable(error))?;
        Ok(user_login)
    }

    pub async fn update_user(
        session: &Session,
        user: Option<&TwitchUser>,
    ) -> Result<(), tower_sessions::session::Error> {
        session.insert(Self::SESSION_USER_KEY, user.clone()).await
    }

    pub async fn destroy(session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.delete().await
    }

    pub async fn user(&self) -> Option<TwitchUser> {
        self.session
            .get(Self::SESSION_USER_KEY)
            .await
            .unwrap_or(None)
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for UserSession
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;
        let user: Option<TwitchUser> = session.get(Self::SESSION_USER_KEY).await.unwrap_or(None);

        Self::update_user(&session, user.as_ref())
            .await
            .inspect_err(|error| tracing::error!(?error, "unable to update user session"))
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Unable to store session"))?;
        Ok(Self { session })
    }
}
