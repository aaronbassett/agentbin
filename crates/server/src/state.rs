use agentbin_core::FileStorage;
use std::sync::Arc;

/// Shared application state available to all route handlers.
#[derive(Clone)]
#[allow(dead_code)] // fields consumed by route handlers once implemented
pub struct AppState {
    pub storage: Arc<FileStorage>,
    pub base_url: String,
}
