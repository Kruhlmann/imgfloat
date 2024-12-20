#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct TwitchUserTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub scope: Vec<String>,
    pub token_type: String,
}
