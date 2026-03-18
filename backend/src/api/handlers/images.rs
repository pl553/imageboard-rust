use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;

use crate::api::state::AppState;

pub async fn get_image(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, crate::domain::DomainError> {
    let bytes = state.images.get(&filename).await?;
    Ok(bytes_response(filename, bytes))
}

pub async fn get_thumbnail(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, crate::domain::DomainError> {
    let bytes = state.images.get_thumbnail(&filename).await?;
    Ok(bytes_response(filename, bytes))
}

fn bytes_response(filename: String, bytes: Vec<u8>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let mime = mime_guess::from_path(&filename).first_or_octet_stream();
    headers.insert(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
    (StatusCode::OK, headers, bytes)
}

