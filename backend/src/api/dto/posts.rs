use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain;

#[derive(Debug, Serialize)]
pub struct ImageInfo {
    pub filename: String,
    pub thumbnail_filename: String,
    pub original_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

impl From<domain::ImageInfo> for ImageInfo {
    fn from(i: domain::ImageInfo) -> Self {
        Self {
            filename: i.filename,
            thumbnail_filename: i.thumbnail_filename,
            original_name: i.original_name,
            size: Some(i.size_bytes),
            width: Some(i.width),
            height: Some(i.height),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Post {
    pub post_number: i64,
    pub board_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_number: Option<i64>,
    pub name: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageInfo>,
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub fn from_domain(board_slug: &str, p: domain::Post) -> Self {
        Self {
            post_number: p.post_number,
            board_slug: board_slug.to_string(),
            thread_number: p.parent_number,
            name: p.name,
            text: p.text,
            image: p.image.map(ImageInfo::from),
            created_at: p.created_at,
        }
    }
}

