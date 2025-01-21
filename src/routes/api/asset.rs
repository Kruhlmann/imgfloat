use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, AssetDirectory},
    models::{UnownedAsset, UserFacingAsset},
};

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn post(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    State(AssetDirectory(asset_dir)): State<AssetDirectory>,
    Path(username): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let broadcaster = match database.read().await.get_user(&username) {
        Some(user) => user,
        None => {
            tracing::error!(?username, "unknown broadcaster");
            return Err(StatusCode::NOT_FOUND);
        }
    };
    while let Some(field) = multipart
        .next_field()
        .await
        .inspect_err(|error| tracing::error!(?error, "no multipart request body"))
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let asset = UnownedAsset::from_mutlipart(field, asset_dir)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
            .with_ownership(&broadcaster.twitch_username);
        database
            .write()
            .await
            .create_asset(&asset)
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        return Ok(asset.local_filename);
    }

    Err(StatusCode::BAD_REQUEST)
}

#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn get(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    Path(username): Path<String>,
) -> Result<Json<Vec<UserFacingAsset>>, StatusCode> {
    let broadcaster = match database.read().await.get_user(&username) {
        Some(user) => user,
        None => {
            tracing::error!(?username, "unknown broadcaster");
            return Err(StatusCode::NOT_FOUND);
        }
    };
    let assets: Json<Vec<UserFacingAsset>> = database
        .read()
        .await
        .get_broadcaster_assets(&broadcaster)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .into_iter()
        .map(|asset| asset.into())
        .collect::<Vec<UserFacingAsset>>()
        .into();
    Ok(assets)
}
