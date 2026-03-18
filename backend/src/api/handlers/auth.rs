use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::api::dto::{AdminInfo, ChangePasswordRequest, LoginRequest, LoginResponse};
use crate::api::state::AppState;
use crate::domain::AdminClaims;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, crate::domain::DomainError> {
    let token = state.auth.login(&req.username, &req.password).await?;
    Ok(Json(LoginResponse {
        token: token.token,
        expires_at: token.expires_at,
    }))
}

pub async fn me(
    claims: AdminClaims,
) -> Result<Json<AdminInfo>, crate::domain::DomainError> {
    Ok(Json(AdminInfo {
        id: claims.admin_id,
        username: claims.username,
        created_at: None,
    }))
}

pub async fn change_password(
    State(state): State<AppState>,
    claims: AdminClaims,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, crate::domain::DomainError> {
    state
        .auth
        .change_password(claims.admin_id, &req.current_password, &req.new_password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

