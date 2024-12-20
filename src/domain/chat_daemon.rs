use std::{error::Error, fmt::Display};

use crate::twitch::{TwitchApiResponse, TwitchUser, TwitchUserTokens};

use super::ApplicationController;

#[derive(Clone)]
pub struct ChatDaemon {
    pub user: TwitchUser,
    pub tokens: TwitchUserTokens,
}

#[derive(Debug)]
pub enum ChatDaemonError {
    TokenRequestFailed(reqwest::Error),
    InvalidTokenResponse(reqwest::Error),
    UserInfoRequestFailed(reqwest::Error),
    InvalidUserInfoResponse(reqwest::Error),
    NoSuchUser,
}

impl Error for ChatDaemonError {}

impl Display for ChatDaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ChatDaemon {
    pub async fn new(
        application_controller: &ApplicationController,
        user_token: &str,
    ) -> Result<Self, ChatDaemonError> {
        let client = reqwest::Client::new();
        let tokens = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", &application_controller.client_id),
                ("client_secret", &application_controller.client_secret),
                ("code", &user_token.to_owned()),
                ("grant_type", &"authorization_code".to_owned()),
                ("redirect_uri", &application_controller.redirect_uri),
            ])
            .send()
            .await
            .map_err(ChatDaemonError::TokenRequestFailed)?
            .error_for_status()
            .map_err(ChatDaemonError::TokenRequestFailed)?
            .json::<TwitchUserTokens>()
            .await
            .map_err(ChatDaemonError::InvalidTokenResponse)?;
        let user = client
            .get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", tokens.access_token))
            .header("Client-Id", &application_controller.client_id)
            .send()
            .await
            .map_err(ChatDaemonError::UserInfoRequestFailed)?
            .error_for_status()
            .map_err(ChatDaemonError::UserInfoRequestFailed)?
            .json::<TwitchApiResponse<Vec<TwitchUser>>>()
            .await
            .map_err(ChatDaemonError::InvalidUserInfoResponse)?
            .data
            .into_iter()
            .next()
            .ok_or(ChatDaemonError::NoSuchUser)?;
        Ok(Self { user, tokens })
    }

    pub async fn tick(&self) {
        println!("Tick on {}", self.user.display_name);
    }
}
