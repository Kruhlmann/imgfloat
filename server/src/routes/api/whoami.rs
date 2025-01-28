use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::domain::UserSession;

#[axum::debug_handler]
pub async fn get(session: UserSession) -> impl IntoResponse {
    session.user.ok_or(StatusCode::UNAUTHORIZED).map(Json)
}
