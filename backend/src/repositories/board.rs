use async_trait::async_trait;
use crate::domain::{Board, CreateBoard, DomainResult};

#[async_trait]
pub trait BoardRepository: Send + Sync {
    /// List all boards
    async fn find_all(&self) -> DomainResult<Vec<Board>>;

    /// Find board by slug (e.g., "tv", "g")
    async fn find_by_slug(&self, slug: &str) -> DomainResult<Option<Board>>;

    /// Find board by ID
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Board>>;

    /// Create a new board
    async fn create(&self, board: CreateBoard) -> DomainResult<Board>;

    /// Delete board by slug (returns true if deleted)
    async fn delete_by_slug(&self, slug: &str) -> DomainResult<bool>;

    /// Check if board exists by slug
    async fn exists(&self, slug: &str) -> DomainResult<bool>;

    /// Increment thread count
    async fn increment_thread_count(&self, board_id: i64) -> DomainResult<()>;

    /// Decrement thread count
    async fn decrement_thread_count(&self, board_id: i64) -> DomainResult<()>;
}
