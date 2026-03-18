use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::dto::{Thread as ThreadDto, ThreadDetail as ThreadDetailDto, ThreadListResponse, ThreadPreview as ThreadPreviewDto, Pagination};
use crate::api::state::AppState;
use crate::domain::{AdminClaims, CreatePost, ImageData, ThreadListParams};

#[derive(Debug, serde::Deserialize)]
pub struct ListThreadsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default = "default_preview_posts")]
    pub preview_posts: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 10 }
fn default_preview_posts() -> u32 { 3 }

pub async fn list_threads(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<ListThreadsQuery>,
) -> Result<Json<ThreadListResponse>, crate::domain::DomainError> {
    let params = ThreadListParams {
        page: q.page.max(1),
        limit: q.limit.clamp(1, 50),
        preview_posts: q.preview_posts.clamp(0, 10),
    };

    let paged = state.posts.list_threads(&slug, params).await?;
    let pagination = Pagination {
        page: paged.page,
        limit: paged.limit,
        total_items: paged.total_items,
        total_pages: paged.total_pages,
    };

    let threads = paged
        .items
        .into_iter()
        .map(|t| ThreadPreviewDto::from_domain(&slug, t))
        .collect();

    Ok(Json(ThreadListResponse { threads, pagination }))
}

pub async fn get_thread(
    State(state): State<AppState>,
    Path((slug, thread_number)): Path<(String, i64)>,
) -> Result<Json<ThreadDetailDto>, crate::domain::DomainError> {
    let detail = state.posts.get_thread(&slug, thread_number).await?;
    Ok(Json(ThreadDetailDto::from_domain(&slug, detail)))
}

pub async fn create_thread(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    mut multipart: axum::extract::Multipart,
) -> Result<(StatusCode, Json<ThreadDto>), crate::domain::DomainError> {
    let (name, text, image) = parse_post_multipart(&mut multipart).await?;
    let post = CreatePost { name, text, image };
    let op = state.posts.create_thread(&slug, post).await?;
    Ok((StatusCode::CREATED, Json(ThreadDto::from_domain(&slug, op))))
}

pub async fn delete_thread(
    State(state): State<AppState>,
    _claims: AdminClaims,
    Path((slug, thread_number)): Path<(String, i64)>,
) -> Result<StatusCode, crate::domain::DomainError> {
    state.posts.delete_thread(&slug, thread_number).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn parse_post_multipart(
    multipart: &mut axum::extract::Multipart,
) -> Result<(String, String, Option<ImageData>), crate::domain::DomainError> {
    use crate::domain::DomainError;

    let mut name: Option<String> = None;
    let mut text: Option<String> = None;
    let mut image: Option<ImageData> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| DomainError::Validation(format!("multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "name" => {
                let v = field
                    .text()
                    .await
                    .map_err(|e| DomainError::Validation(format!("invalid name: {}", e)))?;
                name = Some(v);
            }
            "text" => {
                let v = field
                    .text()
                    .await
                    .map_err(|e| DomainError::Validation(format!("invalid text: {}", e)))?;
                text = Some(v);
            }
            "image" => {
                let content_type = field
                    .content_type()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let file_name = field.file_name().unwrap_or("upload").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| DomainError::Validation(format!("invalid image: {}", e)))?
                    .to_vec();
                if !bytes.is_empty() {
                    image = Some(ImageData {
                        bytes,
                        original_name: file_name,
                        mime_type: content_type,
                    });
                }
            }
            _ => {}
        }
    }

    let name = name.unwrap_or_else(|| "Anonymous".to_string());
    let text = text.ok_or_else(|| DomainError::Validation("text is required".to_string()))?;
    Ok((name, text, image))
}

