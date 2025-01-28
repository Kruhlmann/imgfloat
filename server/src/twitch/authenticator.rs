use super::{TwitchApiResponse, TwitchCredentials, TwitchUser, TwitchUserTokens};

#[derive(Debug)]
pub struct TwitchAuthenticatorError(pub String);

#[async_trait::async_trait]
pub trait TwitchAuthenticator: Send + Sync {
    async fn get_tokens(&self, code: &str) -> Result<TwitchUserTokens, TwitchAuthenticatorError>;
    async fn get_user(
        &self,
        tokens: &TwitchUserTokens,
    ) -> Result<TwitchUser, TwitchAuthenticatorError>;

    fn create_auth_url(&self, scope: &str) -> String;
}

pub struct TwitchHttpAuthenticator {
    twitch_oauth_api_url: String,
    twitch_helix_api_url: String,
    credentials: TwitchCredentials,
}

impl TwitchHttpAuthenticator {
    pub fn new(
        twitch_id_api_url: impl Into<String>,
        twitch_helix_api_url: impl Into<String>,
        credentials: TwitchCredentials,
    ) -> Self {
        Self {
            twitch_helix_api_url: twitch_helix_api_url.into(),
            twitch_oauth_api_url: twitch_id_api_url.into(),
            credentials,
        }
    }
}

#[async_trait::async_trait]
impl TwitchAuthenticator for TwitchHttpAuthenticator {
    async fn get_tokens(&self, code: &str) -> Result<TwitchUserTokens, TwitchAuthenticatorError> {
        reqwest::Client::new()
            .post(format!("{}/token", &self.twitch_oauth_api_url))
            .form(&[
                ("client_id", self.credentials.client_id.clone()),
                ("client_secret", self.credentials.client_secret.clone()),
                ("code", code.to_string()),
                ("grant_type", "authorization_code".to_string()),
                ("redirect_uri", self.credentials.redirect_uri.clone()),
            ])
            .send()
            .await
            .inspect_err(|error| tracing::error!(?error, "token request error"))
            .map_err(|error| TwitchAuthenticatorError(error.to_string()))?
            .json::<TwitchUserTokens>()
            .await
            .inspect_err(|error| tracing::error!(?error, "invalid token response"))
            .map_err(|error| TwitchAuthenticatorError(error.to_string()))
    }

    async fn get_user(
        &self,
        tokens: &TwitchUserTokens,
    ) -> Result<TwitchUser, TwitchAuthenticatorError> {
        reqwest::Client::new()
            .get(format!("{}/users", self.twitch_helix_api_url))
            .header("Authorization", format!("Bearer {}", tokens.access_token))
            .header("Client-Id", &self.credentials.client_id)
            .send()
            .await
            .inspect_err(|error| tracing::error!(?error, "user info request error"))
            .map_err(|error| TwitchAuthenticatorError(error.to_string()))?
            .json::<TwitchApiResponse<Vec<TwitchUser>>>()
            .await
            .inspect_err(|error| tracing::error!(?error, "invalid user info response"))
            .map_err(|error| TwitchAuthenticatorError(error.to_string()))?
            .data
            .into_iter()
            .next()
            .ok_or(TwitchAuthenticatorError("No such user".to_string()))
            .inspect_err(|error| tracing::error!(?error, "user not found"))
    }

    fn create_auth_url(&self, scope: &str) -> String {
        format!(
            "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
            self.twitch_oauth_api_url,
            self.credentials.client_id,
            self.credentials.redirect_uri,
            scope
        )
    }
}
