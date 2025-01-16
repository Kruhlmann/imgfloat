use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::header::CONTENT_TYPE;

#[derive(Debug, serde::Serialize)]
pub struct RemoteImageBlob {
    bytes_base64: String,
    mime_type: String,
}

impl RemoteImageBlob {
    pub async fn download(url: &str) -> Option<Self> {
        tracing::info!(?url, "downloading image");
        let response = reqwest::get(url)
            .await
            .inspect_err(|error| tracing::error!(?url, ?error, "http error when reading image"))
            .ok()?;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())?;
        let mime_type = content_type
            .parse::<mime::Mime>()
            .inspect_err(|error| tracing::error!(?error, ?content_type, "invalid content type"))
            .ok()?;
        tracing::debug!(?mime_type);
        if mime_type.type_() != mime::IMAGE {
            tracing::error!(?mime_type, "resource isn't an image");
            if mime_type == mime::TEXT_PLAIN || mime_type == mime::TEXT_PLAIN_UTF_8 {
                let response_text = response
                    .text()
                    .await
                    .inspect_err(|error| {
                        tracing::error!(?url, ?error, "http error when reading image response")
                    })
                    .ok()?;
                tracing::debug!(?response_text, "text response received");
            }
            return None;
        }
        let bytes = response
            .bytes()
            .await
            .inspect_err(|error| {
                tracing::error!(?url, ?error, "http error when reading image response")
            })
            .ok()?;
        let bytes_base64 = BASE64_STANDARD.encode(&bytes);
        Some(Self {
            bytes_base64,
            mime_type: mime_type.to_string(),
        })
    }
}
