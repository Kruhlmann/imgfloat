use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use imgfloat::{
    routes::auth::callback,
    twitch::{AuthCallbackQuery, TwitchAuthenticator},
};
use tokio::sync::RwLock;

use crate::fixture::{EmptySession, TestAuthenticator, TestDbService, TestTwitchTokens, TestUser};

#[rstest::rstest]
#[tokio::test]
async fn test_new_user() {
    let user = TestUser::new("test-user");
    let TestDbService(dbservice) = TestDbService::new();
    let state_db = Arc::new(RwLock::new(dbservice));
    let EmptySession(session) = EmptySession::new();
    let TestTwitchTokens(tokens) = TestTwitchTokens::default();
    let authenticator = TestAuthenticator::new()
        .with_user(user.as_twitch_user())
        .with_tokens(tokens);
    let authenticator: Arc<Box<dyn TwitchAuthenticator>> = Arc::new(Box::new(authenticator));
    let auth_query = AuthCallbackQuery {
        code: Some("1".to_string()),
        error: None,
        error_description: None,
    };

    assert_eq!(
        None,
        state_db.read().await.get_user(&user.as_db_user().username)
    );
    let response = callback::get(
        State(authenticator),
        State(Arc::clone(&state_db)),
        session,
        Query(auth_query),
    )
    .await
    .into_response();

    let callback::AuthCallbackRedirect(redirect) =
        callback::AuthCallbackRedirect::new_with_user(&user.as_twitch_user().login);
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        Some(user.as_db_user()),
        state_db.read().await.get_user(&user.as_db_user().username)
    );
    assert_eq!(
        response.headers().get("Location"),
        redirect.into_response().headers().get("Location")
    );
}

#[rstest::rstest]
async fn test_failure_no_user() {
    let TestDbService(dbservice) = TestDbService::new();
    let state_db = Arc::new(RwLock::new(dbservice));
    let EmptySession(session) = EmptySession::new();
    let authenticator = TestAuthenticator::new();
    let authenticator: Arc<Box<dyn TwitchAuthenticator>> = Arc::new(Box::new(authenticator));
    let query_uri: Uri = "http://localhost:3000?error=foo&error_description=bar"
        .parse()
        .unwrap();
    let response = callback::get(
        State(authenticator),
        State(Arc::clone(&state_db)),
        session,
        Query::try_from_uri(&query_uri).unwrap(),
    )
    .await
    .into_response();

    let callback::AuthCallbackRedirect(redirect) = callback::AuthCallbackRedirect::new();
    let actual_url = response
        .headers()
        .get("Location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        actual_url,
        redirect
            .into_response()
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap()
    );
}

#[rstest::rstest]
async fn test_failure_no_tokens() {
    let user = TestUser::new("test-user");
    let TestDbService(dbservice) = TestDbService::new();
    let state_db = Arc::new(RwLock::new(dbservice));
    let EmptySession(session) = EmptySession::new();
    let authenticator = TestAuthenticator::new().with_user(user.as_twitch_user());
    let authenticator: Arc<Box<dyn TwitchAuthenticator>> = Arc::new(Box::new(authenticator));
    let query_uri: Uri = "http://localhost:3000?error=foo&error_description=bar"
        .parse()
        .unwrap();
    let response = callback::get(
        State(authenticator),
        State(Arc::clone(&state_db)),
        session,
        Query::try_from_uri(&query_uri).unwrap(),
    )
    .await
    .into_response();

    let callback::AuthCallbackRedirect(redirect) = callback::AuthCallbackRedirect::new();
    let actual_url = response
        .headers()
        .get("Location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        actual_url,
        redirect
            .into_response()
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap()
    );
}
