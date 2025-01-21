use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, RwLock};

pub struct ChannelController {
    channels: RwLock<HashMap<String, broadcast::Sender<String>>>,
    state_cache: RwLock<HashMap<String, String>>,
}

impl ChannelController {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            state_cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_reader(&self, socket: WebSocket, username: &str) {
        let mut receiver = self
            .channels
            .write()
            .await
            .entry(username.to_owned())
            .or_insert_with(|| broadcast::channel(100).0)
            .subscribe();

        let (mut ws_sender, mut ws_receiver) = socket.split();
        if let Some(message) = self.state_cache.read().await.get(username) {
            if let Err(error) = ws_sender.send(Message::Text(message.clone())).await {
                tracing::error!(?error, ?message, "unable to send cache message");
                return;
            }
        }

        let send_task = tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                let message = Message::Text(msg);
                if let Err(error) = ws_sender.send(message.clone()).await {
                    tracing::error!(?error, ?message, "unable to send state message");
                    break;
                }
            }
        });

        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Close(_) = msg {
                tracing::debug!(?username, "reader socket closed");
                break;
            }
        }

        send_task.abort();
        tracing::debug!(?username, "reader disconnected");
    }

    pub async fn add_writer(&self, mut socket: WebSocket, username: &str) {
        let sender = self
            .channels
            .write()
            .await
            .entry(username.to_string())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone();

        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                Message::Text(text) => {
                    if sender.receiver_count() == 0 {
                        tracing::debug!("skipping broadcast (no readers)");
                    } else if let Err(error) = sender.send(text.clone()) {
                        tracing::error!(?error, "error sending message");
                    } else {
                        tracing::debug!(
                            channel_count = sender.receiver_count(),
                            "propagated message"
                        );
                    }
                    self.state_cache
                        .write()
                        .await
                        .insert(username.to_string(), text);
                }
                Message::Close(_) => {
                    tracing::info!(?username, "writer disconnected");
                    break;
                }
                Message::Binary(_) => tracing::warn!("binary data on writer"),
                Message::Ping(_) | Message::Pong(_) => tracing::trace!(?username, "ping"),
            }
        }

        tracing::debug!(?username, "writer socket closed");
    }
}
