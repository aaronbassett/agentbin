#![deny(unsafe_code)]

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use serde_json::{json, Value};

use agentbin_core::{extract_uid, uid_with_slug, CoreError, Metadata};

use crate::{middleware::auth::AuthenticatedUser, state::AppState};

const MAX_FILE_SIZE: usize = 1_048_576; // 1 MiB

/// Returns a consistent JSON error body per the API spec:
/// `{"error": {"code": "...", "message": "...", "status": <u16>}}`
pub fn error_response(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
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

/// `POST /api/upload` — Create a new upload.
///
/// Accepts a `multipart/form-data` body with two fields:
/// - `file` (required): the file to upload
/// - `metadata` (optional): JSON string conforming to [`Metadata`] plus optional
///   top-level `collection` (string) and `expiry` (integer, days from now) keys
///
/// Returns `201 Created` with:
/// ```json
/// {"uid":"…","version":1,"url":"…","raw_url":"…"}
/// ```
pub async fn create_upload(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let mut file_data: Option<(String, Vec<u8>)> = None;
    let mut metadata_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::warn!(error = %e, "Failed to read multipart field");
        error_response(
            StatusCode::BAD_REQUEST,
            "invalid_multipart",
            "Failed to parse multipart form data",
        )
    })? {
        let name = field.name().map(str::to_string).unwrap_or_default();

        match name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .map(str::to_string)
                    .unwrap_or_else(|| "upload".to_string());

                let data = field.bytes().await.map_err(|e| {
                    tracing::warn!(error = %e, "Failed to read file bytes");
                    error_response(
                        StatusCode::BAD_REQUEST,
                        "read_error",
                        "Failed to read file field",
                    )
                })?;

                if data.len() > MAX_FILE_SIZE {
                    return Err(error_response(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "file_too_large",
                        "File size exceeds the 1 MiB limit",
                    ));
                }

                file_data = Some((filename, data.to_vec()));
            }
            "metadata" => {
                let text = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "Failed to read metadata text");
                    error_response(
                        StatusCode::BAD_REQUEST,
                        "read_error",
                        "Failed to read metadata field",
                    )
                })?;
                metadata_str = Some(text);
            }
            _ => {
                // Consume and discard unknown fields.
            }
        }
    }

    let (filename, content) = file_data.ok_or_else(|| {
        error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing_file",
            "Required 'file' field is missing from the multipart body",
        )
    })?;

    // Parse optional metadata JSON, pulling out the upload-specific keys
    // (`collection`, `expiry`) before deserialising the remainder into `Metadata`.
    let (metadata, collection, expiry_days) = match metadata_str {
        Some(raw) => {
            let value: Value = serde_json::from_str(&raw).map_err(|e| {
                tracing::warn!(error = %e, "Invalid metadata JSON");
                error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_metadata",
                    "Metadata field is not valid JSON",
                )
            })?;

            let collection = value
                .get("collection")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            let expiry_days = value.get("expiry").and_then(|v| v.as_i64());

            let metadata: Metadata = serde_json::from_value(value).map_err(|e| {
                tracing::warn!(error = %e, "Metadata does not match schema");
                error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_metadata",
                    "Metadata does not match the expected schema",
                )
            })?;

            (metadata, collection, expiry_days)
        }
        None => (Metadata::default(), None, None),
    };

    let expires_at = expiry_days.map(|days| Utc::now() + chrono::Duration::days(days));

    let (record, _) = state
        .storage
        .create_upload(
            &user.username,
            &filename,
            &content,
            metadata,
            collection.as_deref(),
            expires_at,
        )
        .map_err(|e| {
            tracing::error!(error = %e, "Storage error during create_upload");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to store the upload",
            )
        })?;

    let slugged = uid_with_slug(&record.uid, record.slug.as_deref());
    let url = format!("{}/{}", state.base_url, slugged);
    let raw_url = format!("{}/{}/raw", state.base_url, slugged);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "uid": record.uid,
            "version": 1,
            "url": url,
            "raw_url": raw_url
        })),
    ))
}

/// `POST /api/upload/:uid` — Upload a new version of an existing upload.
///
/// Accepts `multipart/form-data` with the same fields as [`create_upload`].
/// The `collection` metadata field is ignored for versioned uploads (collection
/// membership is set at creation time only).
///
/// Returns `201 Created` with:
/// ```json
/// {"uid":"…","version":N,"url":"…/{uid}/v{N}","raw_url":"…/{uid}/v{N}/raw"}
/// ```
pub async fn upload_version(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(uid): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let uid = extract_uid(&uid);

    let mut file_data: Option<(String, Vec<u8>)> = None;
    let mut metadata_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::warn!(error = %e, "Failed to read multipart field");
        error_response(
            StatusCode::BAD_REQUEST,
            "invalid_multipart",
            "Failed to parse multipart form data",
        )
    })? {
        let name = field.name().map(str::to_string).unwrap_or_default();

        match name.as_str() {
            "file" => {
                let filename = field
                    .file_name()
                    .map(str::to_string)
                    .unwrap_or_else(|| "upload".to_string());

                let data = field.bytes().await.map_err(|e| {
                    tracing::warn!(error = %e, "Failed to read file bytes");
                    error_response(
                        StatusCode::BAD_REQUEST,
                        "read_error",
                        "Failed to read file field",
                    )
                })?;

                if data.len() > MAX_FILE_SIZE {
                    return Err(error_response(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "file_too_large",
                        "File size exceeds the 1 MiB limit",
                    ));
                }

                file_data = Some((filename, data.to_vec()));
            }
            "metadata" => {
                let text = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "Failed to read metadata text");
                    error_response(
                        StatusCode::BAD_REQUEST,
                        "read_error",
                        "Failed to read metadata field",
                    )
                })?;
                metadata_str = Some(text);
            }
            _ => {
                // Consume and discard unknown fields.
            }
        }
    }

    let (filename, content) = file_data.ok_or_else(|| {
        error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing_file",
            "Required 'file' field is missing from the multipart body",
        )
    })?;

    let (metadata, expiry_days) = match metadata_str {
        Some(raw) => {
            let value: Value = serde_json::from_str(&raw).map_err(|e| {
                tracing::warn!(error = %e, "Invalid metadata JSON");
                error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_metadata",
                    "Metadata field is not valid JSON",
                )
            })?;

            let expiry_days = value.get("expiry").and_then(|v| v.as_i64());

            let metadata: Metadata = serde_json::from_value(value).map_err(|e| {
                tracing::warn!(error = %e, "Metadata does not match schema");
                error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_metadata",
                    "Metadata does not match the expected schema",
                )
            })?;

            (metadata, expiry_days)
        }
        None => (Metadata::default(), None),
    };

    let expires_at = expiry_days.map(|days| Utc::now() + chrono::Duration::days(days));

    let version_meta = state
        .storage
        .store_version(
            uid,
            &user.username,
            &filename,
            &content,
            metadata,
            expires_at,
        )
        .map_err(|e| match &e {
            CoreError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
                error_response(StatusCode::NOT_FOUND, "not_found", "Upload not found")
            }
            CoreError::ValidationError(_) => error_response(
                StatusCode::FORBIDDEN,
                "forbidden",
                "You do not own this upload",
            ),
            _ => {
                tracing::error!(uid = %uid, error = %e, "Storage error during store_version");
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    "Failed to store the version",
                )
            }
        })?;

    let version = version_meta.version;
    let record = state.storage.get_upload_record(uid).map_err(|e| {
        tracing::error!(uid = %uid, error = %e, "Storage error reading upload record for slug");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to read upload record",
        )
    })?;
    let slugged = uid_with_slug(uid, record.slug.as_deref());
    let url = format!("{}/{}/v{}", state.base_url, slugged, version);
    let raw_url = format!("{}/{}/v{}/raw", state.base_url, slugged, version);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "uid": uid,
            "version": version,
            "url": url,
            "raw_url": raw_url
        })),
    ))
}
