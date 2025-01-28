use axum::{extract::State, http::StatusCode, response::IntoResponse};
use http_body_util::BodyExt;
use imgfloat::models::user_settings::ValidatedUnownedUserSettings;
use imgfloat::models::UserSettings;
use imgfloat::routes::api::settings;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::fixture::TestDbService;
use crate::fixture::TestUser;

#[rstest::rstest]
async fn test_200_on_existing_settings() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();
    let expected = ValidatedUnownedUserSettings::default().with_owner(&user.as_db_user());

    state.write().await.create_user(&user.as_db_user()).unwrap();
    state.write().await.create_user_settings(&expected).unwrap();

    let response = settings::get(State(Arc::clone(&state)), session).await;
    match response {
        Ok(response) => {
            let response = response.into_response();
            assert_eq!(response.status(), StatusCode::OK);
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let actual: UserSettings = serde_json::from_slice(&body).unwrap();
            assert_eq!(expected, actual);
        }
        Err(status_code) => panic!("handler returned failure {status_code}"),
    }
}

#[rstest::rstest]
async fn test_201_on_missing_settings() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();
    state.write().await.create_user(&user.as_db_user()).unwrap();
    let response = settings::get(State(Arc::clone(&state)), session).await;
    match response {
        Ok(response) => {
            let response = response.into_response();
            assert_eq!(response.status(), StatusCode::CREATED);
            let expected = ValidatedUnownedUserSettings::default().with_owner(&user.as_db_user());
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let actual: UserSettings = serde_json::from_slice(&body).unwrap();
            assert_eq!(expected, actual);
        }
        Err(status_code) => panic!("handler returned failure {status_code}"),
    }
}

#[rstest::rstest]
async fn test_404_on_missing_user() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();
    let response = settings::get(State(Arc::clone(&state)), session).await;
    match response {
        Ok(_) => panic!("handler returned success"),
        Err(status_code) => assert_eq!(status_code, StatusCode::NOT_FOUND),
    }
}
