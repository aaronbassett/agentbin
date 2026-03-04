#![deny(unsafe_code)]

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    Json,
};
use serde_json::{json, Value};

use agentbin_core::CoreError;

use crate::state::AppState;

type RawResponse = Result<Response<Body>, (StatusCode, Json<Value>)>;

fn raw_error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": {
                "code": code,
                "message": message,
                "status": status.as_u16()
            }
        })),
    )
}

fn is_not_found(e: &CoreError) -> bool {
    matches!(e,
        CoreError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound
    ) || matches!(e, CoreError::ValidationError(_))
}

/// Shared raw-content response builder.
fn serve_raw(content: Vec<u8>) -> RawResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CONTENT_DISPOSITION, "inline")
        .body(Body::from(content))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to build raw response");
            raw_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to build response",
            )
        })
}

/// `GET /{uid}/raw` — Return the latest version as raw plain text.
pub async fn raw_latest(State(state): State<AppState>, Path(uid): Path<String>) -> RawResponse {
    let (_record, _meta, content) = state.storage.get_latest_version(&uid).map_err(|e| {
        if is_not_found(&e) {
            raw_error(StatusCode::NOT_FOUND, "not_found", "Upload not found")
        } else {
            tracing::error!(uid = %uid, error = %e, "Storage error in get_latest_version");
            raw_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to retrieve upload",
            )
        }
    })?;
    serve_raw(content)
}

/// `GET /{uid}/v{version}/raw` — Return a specific version as raw plain text.
pub async fn raw_version(
    State(state): State<AppState>,
    Path((uid, version)): Path<(String, u32)>,
) -> RawResponse {
    let (_meta, content) = state.storage.get_version(&uid, version).map_err(|e| {
        if is_not_found(&e) {
            raw_error(StatusCode::NOT_FOUND, "not_found", "Version not found")
        } else {
            tracing::error!(uid = %uid, version, error = %e, "Storage error in get_version");
            raw_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to retrieve version",
            )
        }
    })?;
    serve_raw(content)
}
