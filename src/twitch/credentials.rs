#[derive(Clone, Debug)]
pub struct TwitchCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}
