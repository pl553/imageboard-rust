use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{CreatePost, DomainError, DomainResult, Post, PostId, ThreadDetail, ThreadListParams, ThreadPreview};
use crate::repositories::{BoardRepository, PostRepository};
use crate::services::ImageService;

#[async_trait]
pub trait PostService: Send + Sync {
    async fn list_threads(
        &self,
        board_slug: &str,
        params: ThreadListParams,
    ) -> DomainResult<crate::domain::Paginated<ThreadPreview>>;

    async fn get_thread(&self, board_slug: &str, thread_number: i64) -> DomainResult<ThreadDetail>;

    async fn create_thread(&self, board_slug: &str, post: CreatePost) -> DomainResult<Post>;

    async fn create_reply(
        &self,
        board_slug: &str,
        thread_number: i64,
        post: CreatePost,
    ) -> DomainResult<Post>;

    async fn delete_post(&self, board_slug: &str, post_number: i64) -> DomainResult<()>;

    async fn delete_thread(&self, board_slug: &str, thread_number: i64) -> DomainResult<()>;
}

#[derive(Clone)]
pub struct PostServiceImpl {
    board_repo: Arc<dyn BoardRepository>,
    post_repo: Arc<dyn PostRepository>,
    images: Arc<dyn ImageService>,
}

impl PostServiceImpl {
    pub fn new(
        board_repo: Arc<dyn BoardRepository>,
        post_repo: Arc<dyn PostRepository>,
        images: Arc<dyn ImageService>,
    ) -> Self {
        Self {
            board_repo,
            post_repo,
            images,
        }
    }

    fn validate_name(name: &str) -> DomainResult<()> {
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::Validation(
                "name must be 1..=100 chars".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_text(text: &str) -> DomainResult<()> {
        if text.is_empty() || text.len() > 10_000 {
            return Err(DomainError::Validation(
                "text must be 1..=10000 chars".to_string(),
            ));
        }
        Ok(())
    }

    async fn board_id_by_slug(&self, slug: &str) -> DomainResult<i64> {
        let board = self
            .board_repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| DomainError::BoardNotFound(slug.to_string()))?;
        Ok(board.id)
    }
}

#[async_trait]
impl PostService for PostServiceImpl {
    async fn list_threads(
        &self,
        board_slug: &str,
        params: ThreadListParams,
    ) -> DomainResult<crate::domain::Paginated<ThreadPreview>> {
        let board_id = self.board_id_by_slug(board_slug).await?;
        self.post_repo
            .find_thread_previews(board_id, params.page, params.limit, params.preview_posts)
            .await
    }

    async fn get_thread(&self, board_slug: &str, thread_number: i64) -> DomainResult<ThreadDetail> {
        let board_id = self.board_id_by_slug(board_slug).await?;
        let id = PostId {
            board_id,
            post_number: thread_number,
        };

        let detail = self
            .post_repo
            .find_thread_detail(id)
            .await?
            .ok_or(DomainError::ThreadNotFound(thread_number))?;

        // ensure it's actually a thread op
        if !detail.op.is_op() {
            return Err(DomainError::ThreadNotFound(thread_number));
        }

        Ok(detail)
    }

    async fn create_thread(&self, board_slug: &str, post: CreatePost) -> DomainResult<Post> {
        let board_id = self.board_id_by_slug(board_slug).await?;

        Self::validate_name(&post.name)?;
        Self::validate_text(&post.text)?;

        let image_info = if let Some(img) = post.image {
            Some(self.images.process(img).await?)
        } else {
            None
        };

        let image_ref = image_info.as_ref().map(|p| crate::domain::ImageInfo {
            filename: p.filename.clone(),
            thumbnail_filename: p.thumbnail_filename.clone(),
            original_name: p.original_name.clone(),
            size_bytes: p.size_bytes,
            width: p.width,
            height: p.height,
            mime_type: p.mime_type.clone(),
        });

        self.post_repo
            .create_thread(board_id, &post.name, &post.text, image_ref.as_ref())
            .await
    }

    async fn create_reply(
        &self,
        board_slug: &str,
        thread_number: i64,
        post: CreatePost,
    ) -> DomainResult<Post> {
        let board_id = self.board_id_by_slug(board_slug).await?;
        let thread_id = PostId {
            board_id,
            post_number: thread_number,
        };

        // Ensure parent exists and is OP
        if !self.post_repo.exists(thread_id).await? {
            return Err(DomainError::ThreadNotFound(thread_number));
        }
        if !self.post_repo.is_thread(thread_id).await? {
            return Err(DomainError::ThreadNotFound(thread_number));
        }

        Self::validate_name(&post.name)?;
        Self::validate_text(&post.text)?;

        let image_info = if let Some(img) = post.image {
            Some(self.images.process(img).await?)
        } else {
            None
        };

        let image_ref = image_info.as_ref().map(|p| crate::domain::ImageInfo {
            filename: p.filename.clone(),
            thumbnail_filename: p.thumbnail_filename.clone(),
            original_name: p.original_name.clone(),
            size_bytes: p.size_bytes,
            width: p.width,
            height: p.height,
            mime_type: p.mime_type.clone(),
        });

        self.post_repo
            .create_reply(
                board_id,
                thread_number,
                &post.name,
                &post.text,
                image_ref.as_ref(),
            )
            .await
    }

    async fn delete_post(&self, board_slug: &str, post_number: i64) -> DomainResult<()> {
        let board_id = self.board_id_by_slug(board_slug).await?;
        let id = PostId {
            board_id,
            post_number,
        };

        let post = self
            .post_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound(post_number))?;

        self.post_repo.delete(id).await?;

        if let Some(img) = post.image {
            // Best-effort cleanup; if it fails, bubble up (so admin knows)
            self.images
                .delete(&img.filename, &img.thumbnail_filename)
                .await?;
        }

        Ok(())
    }

    async fn delete_thread(&self, board_slug: &str, thread_number: i64) -> DomainResult<()> {
        let board_id = self.board_id_by_slug(board_slug).await?;
        let thread_id = PostId {
            board_id,
            post_number: thread_number,
        };

        let detail = self
            .post_repo
            .find_thread_detail(thread_id)
            .await?
            .ok_or(DomainError::ThreadNotFound(thread_number))?;

        if !detail.op.is_op() {
            return Err(DomainError::ThreadNotFound(thread_number));
        }

        // collect image files to delete after cascade
        let mut images: Vec<crate::domain::ImageInfo> = Vec::new();
        if let Some(img) = detail.op.image.clone() {
            images.push(img);
        }
        for r in &detail.replies {
            if let Some(img) = r.image.clone() {
                images.push(img);
            }
        }

        // delete OP; replies cascade via FK
        self.post_repo.delete(thread_id).await?;

        // delete files
        for img in images {
            self.images
                .delete(&img.filename, &img.thumbnail_filename)
                .await?;
        }

        Ok(())
    }
}

