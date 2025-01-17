use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::domain::{ApplicationController, UserSession};

pub async fn get(
    ws: WebSocketUpgrade,
    State(controller): State<Arc<ApplicationController>>,
    Path(username): Path<String>,
    session: UserSession,
) -> Response {
    tracing::info!(?username, "write socket requested");
    // TODO: Check username in list..
    match session.user().await {
        Some(_) => ws.on_upgrade(move |socket| async move {
            controller.add_writer(socket, &username).await;
        }),
        None => Redirect::temporary("/").into_response(),
    }
}
