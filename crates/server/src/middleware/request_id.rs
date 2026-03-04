use axum::{extract::Request, middleware::Next, response::Response};
use http::HeaderValue;
use uuid::Uuid;

/// Middleware that echoes the `X-Request-Id` header from the incoming request,
/// or generates a new UUID v4 if the header is absent.
///
/// The resolved request ID is written back to the response headers.
pub async fn request_id_middleware(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut response = next.run(req).await;

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }

    response
}
