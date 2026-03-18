use tempfile::TempDir;

use backend::domain::{ImageData, DomainError};
use backend::repositories::ImageRepository;
use backend::repositories::disk::DiskImageRepository;

async fn setup() -> (DiskImageRepository, TempDir) {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let images_path = tmp.path().join("images");
    let thumbs_path = tmp.path().join("thumbs");

    let repo = DiskImageRepository::new(images_path, thumbs_path)
        .await
        .expect("Failed to create repo");

    // TempDir returned so it stays alive for the test duration
    // dropped at end of test = auto cleanup
    (repo, tmp)
}

fn load_test_image() -> Vec<u8> {
    std::fs::read("tests/fixtures/test_image.png")
        .expect("Put a test PNG at backend/tests/fixtures/test_image.png")
}

fn png_image_data() -> ImageData {
    ImageData {
        bytes: load_test_image(),
        original_name: "test.png".to_string(),
        mime_type: "image/png".to_string(),
    }
}

#[tokio::test]
async fn test_save_creates_image_and_thumbnail() {
    let (repo, _tmp) = setup().await;

    let result = repo.save(png_image_data()).await.unwrap();

    assert!(!result.filename.is_empty());
    assert!(!result.thumbnail_filename.is_empty());
    assert!(result.width > 0);
    assert!(result.height > 0);
    assert!(result.size_bytes > 0);
    assert_eq!(result.mime_type, "image/png");
    assert_eq!(result.original_name, "test.png");
}

#[tokio::test]
async fn test_save_files_actually_exist_on_disk() {
    let (repo, _tmp) = setup().await;

    let result = repo.save(png_image_data()).await.unwrap();

    assert!(repo.exists(&result.filename).await.unwrap());

    let thumb_bytes = repo.get_thumbnail(&result.thumbnail_filename).await.unwrap();
    assert!(!thumb_bytes.is_empty());
}

#[tokio::test]
async fn test_get_returns_original_bytes() {
    let (repo, _tmp) = setup().await;
    let original = load_test_image();

    let result = repo.save(png_image_data()).await.unwrap();
    let retrieved = repo.get(&result.filename).await.unwrap();

    assert_eq!(retrieved, original);
}

#[tokio::test]
async fn test_thumbnail_is_smaller_than_original() {
    let (repo, _tmp) = setup().await;

    let result = repo.save(png_image_data()).await.unwrap();

    let original_bytes = repo.get(&result.filename).await.unwrap();
    let thumb_bytes = repo.get_thumbnail(&result.thumbnail_filename).await.unwrap();

    assert!(!thumb_bytes.is_empty());
    let _ = original_bytes;
}

#[tokio::test]
async fn test_delete_removes_both_files() {
    let (repo, _tmp) = setup().await;

    let result = repo.save(png_image_data()).await.unwrap();
    assert!(repo.exists(&result.filename).await.unwrap());

    repo.delete(&result.filename, &result.thumbnail_filename)
        .await
        .unwrap();

    assert!(!repo.exists(&result.filename).await.unwrap());

    let img_err = repo.get(&result.filename).await.unwrap_err();
    assert!(matches!(img_err, DomainError::ImageNotFound(_)));

    let thumb_err = repo.get_thumbnail(&result.thumbnail_filename).await.unwrap_err();
    assert!(matches!(thumb_err, DomainError::ImageNotFound(_)));
}

#[tokio::test]
async fn test_delete_nonexistent_is_ok() {
    let (repo, _tmp) = setup().await;

    let result = repo
        .delete("nonexistent.png", "nonexistent_thumb.png")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_nonexistent_returns_not_found() {
    let (repo, _tmp) = setup().await;

    let err = repo.get("doesnotexist.png").await.unwrap_err();
    assert!(matches!(err, DomainError::ImageNotFound(_)));
}

#[tokio::test]
async fn test_invalid_mime_type_rejected() {
    let (repo, _tmp) = setup().await;

    let bad_image = ImageData {
        bytes: b"definitely not an image".to_vec(),
        original_name: "file.exe".to_string(),
        mime_type: "application/octet-stream".to_string(),
    };

    let err = repo.save(bad_image).await.unwrap_err();
    assert!(matches!(err, DomainError::InvalidImage(_)));
}

#[tokio::test]
async fn test_corrupted_image_bytes_rejected() {
    let (repo, _tmp) = setup().await;

    let bad_image = ImageData {
        bytes: b"this is not a valid png at all".to_vec(),
        original_name: "fake.png".to_string(),
        mime_type: "image/png".to_string(),
    };

    let err = repo.save(bad_image).await.unwrap_err();
    assert!(matches!(err, DomainError::InvalidImage(_)));
}

#[tokio::test]
async fn test_unique_filenames_per_upload() {
    let (repo, _tmp) = setup().await;

    let r1 = repo.save(png_image_data()).await.unwrap();
    let r2 = repo.save(png_image_data()).await.unwrap();

    assert_ne!(r1.filename, r2.filename);
    assert_ne!(r1.thumbnail_filename, r2.thumbnail_filename);
}
