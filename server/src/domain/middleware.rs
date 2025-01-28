use axum::{extract::Request, middleware::Next, response::Response};

pub async fn log_requests(request: Request, next: Next) -> Response {
    let uri = request.uri().clone();
    let before = std::time::Instant::now();
    let response = next.run(request).await;
    if response.status().is_client_error() || response.status().is_server_error() {
        tracing::warn!(?uri, duration_ms = ?before.elapsed(), status = ?response.status(), "http error")
    } else {
        tracing::trace!(?uri, duration_ms = ?before.elapsed(), status = ?response.status(), "http response");
    }
    response
}
