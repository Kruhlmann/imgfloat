use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

pub struct JsonResponse<T> {
    body: Json<T>,
    status_code: StatusCode,
}

impl<T> JsonResponse<T>
where
    T: Serialize,
{
    pub fn new(body: T) -> Self {
        Self {
            body: Json(body),
            status_code: StatusCode::OK,
        }
    }

    pub fn with_status(self, status_code: StatusCode) -> JsonResponse<T> {
        JsonResponse {
            body: self.body,
            status_code,
        }
    }
}

impl<T> IntoResponse for JsonResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response<Body> {
        (self.status_code, self.body.into_response()).into_response()
    }
}
