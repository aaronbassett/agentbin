#![deny(unsafe_code)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::Utc;
use serde_json::{json, Value};

use agentbin_core::UserRecord;

use crate::{middleware::auth::AuthenticatedUser, routes::upload::error_response, state::AppState};

/// `POST /api/admin/users` — Add a new authorised user.
///
/// Request body (JSON):
/// ```json
/// { "username": "alice", "public_key": "<base64>", "display_name": "Alice", "is_admin": false }
/// ```
///
/// Returns `201 Created` with the new user record on success.
pub async fn add_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if !user.is_admin {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Admin access required",
        ));
    }

    let username = body
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                "missing_field",
                "Missing required field: 'username'",
            )
        })?
        .to_string();

    // Validate username format: 1-64 alphanumeric, hyphen, or underscore.
    if username.is_empty()
        || username.len() > 64
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "invalid_username",
            "Username must be 1-64 alphanumeric, hyphen, or underscore characters",
        ));
    }

    let public_key = body
        .get("public_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                "missing_field",
                "Missing required field: 'public_key'",
            )
        })?
        .to_string();

    let display_name = body
        .get("display_name")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let is_admin = body
        .get("is_admin")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut config = state.storage.load_users().map_err(|e| {
        tracing::error!(error = %e, "Failed to load users");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to load users",
        )
    })?;

    if config.users.contains_key(&username) {
        return Err(error_response(
            StatusCode::CONFLICT,
            "username_conflict",
            "A user with this username already exists",
        ));
    }

    config.users.insert(
        username.clone(),
        UserRecord {
            public_key: public_key.clone(),
            display_name: display_name.clone(),
            is_admin,
            created_at: Utc::now(),
        },
    );

    state.storage.save_users(&config).map_err(|e| {
        tracing::error!(error = %e, "Failed to save users");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to save users",
        )
    })?;

    tracing::info!(username = %username, by = %user.username, "Admin added user");

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "username": username,
            "public_key": public_key,
            "display_name": display_name,
            "is_admin": is_admin,
        })),
    ))
}

/// `PUT /api/admin/users/{username}` — Update an existing user's attributes.
///
/// Request body (JSON, all fields optional):
/// ```json
/// { "display_name": "New Name", "is_admin": true }
/// ```
///
/// Returns `200 OK` with the updated user record.
pub async fn update_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(username): Path<String>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if !user.is_admin {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Admin access required",
        ));
    }

    let mut config = state.storage.load_users().map_err(|e| {
        tracing::error!(error = %e, "Failed to load users");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to load users",
        )
    })?;

    if !config.users.contains_key(&username) {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "User not found",
        ));
    }

    // Validate demotion of last admin before mutating.
    if let Some(new_is_admin) = body.get("is_admin").and_then(|v| v.as_bool()) {
        let currently_admin = config
            .users
            .get(&username)
            .map(|u| u.is_admin)
            .unwrap_or(false);

        if currently_admin && !new_is_admin {
            let admin_count = config.users.values().filter(|u| u.is_admin).count();
            if admin_count <= 1 {
                return Err(error_response(
                    StatusCode::FORBIDDEN,
                    "last_admin",
                    "Cannot demote the last admin user",
                ));
            }
        }

        if let Some(record) = config.users.get_mut(&username) {
            record.is_admin = new_is_admin;
        }
    }

    if let Some(display_name_val) = body.get("display_name") {
        let new_display_name = if display_name_val.is_null() {
            None
        } else {
            display_name_val.as_str().map(str::to_string)
        };
        if let Some(record) = config.users.get_mut(&username) {
            record.display_name = new_display_name;
        }
    }

    state.storage.save_users(&config).map_err(|e| {
        tracing::error!(error = %e, "Failed to save users");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to save users",
        )
    })?;

    let record = config.users.get(&username).ok_or_else(|| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "state_error",
            "Failed to retrieve updated user record",
        )
    })?;

    tracing::info!(username = %username, by = %user.username, "Admin updated user");

    Ok((
        StatusCode::OK,
        Json(json!({
            "username": username,
            "public_key": record.public_key,
            "display_name": record.display_name,
            "is_admin": record.is_admin,
        })),
    ))
}

/// `DELETE /api/admin/users/{username}` — Remove a user.
///
/// Returns `200 OK` with `{"username": "...", "deleted": true}` on success.
pub async fn remove_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(username): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if !user.is_admin {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Admin access required",
        ));
    }

    let mut config = state.storage.load_users().map_err(|e| {
        tracing::error!(error = %e, "Failed to load users");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to load users",
        )
    })?;

    let target = config
        .users
        .get(&username)
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "not_found", "User not found"))?;

    // Prevent removing the last admin.
    if target.is_admin {
        let admin_count = config.users.values().filter(|u| u.is_admin).count();
        if admin_count <= 1 {
            return Err(error_response(
                StatusCode::FORBIDDEN,
                "last_admin",
                "Cannot remove the last admin user",
            ));
        }
    }

    config.users.remove(&username);

    state.storage.save_users(&config).map_err(|e| {
        tracing::error!(error = %e, "Failed to save users");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            "Failed to save users",
        )
    })?;

    tracing::info!(username = %username, by = %user.username, "Admin removed user");

    Ok((
        StatusCode::OK,
        Json(json!({ "username": username, "deleted": true })),
    ))
}
