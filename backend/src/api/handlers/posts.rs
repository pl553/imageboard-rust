use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::dto::Post as PostDto;
use crate::api::state::AppState;
use crate::domain::{AdminClaims, CreatePost};

pub async fn create_post(
    State(state): State<AppState>,
    Path((slug, thread_number)): Path<(String, i64)>,
    mut multipart: axum::extract::Multipart,
) -> Result<(StatusCode, Json<PostDto>), crate::domain::DomainError> {
    let (name, text, image) = super::threads::parse_post_multipart(&mut multipart).await?;
    let post = CreatePost { name, text, image };
    let created = state.posts.create_reply(&slug, thread_number, post).await?;
    Ok((StatusCode::CREATED, Json(PostDto::from_domain(&slug, created))))
}

pub async fn delete_post(
    State(state): State<AppState>,
    _claims: AdminClaims,
    Path((slug, post_number)): Path<(String, i64)>,
) -> Result<StatusCode, crate::domain::DomainError> {
    state.posts.delete_post(&slug, post_number).await?;
    Ok(StatusCode::NO_CONTENT)
}

