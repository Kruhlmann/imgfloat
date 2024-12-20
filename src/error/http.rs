use axum::http::StatusCode;

pub fn internal_server_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub fn service_unavailable<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::SERVICE_UNAVAILABLE, err.to_string())
}

pub fn bad_gateway<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::BAD_GATEWAY, err.to_string())
}

pub fn not_found<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::BAD_GATEWAY, err.to_string())
}
