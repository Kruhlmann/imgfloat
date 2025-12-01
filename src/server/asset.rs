use dioxus::prelude::*;

use std::sync::Arc;

use crate::domain::AppState;
use crate::models::Asset;

#[server(prefix = "/api/asset", endpoint = "upload")]
pub async fn upload(code: String) -> Result<String, ServerFnError> {
    let FromContext(state): FromContext<Arc<AppState>> = extract()
        .await
        .inspect_err(|error| tracing::error!(?error, "error"))?;
    println!("{:?}", state.asset_dir);
    Ok("hello".into())
}
#[server(prefix = "/api/asset", endpoint = "upload")]
pub async fn upload(mut multipart: Multipart) -> Result<String, ServerFnError> {
    let FromContext(state): FromContext<Arc<AppState>> = extract().await?;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name()?.to_string();
        let file_name = field.file_name()?.to_string();
        let bytes = field.bytes().await?;

        // Example: write to disk in your asset_dir
        let path = state.asset_dir.join(&file_name);
        tokio::fs::write(&path, &bytes).await?;

        println!("Saved {file_name} ({name}) to {:?}", path);
    }

    Ok("ok".into())
}
