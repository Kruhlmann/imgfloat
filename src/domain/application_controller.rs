use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, RwLock};

pub struct ApplicationController {
    channels: RwLock<HashMap<String, broadcast::Sender<String>>>,
}

impl ApplicationController {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_reader(&self, socket: WebSocket, username: &str) {
        let rx = {
            let mut channels = self.channels.write().await;
            channels
                .entry(username.to_owned())
                .or_insert_with(|| broadcast::channel(100).0)
                .subscribe()
        };

        let (mut ws_sender, mut ws_receiver) = socket.split();
        let mut rx_for_loop = rx;
        if let Err(error) = ws_sender.send(Message::Text("ready".to_string())).await {
            tracing::error!(?error, "unable to send ready message");
            return;
        }
        let send_task = tokio::spawn(async move {
            while let Ok(msg) = rx_for_loop.recv().await {
                if ws_sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Close(_) => {
                    tracing::debug!(?username, "reader socket closed");
                    break;
                }
                _ => {}
            }
        }

        send_task.abort();
        tracing::debug!(?username, "reader disconnected");
    }

    pub async fn add_writer(&self, mut socket: WebSocket, username: &str) {
        let tx = {
            let mut channels = self.channels.write().await;
            channels
                .entry(username.to_string())
                .or_insert_with(|| broadcast::channel(100).0)
                .clone()
        };

        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                Message::Text(text) => {
                    if tx.receiver_count() == 0 {
                        tracing::debug!("skipping broadcast (no readers)");
                    } else if let Err(e) = tx.send(text.clone()) {
                        tracing::error!(?e, "error sending message");
                    } else {
                        tracing::debug!(channel_count = tx.receiver_count(), "propagated message");
                    }
                }
                Message::Close(_) => {
                    tracing::info!(?username, "writer disconnected");
                    break;
                }
                Message::Binary(_) => tracing::warn!("binary data on writer"),
                Message::Ping(_) | Message::Pong(_) => {}
            }
        }

        tracing::debug!(?username, "writer socket closed");
    }
}
