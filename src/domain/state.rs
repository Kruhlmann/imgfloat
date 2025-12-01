use super::{ChannelController, SqliteDbService};
use crate::twitch::TwitchAuthenticator;

#[derive(Clone, Debug)]
pub struct PostloginUrl(pub String);

#[derive(Clone, Debug)]
pub struct AssetDirectory(pub String);

pub struct AppState {
    pub controller: ChannelController,
    pub twitch_authenticator: Box<dyn TwitchAuthenticator>,
    pub database: SqliteDbService,
    pub asset_dir: AssetDirectory,
}

impl AppState {
    pub fn new(
        controller: ChannelController,
        twitch_authenticator: Box<dyn TwitchAuthenticator>,
        database: SqliteDbService,
        asset_dir: String,
    ) -> Self {
        Self {
            controller,
            twitch_authenticator,
            database,
            asset_dir: AssetDirectory(asset_dir),
        }
    }
}
