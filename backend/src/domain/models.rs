use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============ BOARD ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub thread_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateBoard {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

// ============ THREAD ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPreview {
    pub op: Post,
    pub reply_count: i64,
    pub omitted_count: i64,
    pub last_replies: Vec<Post>,
    pub bumped_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDetail {
    pub op: Post,
    pub replies: Vec<Post>,
}

// ============ POST ============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PostId {
    pub board_id: i64,
    pub post_number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub board_id: i64,
    pub post_number: i64,
    pub parent_number: Option<i64>,
    pub name: String,
    pub text: String,
    pub image: Option<ImageInfo>,
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub fn id(&self) -> PostId {
        PostId {
            board_id: self.board_id,
            post_number: self.post_number,
        }
    }
    
    pub fn is_op(&self) -> bool {
        self.parent_number.is_none()
    }
    
    pub fn thread_number(&self) -> i64 {
        self.parent_number.unwrap_or(self.post_number)
    }
    
    pub fn thread_id(&self) -> PostId {
        PostId {
            board_id: self.board_id,
            post_number: self.thread_number(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreatePost {
    pub name: String,
    pub text: String,
    pub image: Option<ImageData>,
}

// ============ IMAGE ============

/// Stored image metadata (from database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub filename: String,
    pub thumbnail_filename: String,
    pub original_name: String,
    pub size_bytes: i64,
    pub width: i32,
    pub height: i32,
    pub mime_type: String,
}

/// Raw image data for upload processing
#[derive(Debug, Clone)]
pub struct ImageData {
    pub bytes: Vec<u8>,
    pub original_name: String,
    pub mime_type: String,
}

/// Result of processing an uploaded image
#[derive(Debug, Clone)]
pub struct ProcessedImage {
    pub filename: String,
    pub thumbnail_filename: String,
    pub original_name: String,
    pub size_bytes: i64,
    pub width: i32,
    pub height: i32,
    pub mime_type: String,
}

// ============ AUTH ============

#[derive(Debug, Clone)]
pub struct Admin {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AdminClaims {
    pub admin_id: i64,
    pub username: String,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

// ============ PAGINATION ============

#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone)]
pub struct ThreadListParams {
    pub page: u32,
    pub limit: u32,
    pub preview_posts: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub limit: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, page: u32, limit: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (limit as f64)).ceil() as u32;
        Self {
            items,
            page,
            limit,
            total_items,
            total_pages,
        }
    }
}
