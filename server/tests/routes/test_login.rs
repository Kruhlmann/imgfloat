use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use imgfloat::{routes::auth::login, twitch::TwitchAuthenticator};

use crate::fixture::{EmptySession, TestAuthenticator, TestUser};

#[rstest::rstest]
async fn test_redirect_no_session() {
    let EmptySession(session) = EmptySession::new();
    let authenticator = TestAuthenticator::new();
    let authenticator: Arc<Box<dyn TwitchAuthenticator>> = Arc::new(Box::new(authenticator));
    let response = login::get(State(Arc::clone(&authenticator)), session)
        .await
        .into_response();
    let login::LoginRedirect(expected_url) = login::LoginRedirect::new(None, authenticator);
    let actual_url = response
        .headers()
        .get("Location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(actual_url, expected_url);
}

#[rstest::rstest]
async fn test_redirect_existing_session() {
    let user = TestUser::new("test-user");
    let session = user.create_session();
    let authenticator = TestAuthenticator::new();
    let authenticator: Arc<Box<dyn TwitchAuthenticator>> = Arc::new(Box::new(authenticator));
    let response = login::get(State(Arc::clone(&authenticator)), session)
        .await
        .into_response();
    let login::LoginRedirect(expected_url) =
        login::LoginRedirect::new(Some(&user.as_twitch_user()), authenticator);
    let actual_url = response
        .headers()
        .get("Location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(actual_url, expected_url);
}
