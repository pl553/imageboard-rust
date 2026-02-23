use thiserror::Error;

/// Core domain error - used across all layers
#[derive(Error, Debug)]
pub enum DomainError {
    // ---- Not Found ----
    #[error("Board not found: {0}")]
    BoardNotFound(String),

    #[error("Thread not found: {0}")]
    ThreadNotFound(i64),

    #[error("Post not found: {0}")]
    PostNotFound(i64),

    #[error("Image not found: {0}")]
    ImageNotFound(String),

    // ---- Conflict ----
    #[error("Board already exists: {0}")]
    BoardAlreadyExists(String),

    // ---- Validation ----
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid image: {0}")]
    InvalidImage(String),

    // ---- Auth ----
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Unauthorized")]
    Unauthorized,

    // ---- Internal ----
    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Convenience Result type
pub type DomainResult<T> = Result<T, DomainError>;
