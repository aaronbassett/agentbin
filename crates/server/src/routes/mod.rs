pub mod health;

use axum::{middleware, routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::{
    middleware::{auth::auth_middleware, request_id::request_id_middleware},
    state::AppState,
};

/// Build the application router.
///
/// Route structure:
/// - `GET /health` — unauthenticated health check
/// - `GET /api/*` — authenticated API routes (Ed25519 auth middleware applied)
///
/// Global middleware (outermost first):
/// 1. `request_id_middleware` — echo or generate `X-Request-Id`
/// 2. [`TraceLayer`] — HTTP tracing spans
pub fn create_router(state: AppState) -> Router {
    // Routes that require authentication.
    let api_routes = Router::new().route_layer(middleware::from_fn_with_state(
        state.clone(),
        auth_middleware,
    ));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api", api_routes)
        // Layers are applied last-in, first-out; request_id runs before TraceLayer.
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(request_id_middleware))
        .with_state(state)
}
