use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, JsonResponse, UserSession},
    models::UnownedUserSettings,
};

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn post(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
    Json(settings_request): Json<UnownedUserSettings>,
) -> Result<impl IntoResponse, StatusCode> {
    let session_user = session.user().await.ok_or(StatusCode::UNAUTHORIZED)?;
    let user = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;
    let settings = settings_request.with_owner(&user);
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
            Ok(JsonResponse::new(current_settings).with_status(status_code))
        }
        None => {
            let new_settings = database
                .write()
                .await
                .create_user_settings(&settings)
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            Ok(JsonResponse::new(new_settings).with_status(StatusCode::CREATED))
        }
    }
}

pub async fn get(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
) -> Result<impl IntoResponse, StatusCode> {
    let session_user = session.user().await.ok_or(StatusCode::UNAUTHORIZED)?;
    let user = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;
    match database.read().await.get_user_settings(&user) {
        Some(settings) => Ok(JsonResponse::new(settings)),
        None => {
            let settings = UnownedUserSettings::default().with_owner(&user);
            let new_settings = database
                .write()
                .await
                .create_user_settings(&settings)
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            Ok(JsonResponse::new(new_settings).with_status(StatusCode::CREATED))
        }
    }
}
