use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::domain::AppState;
#[cfg(feature = "server")]
use crate::models::{Session, User};
use crate::twitch::TwitchUser;
#[cfg(feature = "server")]
use crate::COOKIE_SID_NAME;
#[cfg(feature = "server")]
use cookie::Cookie;
#[cfg(feature = "server")]
use std::sync::Arc;

#[server(prefix = "/api/auth/oauth", endpoint = "url")]
pub async fn create_oauth_url() -> Result<String, ServerFnError> {
    let FromContext(state): FromContext<Arc<AppState>> = extract()
        .await
        .inspect_err(|error| tracing::error!(?error, "error"))?;
    let url = state
        .twitch_authenticator
        .create_auth_url("user:read:email");
    Ok(url)
}

#[server(prefix = "/api", endpoint = "login")]
pub async fn login(code: String) -> Result<TwitchUser, ServerFnError> {
    let server_context = server_context();
    let mut response_parts = server_context.response_parts_mut();
    let FromContext(state): FromContext<Arc<AppState>> = extract()
        .await
        .inspect_err(|error| tracing::error!(?error, "error"))?;
    let tokens = state
        .twitch_authenticator
        .get_tokens(&code)
        .await
        .map_err(|_| ServerFnError::new("token request failed"))?;
    let user = state
        .twitch_authenticator
        .get_user(&tokens)
        .await
        .map_err(|_| ServerFnError::new("user request failed"))?;
    tracing::debug!(?user, "sign-in request");
    let db_user = match state.database.get_user(&user.login) {
        Some(u) => u,
        None => {
            tracing::info!(?user, "login from new user");
            let new_user: User = user.clone().into();
            let new_user = state
                .database
                .create_user(&new_user)
                .map_err(|_| ServerFnError::new("create user db request failed"))?;
            new_user
        }
    };
    let session: Session = db_user.into();
    let session = state
        .database
        .renew_user_session(&session)
        .map_err(|_| ServerFnError::new("create session db request failed"))?;

    let cookie_str = format!("{}={}; Path=/", COOKIE_SID_NAME, &session.token);
    let header_value = http::HeaderValue::from_str(&cookie_str)
        .map_err(|_| ServerFnError::new("cookie serialization failure"))?;
    response_parts
        .headers
        .insert(http::header::SET_COOKIE, header_value);
    Ok(user)
}
