use axum::{extract::State, http::StatusCode, response::IntoResponse};
use http_body_util::BodyExt;
use imgfloat::models::UnownedUserSettings;
use imgfloat::routes::api::settings;
use imgfloat::{domain::db::SqliteDbService, models::UserSettings};
use std::sync::Arc;
use tokio::sync::RwLock;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::fixture::user::TestUser;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[rstest::fixture]
fn dbservice() -> SqliteDbService {
    let service = SqliteDbService::new(":memory:").unwrap();
    let mut conn = service.pool.get().unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    service
}

#[rstest::rstest]
async fn test_200_on_existing_settings(dbservice: SqliteDbService) {
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();
    state.write().await.create_user(&user.as_db_user()).unwrap();
    let _ = settings::get(State(Arc::clone(&state)), session.clone())
        .await
        .unwrap();
    let response = settings::get(State(Arc::clone(&state)), session).await;
    match response {
        Ok(response) => {
            let response = response.into_response();
            assert_eq!(response.status(), StatusCode::OK);
            let expected = UnownedUserSettings::default().with_owner(&user.as_db_user());
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let actual: UserSettings = serde_json::from_slice(&body).unwrap();
            assert_eq!(expected, actual);
        }
        Err(status_code) => panic!("handler returned failure {status_code}"),
    }
}

#[rstest::rstest]
async fn test_201_on_missing_settings(dbservice: SqliteDbService) {
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();
    state.write().await.create_user(&user.as_db_user()).unwrap();
    let response = settings::get(State(Arc::clone(&state)), session).await;
    match response {
        Ok(response) => {
            let response = response.into_response();
            assert_eq!(response.status(), StatusCode::CREATED);
            let expected = UnownedUserSettings::default().with_owner(&user.as_db_user());
            let body = response.into_body().collect().await.unwrap().to_bytes();
            let actual: UserSettings = serde_json::from_slice(&body).unwrap();
            assert_eq!(expected, actual);
        }
        Err(status_code) => panic!("handler returned failure {status_code}"),
    }
}

#[rstest::rstest]
async fn test_404_on_missing_user(dbservice: SqliteDbService) {
    let state = Arc::new(RwLock::new(dbservice));
    let user = TestUser::new("test-user");
    let session = user.create_session();
    let response = settings::get(State(Arc::clone(&state)), session).await;
    match response {
        Ok(_) => panic!("handler returned success"),
        Err(status_code) => assert_eq!(status_code, StatusCode::NOT_FOUND),
    }
}
