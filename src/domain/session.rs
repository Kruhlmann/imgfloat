use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use tower_sessions::Session;

use crate::twitch::{TwitchUser, TwitchUserTokens};

#[derive(Debug)]
pub struct UserSession {
    pub session: Session,
}

pub struct UnlockedUserSession {
    pub user: TwitchUser,
    pub tokens: TwitchUserTokens,
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

    pub async fn destroy(session: &Session) -> Result<(), tower_sessions::session::Error> {
        session.delete().await
    }

    pub async fn is_logged_in(&self) -> bool {
        match (&self.user().await, &self.tokens().await) {
            (Some(_), Some(_)) => true,
            _ => false,
        }
    }

    pub async fn unlock(&self) -> Result<UnlockedUserSession, ()> {
        match (self.user().await, self.tokens().await) {
            (Some(user), Some(tokens)) => Ok(UnlockedUserSession { user, tokens }),
            _ => Err(()),
        }
    }

    pub async fn user(&self) -> Option<TwitchUser> {
        self.session
            .get(Self::SESSION_USER_KEY)
            .await
            .unwrap_or(None)
    }

    pub async fn tokens(&self) -> Option<TwitchUserTokens> {
        self.session
            .get(Self::SESSION_TOKENS_KEY)
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
        let tokens: Option<TwitchUserTokens> =
            session.get(Self::SESSION_TOKENS_KEY).await.unwrap_or(None);

        Self::update_user(&session, user.as_ref())
            .await
            .inspect_err(|error| tracing::error!(?error, "unable to update user session"))
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Unable to store session"))?;
        Self::update_tokens(&session, tokens.as_ref())
            .await
            .inspect_err(|error| tracing::error!(?error, "unable to update session tokens"))
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Unable to store session"))?;

        Ok(Self { session })
    }
}
