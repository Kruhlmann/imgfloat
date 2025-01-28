use axum::{extract::State, http::StatusCode, response::IntoResponse};
use http_body_util::BodyExt;
use imgfloat::models::ChannelAdmin;
use imgfloat::routes::api::channel_admin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::fixture::TestDbService;
use crate::fixture::TestUser;

#[rstest::rstest]
async fn test_get_all() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let user_1 = TestUser::new("test-user-1");
    let user_2 = TestUser::new("test-user-2");
    let broadcaster_1 = TestUser::new("test-broadcaster-1");
    let broadcaster_2 = TestUser::new("test-broadcaster-2");
    let session = broadcaster_1.create_session();
    let channel_admin_1 = ChannelAdmin::new(&user_1.as_db_user(), &broadcaster_1.as_db_user());
    let channel_admin_2 = ChannelAdmin::new(&user_2.as_db_user(), &broadcaster_1.as_db_user());
    let channel_admin_3 = ChannelAdmin::new(&user_2.as_db_user(), &broadcaster_2.as_db_user());

    {
        let db = state.write().await;
        db.create_user(&user_1.as_db_user()).unwrap();
        db.create_user(&user_2.as_db_user()).unwrap();
        db.create_user(&broadcaster_1.as_db_user()).unwrap();
        db.create_user(&broadcaster_2.as_db_user()).unwrap();
        db.create_channel_admin(&channel_admin_1).unwrap();
        db.create_channel_admin(&channel_admin_2).unwrap();
        db.create_channel_admin(&channel_admin_3).unwrap();
    }

    let response = channel_admin::get(State(Arc::clone(&state)), session)
        .await
        .unwrap()
        .into_response();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actual: Vec<ChannelAdmin> = serde_json::from_slice(&body).unwrap();
    assert_eq!(actual, vec![channel_admin_1, channel_admin_2]);
}

#[rstest::rstest]
async fn test_get_all_empty() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();

    state.write().await.create_user(&user.as_db_user()).unwrap();

    let response = channel_admin::get(State(Arc::clone(&state)), session)
        .await
        .unwrap()
        .into_response();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actual: Vec<ChannelAdmin> = serde_json::from_slice(&body).unwrap();
    assert!(actual.is_empty());
}

#[rstest::rstest]
async fn test_create() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let broadcaster = TestUser::new("test-broadcaster");
    let user = TestUser::new("test-user");
    let session = broadcaster.create_session();
    let channel_admin_user = ChannelAdmin::new(&user.as_db_user(), &broadcaster.as_db_user());

    {
        let db = state.write().await;
        db.create_user(&broadcaster.as_db_user()).unwrap();
        db.create_user(&user.as_db_user()).unwrap();
    }

    let response = channel_admin::post(
        State(Arc::clone(&state)),
        session.clone(),
        channel_admin_user.username.clone(),
    )
    .await
    .unwrap()
    .into_response();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actual: ChannelAdmin = serde_json::from_slice(&body).unwrap();
    assert_eq!(actual, channel_admin_user);
}

#[rstest::rstest]
async fn test_create_duplicate() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let broadcaster = TestUser::new("test-broadcaster");
    let user = TestUser::new("test-user");
    let session = broadcaster.create_session();
    let channel_admin_user = ChannelAdmin::new(&user.as_db_user(), &broadcaster.as_db_user());

    {
        let db = state.write().await;
        db.create_user(&broadcaster.as_db_user()).unwrap();
        db.create_user(&user.as_db_user()).unwrap();
        db.create_channel_admin(&channel_admin_user).unwrap();
    }

    let response = channel_admin::post(
        State(Arc::clone(&state)),
        session.clone(),
        channel_admin_user.username.clone(),
    )
    .await
    .unwrap()
    .into_response();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let actual: ChannelAdmin = serde_json::from_slice(&body).unwrap();
    assert_eq!(actual, channel_admin_user);
}

#[rstest::rstest]
async fn test_missing_user() {
    let TestDbService(dbservice) = TestDbService::new();
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();

    match channel_admin::post(
        State(Arc::clone(&state)),
        session.clone(),
        "not-found".to_string(),
    )
    .await
    {
        Ok(_) => panic!("handler returned success"),
        Err(status_code) => assert_eq!(status_code, StatusCode::NOT_FOUND),
    }
}
