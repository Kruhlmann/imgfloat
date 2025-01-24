use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, UserSession},
    models::UserSettings,
};

#[derive(Debug, serde::Deserialize)]
pub struct SettingsRequest {
    background_opacity: u8,
    fps_target: u16,
}

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn post(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
    Json(settings_request): Json<SettingsRequest>,
) -> Result<(StatusCode, Json<UserSettings>), StatusCode> {
    let session_user = session.user().await.ok_or(StatusCode::UNAUTHORIZED)?;
    let user = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;
    let settings = UserSettings {
        username: user.username.clone(),
        background_opacity: settings_request.background_opacity.into(),
        fps_target: settings_request.fps_target.into(),
    };
    match database.read().await.get_user_settings(&user) {
        Some(current_settings) => {
            let status_code = if current_settings == settings {
                StatusCode::NOT_MODIFIED
            } else {
                database
                    .write()
                    .await
                    .update_user_settings(&settings)
                    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
                StatusCode::OK
            };
            let response = (status_code, Json(current_settings));
            Ok(response)
        }
        None => {
            let new_settings = database
                .write()
                .await
                .create_user_settings(&settings)
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let response = (StatusCode::CREATED, Json(new_settings));
            Ok(response)
        }
    }
}
