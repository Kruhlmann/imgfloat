#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum ImgfloatAssetStateMessage {
    New(ImgfloatState),
    Update(ImgfloatAsset),
    Delete(String),
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct ImgfloatAsset {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub url: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct ImgfloatState {
    pub assets: Vec<ImgfloatAsset>,
}

impl From<ImgfloatState> for ImgfloatAssetStateMessage {
    fn from(value: ImgfloatState) -> Self {
        Self::New(value)
    }
}

impl From<&ImgfloatState> for ImgfloatAssetStateMessage {
    fn from(value: &ImgfloatState) -> Self {
        Self::New(value.clone())
    }
}
