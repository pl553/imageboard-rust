pub mod dto;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod state;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::api::dto::ErrorResponse;
use crate::domain::DomainError;

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        let (status, msg, details): (StatusCode, String, Option<String>) = match &self {
            DomainError::BoardNotFound(_) | DomainError::ThreadNotFound(_) | DomainError::PostNotFound(_) | DomainError::ImageNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string(), None)
            }
            DomainError::AlreadyExists(_) => (StatusCode::CONFLICT, self.to_string(), None),
            DomainError::Validation(_) | DomainError::InvalidImage(_) => {
                (StatusCode::BAD_REQUEST, "Invalid request".to_string(), Some(self.to_string()))
            }
            DomainError::InvalidCredentials
            | DomainError::InvalidToken
            | DomainError::TokenExpired
            | DomainError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string(), Some(self.to_string())),
            DomainError::Database(_) | DomainError::Io(_) | DomainError::Internal(_) | DomainError::AdminNotFound(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string(), Some(self.to_string()))
            }
        };

        (status, Json(ErrorResponse { error: msg, details })).into_response()
    }
}

