use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use tokio::sync::RwLock;

use crate::{
    domain::{db::SqliteDbService, AssetDirectory},
    models::{UnownedAsset, UserFacingAsset},
};

// TODO: Check credentials
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
            .with_owner(&broadcaster);
        database
            .write()
            .await
            .create_asset(&asset)
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        return Ok(asset.local_filename);
    }

    Err(StatusCode::BAD_REQUEST)
}

// TODO: Check credentials
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

// TODO: Check credentials
#[axum::debug_handler(state = crate::domain::AppState)]
pub async fn get_one(
    State(database): State<Arc<RwLock<SqliteDbService>>>,
    State(AssetDirectory(asset_dir)): State<AssetDirectory>,
    Path((username, filename)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let broadcaster = match database.read().await.get_user(&username) {
        Some(user) => user,
        None => {
            tracing::error!(?username, "unknown broadcaster");
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let asset = match database.read().await.get_asset(&filename) {
        Some(asset) => asset,
        None => {
            tracing::error!(?filename, "asset not found in database");
            return Err(StatusCode::NOT_FOUND);
        }
    };

    if asset.username != broadcaster.username {
        tracing::warn!(
            asset_owner = ?asset.username,
            requested_by = ?broadcaster.username,
            "user does not own this asset"
        );
        return Err(StatusCode::NOT_FOUND);
    }

    let asset_path = format!("{}/{}", asset_dir, asset.local_filename);
    let data = tokio::fs::read(&asset_path).await.map_err(|err| {
        tracing::error!(?err, ?asset_path, "unable to read file from disk");
        StatusCode::NOT_FOUND
    })?;

    let content_type_header = [(header::CONTENT_TYPE, asset.content_type.clone())];

    Ok((StatusCode::OK, content_type_header, data))
}
