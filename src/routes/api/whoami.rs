use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::domain::UserSession;

#[axum::debug_handler]
pub async fn get(session: UserSession) -> impl IntoResponse {
    session
        .user()
        .await
        .ok_or(StatusCode::UNAUTHORIZED)
        .map(Json)
}
