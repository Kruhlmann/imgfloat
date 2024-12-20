use std::collections::HashMap;

use crate::twitch::{TwitchUser, TwitchUserTokens};

use super::{ChatDaemon, ChatDaemonError};

#[derive(Clone)]
pub struct OauthSession;

#[derive(Clone)]
pub struct ApplicationController {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub sessions: HashMap<String, OauthSession>,
    chat_daemons: Vec<ChatDaemon>,
}

impl ApplicationController {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            chat_daemons: vec![],
            sessions: HashMap::new(),
        }
    }

    pub async fn register_chat_daemon(
        &mut self,
        user_token: &str,
    ) -> Result<(TwitchUser, TwitchUserTokens), ChatDaemonError> {
        let daemon = ChatDaemon::new(&self, user_token).await?;
        let user = daemon.user.clone();
        let tokens = daemon.tokens.clone();
        self.chat_daemons.push(daemon);
        Ok((user, tokens))
    }

    pub async fn tick(&self) {
        let tasks = self.chat_daemons.iter().map(|c| c.tick());
        let results: Vec<()> = futures::future::join_all(tasks).await;
        for res in results {
            println!("{:?}", res);
        }
    }

    pub async fn register_user_session(&mut self, login_identity: String) -> Result<(), ()> {
        if self.sessions.contains_key(&login_identity) {
            return Err(());
        }
        self.sessions.insert(login_identity, OauthSession);
        Ok(())
    }

    pub async fn user_has_session(&self, login_identity: String) -> bool {
        self.sessions.contains_key(&login_identity)
    }
}
