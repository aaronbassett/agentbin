#![deny(unsafe_code)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde_json::{json, Value};

use agentbin_core::CoreError;

use crate::{middleware::auth::AuthenticatedUser, routes::upload::error_response, state::AppState};

fn is_io_not_found(e: &CoreError) -> bool {
    matches!(e,
        CoreError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound
    ) || matches!(e, CoreError::ValidationError(_))
}

fn is_storage_not_found(e: &CoreError) -> bool {
    matches!(e, CoreError::StorageError(msg) if msg.to_lowercase().contains("not found"))
}

/// `GET /api/uploads` — List the authenticated user's uploads with version details.
pub async fn list_uploads(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let records = state.storage.list_uploads(&user.username).map_err(|e| {
        tracing::error!(error = %e, "Storage error in list_uploads");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to list uploads",
        )
    })?;

    let mut uploads = Vec::with_capacity(records.len());
    for record in &records {
        let version_metas = state.storage.list_version_metas(&record.uid).map_err(|e| {
            tracing::error!(uid = %record.uid, error = %e, "Failed to list version metas");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to list versions",
            )
        })?;

        let versions: Vec<Value> = version_metas
            .iter()
            .map(|vm| {
                json!({
                    "version": vm.version,
                    "filename": vm.filename,
                    "size_bytes": vm.size_bytes,
                    "uploaded_at": vm.uploaded_at,
                    "url": format!("{}/{}/v{}", state.base_url, record.uid, vm.version),
                    "expires_at": vm.expires_at,
                })
            })
            .collect();

        uploads.push(json!({
            "uid": record.uid,
            "latest_version": record.latest_version,
            "collection": record.collection,
            "created_at": record.created_at,
            "versions": versions,
        }));
    }

    Ok(Json(json!({ "uploads": uploads })))
}

/// `DELETE /api/uploads/{uid}/v{version}` — Delete a specific version of an upload.
///
/// The authenticated user must be the owner of the upload or have admin privileges.
pub async fn delete_version(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((uid, version)): Path<(String, u32)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify the upload exists and load owner info.
    let record = state.storage.get_upload_record(&uid).map_err(|e| {
        if is_io_not_found(&e) {
            error_response(StatusCode::NOT_FOUND, "not_found", "Upload not found")
        } else {
            tracing::error!(uid = %uid, error = %e, "Storage error in get_upload_record");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to retrieve upload",
            )
        }
    })?;

    // Verify caller is the owner or an admin.
    if record.owner != user.username && !user.is_admin {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "You do not have permission to delete this upload",
        ));
    }

    state.storage.delete_version(&uid, version).map_err(|e| {
        if is_storage_not_found(&e) || is_io_not_found(&e) {
            error_response(StatusCode::NOT_FOUND, "not_found", "Version not found")
        } else {
            tracing::error!(uid = %uid, version, error = %e, "Storage error in delete_version");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to delete version",
            )
        }
    })?;

    Ok(Json(json!({
        "deleted": true,
        "uid": uid,
        "version": version,
    })))
}
