use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, JsonResponse, UserSession},
    models::{user_settings::ValidatedUnownedUserSettings, UnownedUserSettings},
};

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn put(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    session: UserSession,
    Json(settings_request): Json<UnownedUserSettings>,
) -> Result<impl IntoResponse, StatusCode> {
    let session_user = session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;
    tracing::trace!(?user, ?settings_request, "new settings");
    let settings = settings_request
        .validate()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_owner(&user);
    tracing::trace!(?user, ?settings, "change to settings validated");
    let current_settings = database.read().await.get_user_settings(&user);
    match current_settings {
        Some(current_settings) => {
            let status_code = if current_settings == settings {
                tracing::trace!(?user, ?settings, "user already had these settings");
                StatusCode::OK
            } else {
                tracing::trace!(?user, ?settings, "writing new settings");
                database
                    .write()
                    .await
                    .update_user_settings(&settings)
                    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
                StatusCode::CREATED
            };
            Ok(JsonResponse::new(settings).with_status(status_code))
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
    let session_user = session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user = database
        .read()
        .await
        .get_user(&session_user.login)
        .ok_or(StatusCode::NOT_FOUND)?;

    let existing_settings = {
        let db = database.read().await;
        db.get_user_settings(&user)
    };

    match existing_settings {
        Some(settings) => Ok(JsonResponse::new(settings).with_status(StatusCode::OK)),
        None => {
            let new_settings = ValidatedUnownedUserSettings::default().with_owner(&user);
            database
                .write()
                .await
                .create_user_settings(&new_settings)
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            Ok(JsonResponse::new(new_settings).with_status(StatusCode::CREATED))
        }
    }
}
