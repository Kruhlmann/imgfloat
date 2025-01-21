#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ImgfloatAssetStateMessage {
    New(ImgfloatState),
    Update(ImgfloatStateUpdate),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ImgfloatAsset {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub url: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ImgfloatState {
    pub assets: Vec<ImgfloatAsset>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ImgfloatStateUpdate(pub ImgfloatAsset);
