use std::sync::Arc;

use axum::extract::FromRef;
use tokio::sync::RwLock;

use crate::twitch::TwitchCredentials;

use super::{db::SqliteDbService, ChannelController};

#[derive(Clone)]
pub struct AssetDirectory(pub String);

#[derive(Clone)]
pub struct AppState {
    controller: Arc<ChannelController>,
    credentials: Arc<TwitchCredentials>,
    database: Arc<RwLock<SqliteDbService>>,
    asset_dir: AssetDirectory,
}

impl AppState {
    pub fn new(
        controller: Arc<ChannelController>,
        credentials: Arc<TwitchCredentials>,
        database: Arc<RwLock<SqliteDbService>>,
        asset_dir: String,
    ) -> Self {
        Self {
            controller,
            credentials,
            database,
            asset_dir: AssetDirectory(asset_dir),
        }
    }
}

impl FromRef<AppState> for Arc<ChannelController> {
    fn from_ref(app_state: &AppState) -> Arc<ChannelController> {
        Arc::clone(&app_state.controller)
    }
}

impl FromRef<AppState> for Arc<TwitchCredentials> {
    fn from_ref(app_state: &AppState) -> Arc<TwitchCredentials> {
        Arc::clone(&app_state.credentials)
    }
}

impl FromRef<AppState> for Arc<RwLock<SqliteDbService>> {
    fn from_ref(app_state: &AppState) -> Arc<RwLock<SqliteDbService>> {
        Arc::clone(&app_state.database)
    }
}

impl FromRef<AppState> for AssetDirectory {
    fn from_ref(app_state: &AppState) -> AssetDirectory {
        app_state.asset_dir.clone()
    }
}
