use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use imgfloat::{routes::auth::login, twitch::TwitchCredentials};

use crate::fixture::{EmptySession, TestUser};

#[rstest::rstest]
async fn test_redirect_no_session() {
    let EmptySession(session) = EmptySession::new();
    let credentials = Arc::new(TwitchCredentials {
        client_id: "a".to_string(),
        client_secret: "b".to_string(),
        redirect_uri: "c".to_string(),
    });
    let response = login::get(State(Arc::clone(&credentials)), session)
        .await
        .into_response();
    let login::LoginRedirect(expected_url) = login::LoginRedirect::new(None, credentials);
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
    let credentials = Arc::new(TwitchCredentials {
        client_id: "a".to_string(),
        client_secret: "b".to_string(),
        redirect_uri: "c".to_string(),
    });
    let response = login::get(State(Arc::clone(&credentials)), session)
        .await
        .into_response();
    let login::LoginRedirect(expected_url) =
        login::LoginRedirect::new(Some(&user.as_twitch_user()), credentials);
    let actual_url = response
        .headers()
        .get("Location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(actual_url, expected_url);
}
