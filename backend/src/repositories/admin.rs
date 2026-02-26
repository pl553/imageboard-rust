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
    
    /// Change admin password
    async fn change_password(&self, id: i64, new_password_hash: &str) -> DomainResult<()>;

    /// Check if any admin exists
    async fn exists_any(&self) -> DomainResult<bool>;
}
