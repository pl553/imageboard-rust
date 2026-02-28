use async_trait::async_trait;
use crate::domain::{Paginated, Post, PostId, ThreadPreview, ThreadDetail, ImageInfo, DomainResult};

#[async_trait]
pub trait PostRepository: Send + Sync {
    /// Find post by ID
    async fn find_by_id(&self, id: PostId) -> DomainResult<Option<Post>>;

    /// Create a thread
    async fn create_thread(
        &self,
        board_id: i64,
        name: &str,
        text: &str,
        image: Option<&ImageInfo>,
    ) -> DomainResult<Post>;

    /// Create a reply to a thread
    async fn create_reply(
        &self,
        board_id: i64,
        parent_number: i64,
        name: &str,
        text: &str,
        image: Option<&ImageInfo>,
    ) -> DomainResult<Post>;

    /// Delete post
    async fn delete(&self, id: PostId) -> DomainResult<()>;
    
    // ---- Thread queries ----
    
    /// Get thread list for board page (paginated, with last N replies per thread)
    async fn find_thread_previews(
        &self,
        board_id: i64,
        page: u32,
        limit: u32,
        preview_replies: u32,
    ) -> DomainResult<Paginated<ThreadPreview>>;
    
    /// Get full thread with all replies
    async fn find_thread_detail(&self, thread: PostId) -> DomainResult<Option<ThreadDetail>>;
    
    /// Count threads on a board
    async fn count_threads(&self, board_id: i64) -> DomainResult<u64>;
    
    /// Delete all posts on a board
    async fn delete_by_board(&self, board_id: i64) -> DomainResult<()>;
    
    // ============ CHECKS ============
    
    async fn exists(&self, id: PostId) -> DomainResult<bool>;
    
    async fn is_thread(&self, id: PostId) -> DomainResult<bool>;
}
