use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::api::dto::posts::Post as PostDto;
use crate::domain;

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

impl<T> From<domain::Paginated<T>> for Pagination {
    fn from(p: domain::Paginated<T>) -> Self {
        Self {
            page: p.page,
            limit: p.limit,
            total_items: p.total_items,
            total_pages: p.total_pages,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Thread {
    pub post_number: i64,
    pub board_slug: String,
    pub op_post: PostDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bumped_at: Option<DateTime<Utc>>,
}

impl Thread {
    pub fn from_domain(board_slug: &str, op: domain::Post) -> Self {
        Self {
            post_number: op.post_number,
            board_slug: board_slug.to_string(),
            op_post: PostDto::from_domain(board_slug, op.clone()),
            post_count: Some(1),
            created_at: op.created_at,
            bumped_at: Some(op.created_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ThreadPreview {
    pub post_number: i64,
    pub board_slug: String,
    pub op_post: PostDto,
    pub post_count: i64,
    pub omitted_posts: i64,
    pub last_posts: Vec<PostDto>,
    pub created_at: DateTime<Utc>,
    pub bumped_at: DateTime<Utc>,
}

impl ThreadPreview {
    pub fn from_domain(board_slug: &str, t: domain::ThreadPreview) -> Self {
        let post_count = t.reply_count + 1;
        Self {
            post_number: t.op.post_number,
            board_slug: board_slug.to_string(),
            op_post: PostDto::from_domain(board_slug, t.op.clone()),
            post_count,
            omitted_posts: t.omitted_count,
            last_posts: t
                .last_replies
                .into_iter()
                .map(|p| PostDto::from_domain(board_slug, p))
                .collect(),
            created_at: t.op.created_at,
            bumped_at: t.bumped_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ThreadListResponse {
    pub threads: Vec<ThreadPreview>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
pub struct ThreadDetail {
    pub post_number: i64,
    pub board_slug: String,
    pub op_post: PostDto,
    pub posts: Vec<PostDto>,
    pub created_at: DateTime<Utc>,
    pub bumped_at: DateTime<Utc>,
}

impl ThreadDetail {
    pub fn from_domain(board_slug: &str, d: domain::ThreadDetail) -> Self {
        let mut bumped_at = d.op.created_at;
        for r in &d.replies {
            if r.created_at > bumped_at {
                bumped_at = r.created_at;
            }
        }

        Self {
            post_number: d.op.post_number,
            board_slug: board_slug.to_string(),
            op_post: PostDto::from_domain(board_slug, d.op.clone()),
            posts: d
                .replies
                .into_iter()
                .map(|p| PostDto::from_domain(board_slug, p))
                .collect(),
            created_at: d.op.created_at,
            bumped_at,
        }
    }
}

