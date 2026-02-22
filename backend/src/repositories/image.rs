use async_trait::async_trait;
use std::path::Path;
use crate::domain::{ImageData, ProcessedImage, DomainResult};

/// Handles image file storage (filesystem operations)
#[async_trait]
pub trait ImageRepository: Send + Sync {
    /// Save image and generate thumbnail, returns processed metadata
    async fn save(&self, image: ImageData) -> DomainResult<ProcessedImage>;

    /// Get full image bytes
    async fn get(&self, filename: &str) -> DomainResult<Vec<u8>>;

    /// Get thumbnail bytes
    async fn get_thumbnail(&self, filename: &str) -> DomainResult<Vec<u8>>;

    /// Delete image and its thumbnail
    async fn delete(&self, filename: &str, thumbnail_filename: &str) -> DomainResult<()>;

    /// Check if image exists
    async fn exists(&self, filename: &str) -> DomainResult<bool>;

    /// Get path to images directory
    fn images_path(&self) -> &Path;

    /// Get path to thumbnails directory
    fn thumbnails_path(&self) -> &Path;
}
