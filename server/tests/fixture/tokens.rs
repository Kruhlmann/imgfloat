use imgfloat::twitch::TwitchUserTokens;

pub struct TestTwitchTokens(pub TwitchUserTokens);

impl Default for TestTwitchTokens {
    fn default() -> Self {
        Self(TwitchUserTokens {
            access_token: "".to_string(),
            refresh_token: None,
            expires_in: 0,
            scope: vec![],
            token_type: "".to_string(),
        })
    }
}
