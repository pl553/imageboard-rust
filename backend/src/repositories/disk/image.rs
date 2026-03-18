use async_trait::async_trait;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use image::{ImageFormat, imageops::FilterType};
use tokio::fs;

use crate::domain::{ImageData, ProcessedImage, DomainError, DomainResult};
use crate::repositories::ImageRepository;

const THUMBNAIL_MAX_WIDTH: u32 = 300;
const THUMBNAIL_MAX_HEIGHT: u32 = 300;
const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

pub struct DiskImageRepository {
    images_path: PathBuf,
    thumbnails_path: PathBuf,
}

impl DiskImageRepository {
    pub async fn new(images_path: PathBuf, thumbnails_path: PathBuf) -> DomainResult<Self> {
        fs::create_dir_all(&images_path)
            .await
            .map_err(|e| DomainError::Io(format!("Failed to create images directory: {}", e)))?;

        fs::create_dir_all(&thumbnails_path)
            .await
            .map_err(|e| DomainError::Io(format!("Failed to create thumbnails directory: {}", e)))?;

        Ok(Self {
            images_path,
            thumbnails_path,
        })
    }

    fn mime_to_extension(mime_type: &str) -> Option<&'static str> {
        match mime_type {
            "image/jpeg" => Some("jpg"),
            "image/png"  => Some("png"),
            "image/gif"  => Some("gif"),
            "image/webp" => Some("webp"),
            _ => None,
        }
    }
}

#[async_trait]
impl ImageRepository for DiskImageRepository {
    async fn save(&self, image: ImageData) -> DomainResult<ProcessedImage> {
        // Validate mime type
        if !ALLOWED_MIME_TYPES.contains(&image.mime_type.as_str()) {
            return Err(DomainError::InvalidImage(
                format!("Unsupported image type: {}", image.mime_type)
            ));
        }

        let ext = Self::mime_to_extension(&image.mime_type)
            .ok_or_else(|| DomainError::InvalidImage("Unknown extension".to_string()))?;

        // Decode image to get dimensions + generate thumbnail
        // This is CPU-bound so we run it in a blocking thread
        let bytes = image.bytes.clone();
        let mime = image.mime_type.clone();

        let (width, height, thumbnail_bytes) = tokio::task::spawn_blocking(move || {
            process_image(&bytes, &mime)
        })
        .await
        .map_err(|e| DomainError::Internal(format!("Task join error: {}", e)))?
        .map_err(|e| DomainError::InvalidImage(e))?;

        // Generate unique filenames
        let id = Uuid::new_v4();
        let filename = format!("{}.{}", id, ext);
        let thumbnail_filename = format!("{}_thumb.{}", id, ext);

        let image_path = self.images_path.join(&filename);
        let thumb_path = self.thumbnails_path.join(&thumbnail_filename);

        // Write original
        fs::write(&image_path, &image.bytes)
            .await
            .map_err(|e| DomainError::Io(format!("Failed to write image: {}", e)))?;

        // Write thumbnail
        fs::write(&thumb_path, &thumbnail_bytes)
            .await
            .map_err(|e| {
                // Best effort cleanup of original if thumb write fails
                let p = image_path.clone();
                tokio::spawn(async move { let _ = fs::remove_file(p).await; });
                DomainError::Io(format!("Failed to write thumbnail: {}", e))
            })?;

        Ok(ProcessedImage {
            filename,
            thumbnail_filename,
            original_name: image.original_name,
            size_bytes: image.bytes.len() as i64,
            width,
            height,
            mime_type: image.mime_type,
        })
    }

    async fn get(&self, filename: &str) -> DomainResult<Vec<u8>> {
        let path = self.images_path.join(filename);

        fs::read(&path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => DomainError::ImageNotFound(filename.to_string()),
                _ => DomainError::Io(format!("Failed to read image: {}", e)),
            })
    }

    async fn get_thumbnail(&self, filename: &str) -> DomainResult<Vec<u8>> {
        let path = self.thumbnails_path.join(filename);

        fs::read(&path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => DomainError::ImageNotFound(filename.to_string()),
                _ => DomainError::Io(format!("Failed to read thumbnail: {}", e)),
            })
    }

    async fn delete(&self, filename: &str, thumbnail_filename: &str) -> DomainResult<()> {
        let image_path = self.images_path.join(filename);
        let thumb_path = self.thumbnails_path.join(thumbnail_filename);

        // Delete both, collect errors, report if either failed
        let img_result = fs::remove_file(&image_path).await;
        let thumb_result = fs::remove_file(&thumb_path).await;

        // NotFound is fine - might already be deleted
        for result in [img_result, thumb_result] {
            if let Err(e) = result {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(DomainError::Io(format!("Failed to delete file: {}", e)));
                }
            }
        }

        Ok(())
    }

    async fn exists(&self, filename: &str) -> DomainResult<bool> {
        let path = self.images_path.join(filename);
        Ok(path.exists())
    }

    fn images_path(&self) -> &Path {
        &self.images_path
    }

    fn thumbnails_path(&self) -> &Path {
        &self.thumbnails_path
    }
}

/// CPU-bound image processing - run in spawn_blocking
/// Returns (width, height, thumbnail_bytes)
fn process_image(bytes: &[u8], mime_type: &str) -> Result<(i32, i32, Vec<u8>), String> {
    let format = mime_to_image_format(mime_type)
        .ok_or_else(|| format!("Unsupported format: {}", mime_type))?;

    let img = image::load_from_memory_with_format(bytes, format)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let width = img.width() as i32;
    let height = img.height() as i32;

    // Resize thumbnail maintaining aspect ratio
    let thumbnail = img.resize(
        THUMBNAIL_MAX_WIDTH,
        THUMBNAIL_MAX_HEIGHT,
        FilterType::Lanczos3,
    );

    // Encode thumbnail back to bytes
    let mut thumbnail_bytes: Vec<u8> = Vec::new();
    thumbnail
        .write_to(&mut std::io::Cursor::new(&mut thumbnail_bytes), format)
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    Ok((width, height, thumbnail_bytes))
}

fn mime_to_image_format(mime_type: &str) -> Option<ImageFormat> {
    match mime_type {
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/png"  => Some(ImageFormat::Png),
        "image/gif"  => Some(ImageFormat::Gif),
        "image/webp" => Some(ImageFormat::WebP),
        _ => None,
    }
}
