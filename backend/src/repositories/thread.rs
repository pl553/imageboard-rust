use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::domain::{Thread, DomainResult};

#[async_trait]
pub trait ThreadRepository: Send + Sync {
    /// Find thread by ID
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Thread>>;

    /// List threads for a board (paginated, ordered by bump time)
    async fn find_by_board_id(
        &self,
        board_id: i64,
        limit: u32,
        offset: u32,
    ) -> DomainResult<Vec<Thread>>;

    /// Count threads in a board
    async fn count_by_board_id(&self, board_id: i64) -> DomainResult<u64>;

    /// Create a new thread (returns thread with ID)
    async fn create(&self, board_id: i64) -> DomainResult<Thread>;

    /// Delete thread by ID (returns true if deleted)
    async fn delete(&self, id: i64) -> DomainResult<bool>;

    /// Update bump time (called when new post is added)
    async fn bump(&self, id: i64, time: DateTime<Utc>) -> DomainResult<()>;

    /// Increment post count
    async fn increment_post_count(&self, id: i64) -> DomainResult<()>;

    /// Decrement post count
    async fn decrement_post_count(&self, id: i64) -> DomainResult<()>;

    /// Check if thread exists
    async fn exists(&self, id: i64) -> DomainResult<bool>;
}
