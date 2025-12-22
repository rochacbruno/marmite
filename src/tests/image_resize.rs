use super::*;
use crate::config::Marmite;
use crate::content::ContentBuilder;
use image::{ImageBuffer, Rgb};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use tempfile::TempDir;

#[test]
fn test_validate_width_valid_values() {
    assert_eq!(validate_width(1, "test"), Some(1));
    assert_eq!(validate_width(800, "test"), Some(800));
    assert_eq!(validate_width(10000, "test"), Some(10000));
}

#[test]
fn test_validate_width_invalid_zero() {
    assert_eq!(validate_width(0, "test"), None);
}

#[test]
fn test_validate_width_invalid_too_large() {
    assert_eq!(validate_width(10001, "test"), None);
    assert_eq!(validate_width(u32::MAX, "test"), None);
}

#[test]
fn test_get_resize_settings_valid_config() {
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("max_image_width".to_string(), Value::Number(800.into()));
    extra.insert("banner_image_width".to_string(), Value::Number(1200.into()));
    config.extra = Some(extra);

    let settings = get_resize_settings(&config);
    assert_eq!(settings.banner_width, Some(1200));
    assert_eq!(settings.max_width, Some(800));
    assert!(matches!(settings.filter, FilterType::Lanczos3)); // Default filter
}

#[test]
fn test_get_resize_settings_invalid_zero_width() {
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("max_image_width".to_string(), Value::Number(0.into()));
    config.extra = Some(extra);

    let settings = get_resize_settings(&config);
    assert_eq!(settings.max_width, None);
}

#[test]
fn test_get_resize_settings_invalid_too_large() {
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("max_image_width".to_string(), Value::Number(99999.into()));
    config.extra = Some(extra);

    let settings = get_resize_settings(&config);
    assert_eq!(settings.max_width, None);
}

#[test]
fn test_get_resize_settings_no_extra() {
    let config = Marmite::new();
    let settings = get_resize_settings(&config);
    assert_eq!(settings.banner_width, None);
    assert_eq!(settings.max_width, None);
    assert!(matches!(settings.filter, FilterType::Lanczos3)); // Default filter
}

#[test]
fn test_parse_resize_filter_valid_options() {
    assert!(matches!(
        parse_resize_filter(Some("fast")),
        FilterType::Triangle
    ));
    assert!(matches!(
        parse_resize_filter(Some("balanced")),
        FilterType::CatmullRom
    ));
    assert!(matches!(
        parse_resize_filter(Some("quality")),
        FilterType::Lanczos3
    ));
}

#[test]
fn test_parse_resize_filter_default() {
    assert!(matches!(parse_resize_filter(None), FilterType::Lanczos3));
}

#[test]
fn test_parse_resize_filter_invalid() {
    // Invalid values should fall back to Lanczos3
    assert!(matches!(
        parse_resize_filter(Some("invalid")),
        FilterType::Lanczos3
    ));
    assert!(matches!(
        parse_resize_filter(Some("")),
        FilterType::Lanczos3
    ));
}

#[test]
fn test_get_resize_settings_with_filter() {
    let mut config = Marmite::new();
    let mut extra = HashMap::new();
    extra.insert("max_image_width".to_string(), Value::Number(800.into()));
    extra.insert(
        "resize_filter".to_string(),
        Value::String("fast".to_string()),
    );
    config.extra = Some(extra);

    let settings = get_resize_settings(&config);
    assert_eq!(settings.max_width, Some(800));
    assert!(matches!(settings.filter, FilterType::Triangle));
}

// Path normalization tests
#[test]
fn test_normalize_banner_path_simple_media_prefix() {
    assert_eq!(normalize_banner_path("media/file.jpg"), "file.jpg");
    assert_eq!(
        normalize_banner_path("media/subdir/file.jpg"),
        "subdir/file.jpg"
    );
}

#[test]
fn test_normalize_banner_path_relative_with_dot() {
    assert_eq!(normalize_banner_path("./media/file.jpg"), "file.jpg");
    assert_eq!(
        normalize_banner_path("./media/subdir/file.jpg"),
        "subdir/file.jpg"
    );
}

#[test]
fn test_normalize_banner_path_parent_dir() {
    assert_eq!(normalize_banner_path("../media/file.jpg"), "file.jpg");
    assert_eq!(normalize_banner_path("../../media/file.jpg"), "file.jpg");
}

#[test]
fn test_normalize_banner_path_windows_backslashes() {
    assert_eq!(normalize_banner_path("media\\file.jpg"), "file.jpg");
    assert_eq!(
        normalize_banner_path("media\\subdir\\file.jpg"),
        "subdir/file.jpg"
    );
    assert_eq!(normalize_banner_path(".\\media\\file.jpg"), "file.jpg");
}

#[test]
fn test_normalize_banner_path_no_media_prefix() {
    assert_eq!(normalize_banner_path("file.jpg"), "file.jpg");
    assert_eq!(normalize_banner_path("subdir/file.jpg"), "subdir/file.jpg");
}

#[test]
fn test_normalize_banner_path_case_insensitive_media() {
    assert_eq!(normalize_banner_path("Media/file.jpg"), "file.jpg");
    assert_eq!(normalize_banner_path("MEDIA/file.jpg"), "file.jpg");
}

#[test]
fn test_normalize_banner_path_only_media() {
    // Edge case: path is just "media" with no file
    assert_eq!(normalize_banner_path("media"), "");
    assert_eq!(normalize_banner_path("./media"), "");
}

#[test]
fn test_normalize_banner_path_mixed_separators() {
    assert_eq!(
        normalize_banner_path("./media\\subdir/file.jpg"),
        "subdir/file.jpg"
    );
}

#[test]
fn test_is_image_file() {
    // Supported raster formats
    assert!(is_image_file(Path::new("test.jpg")));
    assert!(is_image_file(Path::new("test.JPEG")));
    assert!(is_image_file(Path::new("test.png")));
    assert!(is_image_file(Path::new("test.webp")));
    assert!(is_image_file(Path::new("test.gif")));
    assert!(is_image_file(Path::new("test.bmp")));
    assert!(is_image_file(Path::new("test.tiff")));
    assert!(is_image_file(Path::new("test.avif")));
    assert!(is_image_file(Path::new("test.AVIF")));

    // Non-image files
    assert!(!is_image_file(Path::new("test.txt")));
    assert!(!is_image_file(Path::new("test.md")));

    // Vector/icon formats should NOT be detected as resizable images
    assert!(!is_image_file(Path::new("test.svg")));
    assert!(!is_image_file(Path::new("test.ico")));
}

#[test]
fn test_is_vector_or_icon_image() {
    // Vector and icon formats
    assert!(is_vector_or_icon_image(Path::new("logo.svg")));
    assert!(is_vector_or_icon_image(Path::new("logo.SVG")));
    assert!(is_vector_or_icon_image(Path::new("favicon.ico")));
    assert!(is_vector_or_icon_image(Path::new("favicon.ICO")));

    // Raster formats should NOT be detected as vector/icon
    assert!(!is_vector_or_icon_image(Path::new("test.jpg")));
    assert!(!is_vector_or_icon_image(Path::new("test.png")));
    assert!(!is_vector_or_icon_image(Path::new("test.avif")));

    // Non-image files
    assert!(!is_vector_or_icon_image(Path::new("test.txt")));
    assert!(!is_vector_or_icon_image(Path::new("test.html")));
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
    let resized = resize_image(&input_path, &output_path, 800, FilterType::Lanczos3).unwrap();
    assert!(resized);

    // Verify output dimensions
    let output_img = image::open(&output_path).unwrap();
    assert_eq!(output_img.width(), 800);
    assert_eq!(output_img.height(), 400); // Maintains aspect ratio
}

#[test]
fn test_resize_image_in_place_atomic() {
    let temp_dir = TempDir::new().unwrap();
    let image_path = temp_dir.path().join("inplace.jpg");

    // Create a 1000x500 test image
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(1000, 500);
    img.save(&image_path).unwrap();

    // Get original file metadata for comparison
    let original_metadata = std::fs::metadata(&image_path).unwrap();
    let original_size = original_metadata.len();

    // Resize in-place (input_path == output_path)
    let resized = resize_image(&image_path, &image_path, 800, FilterType::Lanczos3).unwrap();
    assert!(resized);

    // Verify the file was replaced atomically
    let resized_img = image::open(&image_path).unwrap();
    assert_eq!(resized_img.width(), 800);
    assert_eq!(resized_img.height(), 400);

    // Verify no temp files remain in the directory
    let remaining_files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(
        remaining_files.len(),
        1,
        "Only the resized image should remain"
    );

    // Verify file size changed (resized should be different from original)
    let new_size = std::fs::metadata(&image_path).unwrap().len();
    assert_ne!(
        original_size, new_size,
        "File size should change after resize"
    );
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
    let resized = resize_image(&input_path, &output_path, 800, FilterType::Lanczos3).unwrap();
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
