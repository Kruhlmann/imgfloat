use std::fmt;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use tower_sessions::Session;

use crate::twitch::{TwitchUser, TwitchUserTokens};

pub struct UserSession {
    pub session: Session,
    pub user: Option<TwitchUser>,
    pub tokens: Option<TwitchUserTokens>,
}

impl fmt::Display for UserSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserSession")
            .field("user", &self.user)
            .field("tokens", &"[redacted]")
            .finish()
    }
}

impl UserSession {
    const SESSION_USER_KEY: &'static str = "session.user";
    const SESSION_TOKENS_KEY: &'static str = "session.tokens";

    pub async fn update_user(
        session: &Session,
        user: Option<&TwitchUser>,
    ) -> Result<(), tower_sessions::session::Error> {
        session.insert(Self::SESSION_USER_KEY, user.clone()).await
    }

    pub async fn update_tokens(
        session: &Session,
        tokens: Option<&TwitchUserTokens>,
    ) -> Result<(), tower_sessions::session::Error> {
        session
            .insert(Self::SESSION_TOKENS_KEY, tokens.clone())
            .await
    }

    pub fn is_logged_in(&self) -> bool {
        match (&self.user, &self.tokens) {
            (Some(_), Some(_)) => true,
            _ => false,
        }
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
        let tokens: Option<TwitchUserTokens> =
            session.get(Self::SESSION_TOKENS_KEY).await.unwrap_or(None);

        Self::update_user(&session, user.as_ref())
            .await
            .inspect_err(|e| eprintln!("User session write error: {e}"))
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Unable to store session"))?;
        Self::update_tokens(&session, tokens.as_ref())
            .await
            .inspect_err(|e| eprintln!("Token session write error: {e}"))
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Unable to store session"))?;

        Ok(Self {
            session,
            user,
            tokens,
        })
    }
}
