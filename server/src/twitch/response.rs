#[derive(serde::Deserialize)]
pub struct TwitchApiResponse<T> {
    pub data: T,
}
