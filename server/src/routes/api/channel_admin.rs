use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, JsonResponse, UserSession},
    models::ChannelAdmin,
};

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn post(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
    channel_admin_username: String,
) -> Result<impl IntoResponse, StatusCode> {
    let session_user = session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let broadcaster = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;
    tracing::trace!(?broadcaster, ?channel_admin_username, "new channel admin");
    let channel_admin_user = database
        .read()
        .await
        .get_user(&channel_admin_username)
        .ok_or(StatusCode::NOT_FOUND)?;

    let existing_channel_admin = database
        .read()
        .await
        .get_channel_admin(&channel_admin_username, &broadcaster);
    let response = match existing_channel_admin {
        Some(channel_admin) => JsonResponse::new(channel_admin).with_status(StatusCode::OK),
        None => {
            let channel_admin = ChannelAdmin::new(&channel_admin_user, &broadcaster);
            let channel_admin = database
                .write()
                .await
                .create_channel_admin(&channel_admin)
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            JsonResponse::new(channel_admin).with_status(StatusCode::CREATED)
        }
    };
    Ok(response)
}

pub async fn get(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
) -> Result<impl IntoResponse, StatusCode> {
    let session_user = session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;
    let channel_admins = database
        .read()
        .await
        .get_channel_admins(&user)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let response = JsonResponse::new(channel_admins).with_status(StatusCode::OK);
    Ok(response)
}
