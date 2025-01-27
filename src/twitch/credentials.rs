#[derive(Clone, Debug)]
pub struct TwitchCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl TwitchCredentials {
    pub fn into_auth_url(&self, scope: impl std::fmt::Display) -> String {
        format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
        self.client_id, self.redirect_uri, scope
    )
    }
}
