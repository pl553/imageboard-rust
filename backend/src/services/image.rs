use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{DomainResult, ImageData, ProcessedImage};
use crate::repositories::ImageRepository;

#[async_trait]
pub trait ImageService: Send + Sync {
    async fn process(&self, image: ImageData) -> DomainResult<ProcessedImage>;
    async fn get(&self, filename: &str) -> DomainResult<Vec<u8>>;
    async fn get_thumbnail(&self, filename: &str) -> DomainResult<Vec<u8>>;
    async fn delete(&self, filename: &str, thumbnail_filename: &str) -> DomainResult<()>;
}

#[derive(Clone)]
pub struct ImageServiceImpl {
    image_repo: Arc<dyn ImageRepository>,
}

impl ImageServiceImpl {
    pub fn new(image_repo: Arc<dyn ImageRepository>) -> Self {
        Self { image_repo }
    }
}

#[async_trait]
impl ImageService for ImageServiceImpl {
    async fn process(&self, image: ImageData) -> DomainResult<ProcessedImage> {
        // The disk repo currently owns decode/thumbnail generation; service stays single-purpose.
        self.image_repo.save(image).await
    }

    async fn get(&self, filename: &str) -> DomainResult<Vec<u8>> {
        self.image_repo.get(filename).await
    }

    async fn get_thumbnail(&self, filename: &str) -> DomainResult<Vec<u8>> {
        self.image_repo.get_thumbnail(filename).await
    }

    async fn delete(&self, filename: &str, thumbnail_filename: &str) -> DomainResult<()> {
        self.image_repo.delete(filename, thumbnail_filename).await
    }
}

