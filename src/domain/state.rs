use std::sync::Arc;

use axum::extract::FromRef;

use crate::twitch::TwitchCredentials;

use super::ChannelController;

#[derive(Clone)]
pub struct AppState {
    controller: Arc<ChannelController>,
    credentials: Arc<TwitchCredentials>,
}

impl AppState {
    pub fn new(controller: Arc<ChannelController>, credentials: Arc<TwitchCredentials>) -> Self {
        Self {
            controller,
            credentials,
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
