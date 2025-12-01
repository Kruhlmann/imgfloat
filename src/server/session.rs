use dioxus::prelude::*;

#[server(prefix = "/api/session", endpoint = "full")]
pub async fn get_session_info() -> Result<String, ServerFnError> {
    use crate::domain::{AppState, SessionCookie, Traceable};
    use crate::models::{Session, User};
    use crate::COOKIE_SID_NAME;
    use std::sync::Arc;

    let FromContext(state): FromContext<Arc<AppState>> = extract().await.trace_err("app state")?;
    let headers: http::HeaderMap = extract().await?;
    let SessionCookie(cookie) = SessionCookie::try_from_header_map(headers.clone())?;
    let session = state
        .database
        .get_user_session_token(cookie.value())
        .ok_or(ServerFnError::new("not logged in"))
        .inspect_err(|error| tracing::error!(token = ?cookie.value(), "session not found"))?;
    Ok(session.username)
}
