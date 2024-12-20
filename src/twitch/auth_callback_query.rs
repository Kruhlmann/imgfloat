#[derive(serde::Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub scope: String,
}
