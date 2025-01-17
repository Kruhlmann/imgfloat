use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::domain::UserSession;

pub async fn get(session: UserSession) -> Response {
    match session.user().await {
        Some(user) => Json(user).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}
