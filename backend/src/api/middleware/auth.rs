use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::api::state::AppState;
use crate::domain::{AdminClaims, DomainError};

impl FromRequestParts<AppState> for AdminClaims {
    type Rejection = DomainError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let auth = state.auth.clone();

        async move {
            let auth_header = auth_header.ok_or(DomainError::Unauthorized)?;
            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(DomainError::Unauthorized)?;
            auth.verify(token)
        }
    }
}

