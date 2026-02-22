use async_trait::async_trait;
use crate::domain::{Post, ImageInfo, DomainResult};

#[async_trait]
pub trait PostRepository: Send + Sync {
    /// Find post by ID
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Post>>;

    /// Find OP (first post) of a thread
    async fn find_op_by_thread_id(&self, thread_id: i64) -> DomainResult<Option<Post>>;

    /// Find all posts in a thread (excluding OP)
    async fn find_by_thread_id(&self, thread_id: i64) -> DomainResult<Vec<Post>>;

    /// Find last N posts in a thread (for preview, excluding OP)
    async fn find_last_n_by_thread_id(
        &self,
        thread_id: i64,
        n: u32,
    ) -> DomainResult<Vec<Post>>;

    /// Count posts in a thread (excluding OP)
    async fn count_by_thread_id(&self, thread_id: i64) -> DomainResult<u64>;

    /// Create a post
    async fn create(
        &self,
        thread_id: i64,
        name: &str,
        text: &str,
        image: Option<&ImageInfo>,
        is_op: bool,
    ) -> DomainResult<Post>;

    /// Delete post by ID (returns deleted post's image info if any)
    async fn delete(&self, id: i64) -> DomainResult<Option<ImageInfo>>;

    /// Delete all posts in a thread (returns image infos for cleanup)
    async fn delete_by_thread_id(&self, thread_id: i64) -> DomainResult<Vec<ImageInfo>>;
}
