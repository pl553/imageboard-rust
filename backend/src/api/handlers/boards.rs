use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::dto::{Board as BoardDto, CreateBoardRequest};
use crate::api::state::AppState;
use crate::domain::AdminClaims;

pub async fn list_boards(
    State(state): State<AppState>,
) -> Result<Json<Vec<BoardDto>>, crate::domain::DomainError> {
    let boards = state.boards.list_boards().await?;
    Ok(Json(boards.into_iter().map(BoardDto::from).collect()))
}

pub async fn get_board(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<BoardDto>, crate::domain::DomainError> {
    let board = state.boards.get_board(&slug).await?;
    Ok(Json(BoardDto::from(board)))
}

pub async fn create_board(
    State(state): State<AppState>,
    _claims: AdminClaims,
    Json(req): Json<CreateBoardRequest>,
) -> Result<(StatusCode, Json<BoardDto>), crate::domain::DomainError> {
    let board = state.boards.create_board(req.into()).await?;
    Ok((StatusCode::CREATED, Json(BoardDto::from(board))))
}

pub async fn delete_board(
    State(state): State<AppState>,
    _claims: AdminClaims,
    Path(slug): Path<String>,
) -> Result<StatusCode, crate::domain::DomainError> {
    state.boards.delete_board(&slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

