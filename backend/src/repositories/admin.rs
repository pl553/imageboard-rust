use async_trait::async_trait;
use crate::domain::{Admin, DomainResult};

#[async_trait]
pub trait AdminRepository: Send + Sync {
    /// Find admin by username
    async fn find_by_username(&self, username: &str) -> DomainResult<Option<Admin>>;

    /// Find admin by ID
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Admin>>;

    /// Create admin account
    async fn create(&self, username: &str, password_hash: &str) -> DomainResult<Admin>;
}
