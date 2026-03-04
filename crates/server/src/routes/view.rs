#![deny(unsafe_code)]

use askama::Template;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    Json,
};
use serde_json::{json, Value};

use agentbin_core::{CoreError, FileType};

use crate::{
    badge::{generate_badge_html, inject_badge_into_html},
    render::{render_content, RenderResult},
    state::AppState,
    templates::{PlainTemplate, RenderedTemplate},
};

const CSP: &str = "default-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; \
                   style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; \
                   frame-ancestors 'none'";

type ViewResponse = Result<Response<Body>, (StatusCode, Json<Value>)>;

fn view_error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
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
    matches!(e, CoreError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound)
}

fn build_html_response(html: String) -> ViewResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header("content-security-policy", CSP)
        .header("x-content-type-options", "nosniff")
        .header("x-frame-options", "DENY")
        .body(Body::from(html))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to build HTML response");
            view_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to build response",
            )
        })
}

/// Shared rendering logic for both view handlers.
async fn render_view(state: &AppState, uid: &str, requested_version: Option<u32>) -> ViewResponse {
    // Load upload record + version content from storage.
    let (record, version_meta, content_bytes) = match requested_version {
        None => state.storage.get_latest_version(uid).map_err(|e| {
            if is_not_found(&e) {
                view_error(StatusCode::NOT_FOUND, "not_found", "Upload not found")
            } else {
                tracing::error!(uid, error = %e, "Storage error in get_latest_version");
                view_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    "Failed to retrieve upload",
                )
            }
        })?,
        Some(v) => {
            let record = state.storage.get_upload_record(uid).map_err(|e| {
                if is_not_found(&e) {
                    view_error(StatusCode::NOT_FOUND, "not_found", "Upload not found")
                } else {
                    tracing::error!(uid, error = %e, "Storage error in get_upload_record");
                    view_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "storage_error",
                        "Failed to retrieve upload",
                    )
                }
            })?;
            let (version_meta, content_bytes) = state.storage.get_version(uid, v).map_err(|e| {
                if is_not_found(&e) {
                    view_error(StatusCode::NOT_FOUND, "not_found", "Version not found")
                } else {
                    tracing::error!(uid, version = v, error = %e, "Storage error in get_version");
                    view_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "storage_error",
                        "Failed to retrieve version",
                    )
                }
            })?;
            (record, version_meta, content_bytes)
        }
    };

    // Convert bytes to string — uploads are text documents.
    let content = String::from_utf8(content_bytes).map_err(|e| {
        tracing::warn!(uid, error = %e, "Content is not valid UTF-8");
        view_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "encoding_error",
            "Content is not valid UTF-8",
        )
    })?;

    let file_type = FileType::from_filename(&version_meta.filename);
    let badge_html = generate_badge_html(&version_meta, &state.base_url, uid);

    // Show a version banner when viewing a non-latest specific version.
    let version_banner_html = requested_version.and_then(|req_v| {
        if req_v != record.latest_version {
            let latest_url = format!("{}/{}", state.base_url, uid);
            Some(format!(
                r#"<div class="version-banner">Viewing version {req_v}. \
                   <a href="{latest_url}">View latest (v{})</a>.</div>"#,
                record.latest_version
            ))
        } else {
            None
        }
    });

    let render_result =
        render_content(&content, &file_type, &version_meta.filename).map_err(|e| {
            tracing::error!(uid, error = %e, "Render error");
            view_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "render_error",
                "Failed to render content",
            )
        })?;

    let html = match render_result {
        RenderResult::HtmlPassthrough(html) => inject_badge_into_html(&html, &badge_html),
        RenderResult::Rendered { title, content } => RenderedTemplate {
            title,
            content,
            badge_html,
            version_banner_html,
        }
        .render()
        .map_err(|e| {
            tracing::error!(uid, error = %e, "Template render error");
            view_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "render_error",
                "Failed to render template",
            )
        })?,
        RenderResult::PlainText { title, content } => PlainTemplate {
            title,
            content,
            badge_html,
            version_banner_html,
        }
        .render()
        .map_err(|e| {
            tracing::error!(uid, error = %e, "Template render error");
            view_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "render_error",
                "Failed to render template",
            )
        })?,
    };

    build_html_response(html)
}

/// `GET /{uid}` — View the latest version of an upload.
pub async fn view_latest(State(state): State<AppState>, Path(uid): Path<String>) -> ViewResponse {
    render_view(&state, &uid, None).await
}

/// `GET /{uid}/v{version}` — View a specific version of an upload.
pub async fn view_version(
    State(state): State<AppState>,
    Path((uid, version)): Path<(String, u32)>,
) -> ViewResponse {
    render_view(&state, &uid, Some(version)).await
}
