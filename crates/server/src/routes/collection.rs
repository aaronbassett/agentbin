#![deny(unsafe_code)]

use askama::Template;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use agentbin_core::CoreError;

use crate::{
    middleware::auth::AuthenticatedUser,
    routes::upload::error_response,
    state::AppState,
    templates::{CollectionMemberView, CollectionTemplate},
};

const CSP: &str = "default-src 'self' 'unsafe-inline'; script-src 'self'; \
                   style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; \
                   frame-ancestors 'none'";

type HtmlResponse = Result<Response<Body>, (StatusCode, Json<Value>)>;

fn is_not_found(e: &CoreError) -> bool {
    matches!(e,
        CoreError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound
    ) || matches!(e, CoreError::ValidationError(_))
}

fn build_html_response(html: String) -> HtmlResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header("content-security-policy", CSP)
        .header("x-content-type-options", "nosniff")
        .header("x-frame-options", "DENY")
        .body(Body::from(html))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to build HTML response");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Failed to build response",
            )
        })
}

fn timeline_pct(ts: i64, min_ts: i64, range: i64) -> u8 {
    if range == 0 {
        50
    } else {
        u8::try_from(((ts - min_ts) * 100) / range).unwrap_or(50)
    }
}

/// `GET /c/:name` — Public collection overview page.
pub async fn view_collection(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> HtmlResponse {
    let collection = state.storage.get_collection(&name).map_err(|e| {
        if is_not_found(&e) {
            error_response(StatusCode::NOT_FOUND, "not_found", "Collection not found")
        } else {
            tracing::error!(name = %name, error = %e, "Storage error in get_collection");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to retrieve collection",
            )
        }
    })?;

    // Build enriched member views, skipping any orphaned entries.
    let mut member_entries: Vec<(i64, CollectionMemberView)> = Vec::new();
    for member in &collection.members {
        let uid = &member.uid;

        let Ok(record) = state.storage.get_upload_record(uid) else {
            tracing::warn!(uid = %uid, "Skipping orphaned collection member");
            continue;
        };

        let Ok((latest_meta, _)) = state.storage.get_version(uid, record.latest_version) else {
            tracing::warn!(uid = %uid, "Skipping collection member (version not found)");
            continue;
        };

        let url = format!("{}/{}", state.base_url, uid);
        let added_at = member.added_at.format("%Y-%m-%d %H:%M UTC").to_string();

        member_entries.push((
            member.added_at.timestamp(),
            CollectionMemberView {
                uid: uid.clone(),
                filename: latest_meta.filename,
                latest_version: record.latest_version,
                added_at,
                url,
                timeline_pct: 50, // placeholder; updated below
            },
        ));
    }

    // Sort chronologically so the timeline reads left-to-right.
    member_entries.sort_by_key(|(ts, _)| *ts);

    let min_ts = member_entries.first().map(|(ts, _)| *ts).unwrap_or(0);
    let max_ts = member_entries.last().map(|(ts, _)| *ts).unwrap_or(0);
    let range = max_ts - min_ts;

    let oldest_date = member_entries
        .first()
        .map(|(_, m)| m.added_at.clone())
        .unwrap_or_default();
    let newest_date = member_entries
        .last()
        .map(|(_, m)| m.added_at.clone())
        .unwrap_or_default();

    let members: Vec<CollectionMemberView> = member_entries
        .into_iter()
        .map(|(ts, mut m)| {
            m.timeline_pct = timeline_pct(ts, min_ts, range);
            m
        })
        .collect();

    let tmpl = CollectionTemplate {
        name: name.clone(),
        members,
        oldest_date,
        newest_date,
    };

    let html = tmpl.render().map_err(|e| {
        tracing::error!(name = %name, error = %e, "Template render error");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "render_error",
            "Failed to render template",
        )
    })?;

    build_html_response(html)
}

#[derive(Deserialize)]
pub struct AddMemberBody {
    uid: String,
}

/// `POST /api/collections/:name/members` — Add a file to a collection.
pub async fn add_member(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(name): Path<String>,
    Json(body): Json<AddMemberBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    state
        .storage
        .add_to_collection(&name, &body.uid)
        .map_err(|e| {
            tracing::error!(name = %name, uid = %body.uid, error = %e,
                "Storage error in add_to_collection");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Failed to add file to collection",
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "name": name,
            "uid": body.uid,
        })),
    ))
}

/// `DELETE /api/collections/:name/members/:uid` — Remove a file from a collection.
pub async fn remove_member(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path((name, uid)): Path<(String, String)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let collection_deleted = state
        .storage
        .remove_from_collection(&name, &uid)
        .map_err(|e| {
            if is_not_found(&e) {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "Collection or member not found",
                )
            } else {
                tracing::error!(name = %name, uid = %uid, error = %e,
                    "Storage error in remove_from_collection");
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    "Failed to remove file from collection",
                )
            }
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "name": name,
            "uid": uid,
            "collection_deleted": collection_deleted,
        })),
    ))
}
