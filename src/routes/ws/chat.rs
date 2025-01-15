use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::IntoResponse,
};
use tokio::sync::RwLock;

use crate::domain::ApplicationController;

pub async fn route(
    ws: WebSocketUpgrade,
    State(controller): State<Arc<RwLock<ApplicationController>>>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    tracing::info!(?username, "chat socket requested");
    ws.on_upgrade(move |socket| async move {
        controller
            .write()
            .await
            .register_chat_socket(socket, username)
            .await;
    })
}
