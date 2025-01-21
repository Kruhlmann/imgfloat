use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, RwLock};

use crate::domain::message::ImgfloatAssetStateMessage;

use super::message::ImgfloatState;

pub struct ChannelController {
    channels: RwLock<HashMap<String, broadcast::Sender<String>>>,
    state_cache: RwLock<HashMap<String, ImgfloatState>>,
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
        if let Some(current_state) = self.state_cache.read().await.get(username) {
            tracing::debug!(?username, ?current_state, "serving local state");
            let state_message = match serde_json::to_string(current_state)
                .inspect_err(|error| tracing::error!(?error, "state could not be serialized"))
                .map(|state| Message::Text(state.clone()))
            {
                Ok(m) => m,
                Err(_) => return,
            };
            if let Err(error) = ws_sender.send(state_message).await {
                tracing::error!(?error, ?current_state, "unable to send cache message");
                return;
            }
        } else {
            tracing::info!(?username, "no cached state available");
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

        if let Some(state) = self.state_cache.read().await.get(username) {
            tracing::info!(?username, ?state, "sending cache to writer");
            match serde_json::to_string(state) {
                Ok(json_str) => {
                    socket
                        .send(Message::Text(json_str))
                        .await
                        .inspect_err(|error| {
                            tracing::error!(
                                ?username,
                                ?state,
                                ?error,
                                "unable to send initial state"
                            )
                        })
                        .ok();
                }
                Err(error) => {
                    tracing::error!(
                        ?username,
                        ?state,
                        ?error,
                        "unable to serialize initial state"
                    )
                }
            };
        } else {
            tracing::info!(?username, "no cached state available");
        }

        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                Message::Text(state_str) => {
                    if sender.receiver_count() == 0 {
                        tracing::debug!("skipping broadcast (no readers)");
                    } else if let Err(error) = sender.send(state_str.clone()) {
                        tracing::error!(?error, "error sending message");
                    } else {
                        tracing::debug!(
                            channel_count = sender.receiver_count(),
                            "propagated message"
                        );
                    }
                    match serde_json::from_str::<ImgfloatAssetStateMessage>(&state_str) {
                        Ok(state) => match state {
                            ImgfloatAssetStateMessage::Delete(id) => {
                                let mut cache = self.state_cache.write().await;
                                if let Some(user_state) = cache.get_mut(username) {
                                    user_state.assets.retain(|asset| asset.id != id);
                                } else {
                                    tracing::warn!(
                                        ?cache,
                                        ?username,
                                        ?id,
                                        "unable to apply remove asset on missing state"
                                    )
                                }
                            }
                            ImgfloatAssetStateMessage::New(new_state) => {
                                self.state_cache
                                    .write()
                                    .await
                                    .insert(username.to_string(), new_state);
                            }
                            ImgfloatAssetStateMessage::Update(asset_update) => {
                                let mut cache = self.state_cache.write().await;
                                if let Some(user_state) = cache.get_mut(username) {
                                    if let Some(asset) = user_state
                                        .assets
                                        .iter_mut()
                                        .find(|a| a.id == asset_update.0.id)
                                    {
                                        asset.x = asset_update.0.x;
                                        asset.y = asset_update.0.y;
                                        asset.w = asset_update.0.w;
                                        asset.h = asset_update.0.h;
                                        asset.url = asset_update.0.url;
                                        tracing::debug!(
                                            ?asset,
                                            ?username,
                                            "applied partial asset state update"
                                        );
                                    }
                                } else {
                                    tracing::warn!(
                                        ?cache,
                                        ?username,
                                        ?asset_update,
                                        "unable to apply partial update to missing state"
                                    )
                                }
                            }
                        },
                        Err(error) => {
                            tracing::error!(?state_str, ?error, "could not de-serialize state");
                        }
                    };
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
