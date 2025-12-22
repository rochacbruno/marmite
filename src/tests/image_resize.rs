use super::*;
use crate::config::Marmite;
use crate::content::ContentBuilder;
use image::{ImageBuffer, Rgb};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use tempfile::TempDir;

#[test]
fn test_is_image_file() {
    assert!(is_image_file(Path::new("test.jpg")));
    assert!(is_image_file(Path::new("test.JPEG")));
    assert!(is_image_file(Path::new("test.png")));
    assert!(is_image_file(Path::new("test.webp")));
    assert!(!is_image_file(Path::new("test.txt")));
    assert!(!is_image_file(Path::new("test.md")));
}

#[test]
fn test_is_banner_image() {
    assert!(is_banner_image(Path::new("post.banner.jpg")));
    assert!(is_banner_image(Path::new("my-post.banner.png")));
    assert!(!is_banner_image(Path::new("regular-image.jpg")));
    assert!(!is_banner_image(Path::new("banner-style.jpg")));
}

#[test]
fn test_resize_image() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.jpg");
    let output_path = temp_dir.path().join("output.jpg");

    // Create a 1000x500 test image
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(1000, 500);
    img.save(&input_path).unwrap();

    // Resize to max width 800
    let resized = resize_image(&input_path, &output_path, 800).unwrap();
    assert!(resized);

    // Verify output dimensions
    let output_img = image::open(&output_path).unwrap();
    assert_eq!(output_img.width(), 800);
    assert_eq!(output_img.height(), 400); // Maintains aspect ratio
}

#[test]
fn test_resize_image_smaller_than_max() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("small.jpg");
    let output_path = temp_dir.path().join("output.jpg");

    // Create a 400x200 test image (smaller than max)
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(400, 200);
    img.save(&input_path).unwrap();

    // Try to resize to max width 800
    let resized = resize_image(&input_path, &output_path, 800).unwrap();
    assert!(!resized); // Should not resize

    // Verify output dimensions unchanged
    let output_img = image::open(&output_path).unwrap();
    assert_eq!(output_img.width(), 400);
    assert_eq!(output_img.height(), 200);
}

#[test]
fn test_collect_banner_paths() {
    let posts = vec![
        ContentBuilder::new()
            .title("Post 1".to_string())
            .slug("post-1".to_string())
            .banner_image("media/banner1.jpg".to_string())
            .build(),
        ContentBuilder::new()
            .title("Post 2".to_string())
            .slug("post-2".to_string())
            .build(),
    ];

    let pages = vec![ContentBuilder::new()
        .title("Page 1".to_string())
        .slug("page-1".to_string())
        .banner_image("./media/page-banner.png".to_string())
        .build()];

    let banner_paths = collect_banner_paths_from_content(&posts, &pages);

    assert!(banner_paths.contains("banner1.jpg"));
    assert!(banner_paths.contains("page-banner.png"));
    assert_eq!(banner_paths.len(), 2);
}

#[test]
fn test_process_media_images_no_config() {
    let temp_dir = TempDir::new().unwrap();
    let media_path = temp_dir.path();
    let config = Marmite::new();
    let banner_paths = HashSet::new();

    // Should do nothing and not panic
    process_media_images(media_path, &config, &banner_paths);
}

#[test]
fn test_process_media_images_with_max_width() {
    let temp_dir = TempDir::new().unwrap();
    let media_path = temp_dir.path();

    // Create a large test image
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(1200, 800);
    img.save(media_path.join("large.jpg")).unwrap();

    // Create config with max_image_width in extra
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("max_image_width".to_string(), Value::Number(800.into()));
    config.extra = Some(extra);

    let banner_paths = HashSet::new();
    process_media_images(media_path, &config, &banner_paths);

    // Verify image was resized
    let resized_img = image::open(media_path.join("large.jpg")).unwrap();
    assert_eq!(resized_img.width(), 800);
}

#[test]
fn test_process_media_images_banner_detection() {
    let temp_dir = TempDir::new().unwrap();
    let media_path = temp_dir.path();

    // Create a banner image (by filename pattern)
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(2000, 600);
    img.save(media_path.join("post.banner.jpg")).unwrap();

    // Create config with banner_image_width in extra
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("banner_image_width".to_string(), Value::Number(1024.into()));
    extra.insert("max_image_width".to_string(), Value::Number(800.into()));
    config.extra = Some(extra);

    let banner_paths = HashSet::new();
    process_media_images(media_path, &config, &banner_paths);

    // Verify banner was resized to banner width (not max width)
    let resized_img = image::open(media_path.join("post.banner.jpg")).unwrap();
    // Allow 1px tolerance due to rounding
    assert!(resized_img.width() >= 1023 && resized_img.width() <= 1024);
}

#[test]
fn test_process_media_images_banner_from_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let media_path = temp_dir.path();

    // Create a regular named image that is referenced as banner in frontmatter
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(2000, 600);
    img.save(media_path.join("hero.jpg")).unwrap();

    // Create config with banner_image_width in extra
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("banner_image_width".to_string(), Value::Number(1024.into()));
    extra.insert("max_image_width".to_string(), Value::Number(800.into()));
    config.extra = Some(extra);

    // Simulate frontmatter reference
    let mut banner_paths = HashSet::new();
    banner_paths.insert("hero.jpg".to_string());

    process_media_images(media_path, &config, &banner_paths);

    // Verify image was resized to banner width (allow 1px tolerance)
    let resized_img = image::open(media_path.join("hero.jpg")).unwrap();
    assert!(resized_img.width() >= 1023 && resized_img.width() <= 1024);
}
