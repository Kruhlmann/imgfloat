use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use tokio::sync::{mpsc, Mutex};

use crate::twitch::{TwitchUser, TwitchUserTokens};

use super::{chat_daemon::TwitchMessage, ChatDaemon, ChatDaemonError};

#[derive(Clone)]
pub struct ChatSocketCtx {
    socket: Arc<WebSocket>,
    address: Arc<SocketAddr>,
}

#[derive(Clone)]
pub struct ApplicationController {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    connections: Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<TwitchMessage>>>>,
    chat_daemons: Vec<ChatDaemon>,
}

impl ApplicationController {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            chat_daemons: vec![],
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_chat_daemon(
        &mut self,
        user_token: &str,
    ) -> Result<(TwitchUser, TwitchUserTokens), ChatDaemonError> {
        let daemon = ChatDaemon::new(&self, user_token).await?;
        let user = daemon.user.clone();
        let tokens = daemon.tokens.lock().await.clone();
        tracing::info!(?user, "registered new daemon");
        self.chat_daemons.push(daemon);
        Ok((user, tokens))
    }

    pub async fn register_chat_socket(&mut self, mut socket: WebSocket, username: String) {
        let (tx, mut rx) = mpsc::channel::<TwitchMessage>(1024);
        {
            let mut map = self.connections.lock().await;
            map.insert(username.clone(), tx);
        }

        socket
            .send(Message::Text("RDY".to_string()))
            .await
            .inspect_err(|error| tracing::error!(?error, ?username, "socket error"))
            .ok();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let message = serde_json::to_string(&msg).unwrap_or("ERROR".to_string());
                socket
                    .send(Message::Text(message))
                    .await
                    .inspect_err(|error| tracing::error!(?error, ?username, "socket error"))
                    .ok();
            }
        });
    }

    pub async fn tick(&self) {
        let daemons = self.chat_daemons.clone();
        let chat_daemon_tasks: Vec<_> = daemons
            .into_iter()
            .map(|daemon| {
                let connections = Arc::clone(&self.connections);

                tokio::spawn(async move {
                    let map = connections.lock().await;
                    if let Some(sender) = map.get(&daemon.user.login) {
                        for message in daemon.consume_messages().await {
                            let _ = sender.send(message).await;
                        }
                    }
                })
            })
            .collect();

        for handle in chat_daemon_tasks {
            let _ = handle.await;
        }
    }
}
