use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use imgfloat::{
    routes::auth::callback,
    twitch::{AuthCallbackQuery, TwitchCredentials},
};
use tokio::sync::RwLock;

use crate::fixture::{EmptySession, TestDbService, TestUser};

#[rstest::rstest]
#[tokio::test]
async fn test_new_user() {
    let user = TestUser::new("test-user");
    let TestDbService(dbservice) = TestDbService::new();
    let state_db = Arc::new(RwLock::new(dbservice));
    let EmptySession(session) = EmptySession::new();
    let credentials = Arc::new(TwitchCredentials {
        client_id: "a".to_string(),
        client_secret: "b".to_string(),
        redirect_uri: "c".to_string(),
    });
    let query = Query(AuthCallbackQuery {
        code: Some("1".to_string()),
        error: None,
        error_description: None,
    });
    let response = callback::get(
        State(credentials),
        State(Arc::clone(&state_db)),
        session,
        query,
    )
    .await
    .into_response();

    assert!(state_db
        .read()
        .await
        .get_user(&user.as_db_user().username)
        .is_none());
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

// #[rstest::rstest]
// async fn test_failure() {
//     let TestDbService(dbservice) = TestDbService::new();
//     let state_db = Arc::new(RwLock::new(dbservice));
//     let EmptySession(session) = EmptySession::new();
//     let credentials = Arc::new(TwitchCredentials {
//         client_id: "a".to_string(),
//         client_secret: "b".to_string(),
//         redirect_uri: "c".to_string(),
//     });
//     let query_uri: Uri = "http://localhost:3000?error=foo&error_description=bar"
//         .parse()
//         .unwrap();
//     let response = callback::get(
//         State(credentials),
//         State(Arc::clone(&state_db)),
//         session,
//         Query::try_from_uri(&query_uri).unwrap(),
//     )
//     .await
//     .into_response();

//     let callback::AuthCallbackRedirect(redirect) = callback::AuthCallbackRedirect::new();
//     let actual_url = response
//         .headers()
//         .get("Location")
//         .unwrap()
//         .to_str()
//         .unwrap();
//     assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
//     assert_eq!(
//         actual_url,
//         redirect
//             .into_response()
//             .headers()
//             .get("Location")
//             .unwrap()
//             .to_str()
//             .unwrap()
//     );
// }
