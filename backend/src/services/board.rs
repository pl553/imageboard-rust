use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Board, CreateBoard, DomainError, DomainResult};
use crate::repositories::{BoardRepository, PostRepository};

#[async_trait]
pub trait BoardService: Send + Sync {
    async fn list_boards(&self) -> DomainResult<Vec<Board>>;
    async fn get_board(&self, slug: &str) -> DomainResult<Board>;
    async fn create_board(&self, req: CreateBoard) -> DomainResult<Board>;
    async fn delete_board(&self, slug: &str) -> DomainResult<()>;
}

#[derive(Clone)]
pub struct BoardServiceImpl {
    board_repo: Arc<dyn BoardRepository>,
    post_repo: Arc<dyn PostRepository>,
}

impl BoardServiceImpl {
    pub fn new(board_repo: Arc<dyn BoardRepository>, post_repo: Arc<dyn PostRepository>) -> Self {
        Self { board_repo, post_repo }
    }

    fn validate_slug(slug: &str) -> DomainResult<()> {
        let ok = !slug.is_empty()
            && slug.len() <= 10
            && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());

        if !ok {
            return Err(DomainError::Validation(
                "slug must match ^[a-z0-9]+$ and be 1..=10 chars".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_name(name: &str) -> DomainResult<()> {
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::Validation(
                "name must be 1..=100 chars".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_description(description: &Option<String>) -> DomainResult<()> {
        if let Some(d) = description {
            if d.len() > 500 {
                return Err(DomainError::Validation(
                    "description must be <= 500 chars".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl BoardService for BoardServiceImpl {
    async fn list_boards(&self) -> DomainResult<Vec<Board>> {
        self.board_repo.find_all().await
    }

    async fn get_board(&self, slug: &str) -> DomainResult<Board> {
        Self::validate_slug(slug)?;
        self.board_repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| DomainError::BoardNotFound(slug.to_string()))
    }

    async fn create_board(&self, req: CreateBoard) -> DomainResult<Board> {
        Self::validate_slug(&req.slug)?;
        Self::validate_name(&req.name)?;
        Self::validate_description(&req.description)?;

        if self.board_repo.exists(&req.slug).await? {
            return Err(DomainError::AlreadyExists(format!(
                "board '{}' already exists",
                req.slug
            )));
        }

        self.board_repo.create(req).await
    }

    async fn delete_board(&self, slug: &str) -> DomainResult<()> {
        Self::validate_slug(slug)?;

        let board = self
            .board_repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| DomainError::BoardNotFound(slug.to_string()))?;

        // Delete posts first (or rely on cascade; this is safe and explicit)
        self.post_repo.delete_by_board(board.id).await?;

        let deleted = self.board_repo.delete_by_slug(slug).await?;
        if !deleted {
            return Err(DomainError::BoardNotFound(slug.to_string()));
        }
        Ok(())
    }
}

