use serde::{Deserialize, Serialize};

use crate::domain;

#[derive(Debug, Serialize)]
pub struct Board {
    pub id: i64,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub thread_count: i32,
}

impl From<domain::Board> for Board {
    fn from(b: domain::Board) -> Self {
        Self {
            id: b.id,
            slug: b.slug,
            name: b.name,
            description: b.description,
            thread_count: b.thread_count,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBoardRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<CreateBoardRequest> for domain::CreateBoard {
    fn from(r: CreateBoardRequest) -> Self {
        Self {
            slug: r.slug,
            name: r.name,
            description: r.description,
        }
    }
}

