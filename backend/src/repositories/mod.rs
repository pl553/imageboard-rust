mod admin;
mod board;
mod post;
mod image;
pub mod disk;
pub mod postgres;

pub use admin::AdminRepository;
pub use board::BoardRepository;
pub use post::PostRepository;
pub use image::ImageRepository;
