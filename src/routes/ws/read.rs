use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::IntoResponse,
};

use crate::domain::ChannelController;

pub async fn get(
    ws: WebSocketUpgrade,
    State(controller): State<Arc<ChannelController>>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    tracing::info!(?username, "read socket requested");
    ws.on_upgrade(move |socket| async move {
        controller.add_reader(socket, &username).await;
    })
}
