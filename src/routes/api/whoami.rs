use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::UserSession;

pub async fn route(session: UserSession) -> Response {
    match session.unlock().await {
        Ok(s) => Json(s.user).into_response(),
        Err(()) => StatusCode::UNAUTHORIZED.into_response(),
    }
}
