use diesel::prelude::*;
use sha2::{Digest, Sha256};
use std::io::Write;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::models::schema::assets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Asset {
    pub local_filename: String,
    pub original_filename: String,
    pub checksum: String,
    pub content_type: String,
    pub broadcaster_username: String,
}

#[derive(serde::Serialize)]
pub struct UserFacingAsset {
    pub filename: String,
    pub content_type: String,
}

impl From<Asset> for UserFacingAsset {
    fn from(value: Asset) -> Self {
        Self {
            filename: value.local_filename,
            content_type: value.content_type,
        }
    }
}

#[derive(Debug)]
pub struct UnownedAsset {
    pub local_filename: String,
    pub original_filename: String,
    pub checksum: String,
    pub content_type: String,
}

impl UnownedAsset {
    pub async fn from_mutlipart(
        field: axum::extract::multipart::Field<'_>,
        asset_dir: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let original_filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field
            .content_type()
            .map(|mime| mime.to_string())
            .unwrap_or_else(|| "application/octet-stream".into());
        let extension = mime_guess::get_mime_extensions_str(&content_type)
            .and_then(|exts| exts.first().cloned())
            .unwrap_or("bin");
        let data = field
            .bytes()
            .await
            .inspect_err(|error| tracing::error!(?error, "unable to read file"))?;

        let local_filename = if extension.is_empty() {
            tracing::warn!(?original_filename, "filename had no extension");
            Uuid::new_v4().to_string()
        } else {
            format!("{}.{}", Uuid::new_v4(), extension)
        };

        std::fs::File::create(&format!("{asset_dir}/{local_filename}"))
            .inspect_err(|error| tracing::error!(?error, "unable to create file"))?
            .write_all(&data)
            .inspect_err(|error| tracing::error!(?error, "unable to write file"))?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let checksum = format!("{:x}", hasher.finalize());

        let unowned_asset = Self {
            local_filename,
            original_filename,
            checksum,
            content_type,
        };
        tracing::info!(?unowned_asset, "created unowned asset from multipart");
        Ok(unowned_asset)
    }

    pub fn with_ownership(self, owner_username: impl Into<String>) -> Asset {
        Asset {
            local_filename: self.local_filename,
            original_filename: self.original_filename,
            checksum: self.checksum,
            content_type: self.content_type,
            broadcaster_username: owner_username.into().clone(),
        }
    }
}
