use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}
