/// Health check endpoint. Returns `{"status": "ok"}`.
pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
