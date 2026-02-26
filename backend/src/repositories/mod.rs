mod admin;
mod board;
mod thread;
mod post;
mod image;
pub mod postgres;

pub use admin::AdminRepository;
pub use board::BoardRepository;
pub use thread::ThreadRepository;
pub use post::PostRepository;
pub use image::ImageRepository;
