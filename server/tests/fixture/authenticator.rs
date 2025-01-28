use imgfloat::twitch::{
    TwitchAuthenticator, TwitchAuthenticatorError, TwitchUser, TwitchUserTokens,
};

pub struct TestAuthenticator {
    user: Option<TwitchUser>,
    tokens: Option<TwitchUserTokens>,
}

impl TestAuthenticator {
    pub fn new() -> Self {
        Self {
            user: None,
            tokens: None,
        }
    }

    pub fn with_user(mut self, user: TwitchUser) -> Self {
        self.user = Some(user);
        self
    }

    pub fn with_tokens(mut self, tokens: TwitchUserTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }
}

#[async_trait::async_trait]
impl TwitchAuthenticator for TestAuthenticator {
    async fn get_tokens(
        &self,
        _code: &str,
    ) -> Result<TwitchUserTokens, imgfloat::twitch::TwitchAuthenticatorError> {
        self.tokens
            .clone()
            .ok_or(TwitchAuthenticatorError("tokens missing".to_string()))
    }

    async fn get_user(
        &self,
        _tokens: &TwitchUserTokens,
    ) -> Result<TwitchUser, imgfloat::twitch::TwitchAuthenticatorError> {
        self.user
            .clone()
            .ok_or(TwitchAuthenticatorError("tokens missing".to_string()))
    }

    fn create_auth_url(&self, _scope: &str) -> String {
        "".to_string()
    }
}
