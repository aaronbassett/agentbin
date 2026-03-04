pub mod admin;
pub mod collection;
pub mod health;
pub mod manage;
pub mod raw;
pub mod upload;
pub mod view;

use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    middleware::{auth::auth_middleware, request_id::request_id_middleware},
    state::AppState,
};

/// Embedded badge WebComponent script served at `/_static/badge.js`.
const BADGE_JS: &str = include_str!("../../static/badge.js");

async fn serve_badge_js() -> Response<Body> {
    match Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )
        .header("cache-control", "public, max-age=3600")
        .body(Body::from(BADGE_JS))
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!(error = %e, "Failed to build badge.js response");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap_or_default()
        }
    }
}

/// Build the application router.
///
/// Route structure:
/// - `GET  /health`              — unauthenticated health check
/// - `GET  /_static/badge.js`   — embedded badge WebComponent (no auth)
/// - `POST /api/upload`          — create a new upload (authenticated)
/// - `POST /api/upload/:uid`     — add a new version to an existing upload (authenticated)
/// - `GET  /{uid}`               — view latest version (no auth)
/// - `GET  /{uid}/v{version}`    — view specific version (no auth)
///
/// Specific routes are registered before the `/{uid}` catch-all so they take
/// precedence. Global middleware (outermost first):
/// 1. `request_id_middleware` — echo or generate `X-Request-Id`
/// 2. [`TraceLayer`] — HTTP tracing spans
pub fn create_router(state: AppState) -> Router {
    // Admin routes — require authentication.
    let admin_routes = Router::new().route("/users", post(admin::add_user)).route(
        "/users/{username}",
        put(admin::update_user).delete(admin::remove_user),
    );

    // Routes that require authentication.
    let api_routes = Router::new()
        .route("/upload", post(upload::create_upload))
        .route("/upload/{uid}", post(upload::upload_version))
        .route("/uploads", get(manage::list_uploads))
        .route("/uploads/{uid}/v{version}", delete(manage::delete_version))
        .route("/collections/{name}/members", post(collection::add_member))
        .route(
            "/collections/{name}/members/{uid}",
            delete(collection::remove_member),
        )
        .nest("/admin", admin_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Specific static/infra routes must come before the /{uid} catch-all.
    Router::new()
        .route("/health", get(health::health))
        .route("/_static/badge.js", get(serve_badge_js))
        .nest("/api", api_routes)
        // Public collection overview — no auth required.
        .route("/c/{name}", get(collection::view_collection))
        // Public raw routes — no auth required. Must come before the /{uid} catch-all.
        .route("/{uid}/raw", get(raw::raw_latest))
        .route("/{uid}/v{version}/raw", get(raw::raw_version))
        // Public view routes — no auth required.
        .route("/{uid}/v{version}", get(view::view_version))
        .route("/{uid}", get(view::view_latest))
        // Layers are applied last-in, first-out; request_id runs before TraceLayer.
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(request_id_middleware))
        .with_state(state)
}
