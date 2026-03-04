use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
};
use serde_json::json;

use agentbin_core::{construct_signing_payload, validate_timestamp, verify_signature};

use crate::state::AppState;

/// Identity injected into request extensions after successful authentication.
#[derive(Clone, Debug)]
#[allow(dead_code)] // fields consumed by route handlers once implemented
pub struct AuthenticatedUser {
    pub username: String,
    pub is_admin: bool,
}

fn unauthorized(code: &str, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": code, "message": message })),
    )
}

/// Axum middleware that validates Ed25519 request signatures.
///
/// Expects three headers:
/// - `X-AgentBin-PublicKey`  — base64-encoded Ed25519 public key
/// - `X-AgentBin-Signature`  — base64-encoded Ed25519 signature
/// - `X-AgentBin-Timestamp`  — Unix timestamp (seconds, i64)
///
/// On success, inserts [`AuthenticatedUser`] into the request extensions and
/// forwards the request (with the buffered body) to the next handler.
///
/// On failure, returns a JSON error response with an appropriate status code.
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = req.headers();

    let public_key = headers
        .get("x-agentbin-publickey")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or_else(|| unauthorized("missing_headers", "Missing X-AgentBin-PublicKey header"))?;

    let signature = headers
        .get("x-agentbin-signature")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or_else(|| unauthorized("missing_headers", "Missing X-AgentBin-Signature header"))?;

    let timestamp_str = headers
        .get("x-agentbin-timestamp")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or_else(|| unauthorized("missing_headers", "Missing X-AgentBin-Timestamp header"))?;

    let timestamp: i64 = timestamp_str.parse().map_err(|_| {
        unauthorized(
            "invalid_timestamp",
            "Timestamp must be a Unix epoch integer",
        )
    })?;

    validate_timestamp(timestamp).map_err(|e| unauthorized("replay_detected", &e.to_string()))?;

    // Look up the public key in the users config.
    let users_config = state.storage.load_users().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "storage_error", "message": "Failed to load users" })),
        )
    })?;

    let (username, user_record) = users_config
        .users
        .iter()
        .find(|(_, u)| u.public_key == public_key)
        .ok_or_else(|| unauthorized("unknown_key", "Public key not recognized"))?;

    // Buffer the request body so it can be included in signature verification
    // and then replayed to the downstream handler.
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "body_error", "message": "Failed to read request body" })),
        )
    })?;

    let method = parts.method.as_str();
    let path = parts.uri.path();
    let payload = construct_signing_payload(method, path, timestamp, &body_bytes);

    verify_signature(&public_key, &payload, &signature)
        .map_err(|e| unauthorized("invalid_signature", &e.to_string()))?;

    let authenticated_user = AuthenticatedUser {
        username: username.clone(),
        is_admin: user_record.is_admin,
    };

    // Reconstruct the request with the buffered body so the handler can read it.
    let mut req = Request::from_parts(parts, Body::from(body_bytes));
    req.extensions_mut().insert(authenticated_user);

    Ok(next.run(req).await)
}
