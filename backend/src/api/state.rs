use std::sync::Arc;

use crate::services::{AuthService, BoardService, ImageService, PostService};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    pub boards: Arc<dyn BoardService>,
    pub posts: Arc<dyn PostService>,
    pub images: Arc<dyn ImageService>,
}

