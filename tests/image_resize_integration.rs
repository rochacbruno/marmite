use image::{GenericImageView, ImageBuffer, Rgb};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test image with specific dimensions
fn create_test_image(path: &std::path::Path, width: u32, height: u32) {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    img.save(path).unwrap();
}

/// Helper to get image dimensions
fn get_image_dimensions(path: &std::path::Path) -> (u32, u32) {
    let img = image::open(path).unwrap();
    img.dimensions()
}

#[test]
fn test_image_resize_integration_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();

    // Create config with image resize settings
    let config = r#"
name: Test Site
extra:
  max_image_width: 800
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Create a large test image (1200x600)
    let image_path = input_dir.join("content").join("media").join("large.jpg");
    create_test_image(&image_path, 1200, 600);

    // Create a small test image (400x200) - should NOT be resized
    let small_image_path = input_dir.join("content").join("media").join("small.jpg");
    create_test_image(&small_image_path, 400, 200);

    // Create a content file
    fs::write(
        input_dir.join("content").join("post.md"),
        "---\ntitle: Test Post\n---\n# Test\n![Image](media/large.jpg)",
    )
    .unwrap();

    // Generate site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify large image was resized to 800px width
    let output_large = output_dir.join("media").join("large.jpg");
    assert!(output_large.exists(), "Large image should exist in output");
    let (width, height) = get_image_dimensions(&output_large);
    assert_eq!(width, 800, "Large image should be resized to 800px width");
    assert_eq!(height, 400, "Height should maintain aspect ratio");

    // Verify small image was NOT resized (already smaller than max)
    let output_small = output_dir.join("media").join("small.jpg");
    assert!(output_small.exists(), "Small image should exist in output");
    let (width, height) = get_image_dimensions(&output_small);
    assert_eq!(width, 400, "Small image should remain 400px width");
    assert_eq!(height, 200, "Small image height should be unchanged");

    // Verify source images are unchanged
    let (src_width, _) = get_image_dimensions(&image_path);
    assert_eq!(src_width, 1200, "Source image should be unchanged");
}

#[test]
fn test_image_resize_integration_banner_detection() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();

    // Create config with separate banner and regular image widths
    let config = r#"
name: Test Site
extra:
  max_image_width: 600
  banner_image_width: 1000
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Create a banner image (detected by filename pattern)
    let banner_path = input_dir
        .join("content")
        .join("media")
        .join("hero.banner.jpg");
    create_test_image(&banner_path, 1500, 500);

    // Create a regular image
    let regular_path = input_dir.join("content").join("media").join("photo.jpg");
    create_test_image(&regular_path, 1500, 1000);

    // Create a content file
    fs::write(
        input_dir.join("content").join("post.md"),
        "---\ntitle: Test Post\n---\n# Test",
    )
    .unwrap();

    // Generate site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify banner image was resized to banner_image_width (1000px)
    // Allow 1px tolerance due to aspect ratio rounding
    let output_banner = output_dir.join("media").join("hero.banner.jpg");
    assert!(
        output_banner.exists(),
        "Banner image should exist in output"
    );
    let (width, _) = get_image_dimensions(&output_banner);
    assert!(
        (999..=1000).contains(&width),
        "Banner image should be resized to ~1000px width, got {width}"
    );

    // Verify regular image was resized to max_image_width (600px)
    let output_regular = output_dir.join("media").join("photo.jpg");
    assert!(
        output_regular.exists(),
        "Regular image should exist in output"
    );
    let (width, _) = get_image_dimensions(&output_regular);
    assert_eq!(width, 600, "Regular image should be resized to 600px width");
}

#[test]
fn test_image_resize_integration_frontmatter_banner() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();

    // Create config with separate banner and regular image widths
    let config = r#"
name: Test Site
extra:
  max_image_width: 600
  banner_image_width: 1000
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Create an image that will be referenced as banner in frontmatter
    // (no .banner. in filename)
    let hero_path = input_dir.join("content").join("media").join("hero.jpg");
    create_test_image(&hero_path, 1500, 500);

    // Create a regular image (not referenced as banner)
    let regular_path = input_dir.join("content").join("media").join("inline.jpg");
    create_test_image(&regular_path, 1500, 1000);

    // Create a content file with banner_image in frontmatter
    fs::write(
        input_dir.join("content").join("post.md"),
        "---\ntitle: Test Post\nbanner_image: media/hero.jpg\n---\n# Test\n![Inline](media/inline.jpg)",
    )
    .unwrap();

    // Generate site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify hero image (referenced in frontmatter) was resized to banner width
    // Allow 1px tolerance due to aspect ratio rounding
    let output_hero = output_dir.join("media").join("hero.jpg");
    assert!(output_hero.exists(), "Hero image should exist in output");
    let (width, _) = get_image_dimensions(&output_hero);
    assert!(
        (999..=1000).contains(&width),
        "Hero image (from frontmatter) should be resized to ~1000px width, got {width}"
    );

    // Verify inline image was resized to regular max width
    let output_inline = output_dir.join("media").join("inline.jpg");
    assert!(
        output_inline.exists(),
        "Inline image should exist in output"
    );
    let (width, _) = get_image_dimensions(&output_inline);
    assert_eq!(width, 600, "Inline image should be resized to 600px width");
}

#[test]
fn test_image_resize_integration_thumbnails_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure including thumbnails
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media").join("thumbnails")).unwrap();

    // Create config with image resize settings
    let config = r#"
name: Test Site
extra:
  max_image_width: 800
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Create a regular image
    let image_path = input_dir.join("content").join("media").join("photo.jpg");
    create_test_image(&image_path, 1200, 800);

    // Create a thumbnail image (should NOT be resized)
    let thumb_path = input_dir
        .join("content")
        .join("media")
        .join("thumbnails")
        .join("thumb.jpg");
    create_test_image(&thumb_path, 1200, 800);

    // Create a content file
    fs::write(
        input_dir.join("content").join("post.md"),
        "---\ntitle: Test Post\n---\n# Test",
    )
    .unwrap();

    // Generate site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify regular image was resized
    let output_photo = output_dir.join("media").join("photo.jpg");
    assert!(output_photo.exists(), "Photo should exist in output");
    let (width, _) = get_image_dimensions(&output_photo);
    assert_eq!(width, 800, "Photo should be resized to 800px width");

    // Verify thumbnail was NOT resized (thumbnails directory is skipped)
    let output_thumb = output_dir
        .join("media")
        .join("thumbnails")
        .join("thumb.jpg");
    assert!(output_thumb.exists(), "Thumbnail should exist in output");
    let (width, _) = get_image_dimensions(&output_thumb);
    assert_eq!(
        width, 1200,
        "Thumbnail should NOT be resized (directory skipped)"
    );
}

#[test]
fn test_image_resize_integration_no_config() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();

    // Create config WITHOUT image resize settings
    let config = r#"
name: Test Site
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Create a large test image
    let image_path = input_dir.join("content").join("media").join("large.jpg");
    create_test_image(&image_path, 2000, 1000);

    // Create a content file
    fs::write(
        input_dir.join("content").join("post.md"),
        "---\ntitle: Test Post\n---\n# Test",
    )
    .unwrap();

    // Generate site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify image was NOT resized (no config)
    let output_large = output_dir.join("media").join("large.jpg");
    assert!(output_large.exists(), "Image should exist in output");
    let (width, height) = get_image_dimensions(&output_large);
    assert_eq!(
        width, 2000,
        "Image should NOT be resized when no config is set"
    );
    assert_eq!(height, 1000, "Image height should be unchanged");
}

#[test]
fn test_image_resize_integration_multiple_formats() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();

    // Create config with image resize settings
    let config = r#"
name: Test Site
extra:
  max_image_width: 800
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();

    // Create test images in different formats
    let jpg_path = input_dir.join("content").join("media").join("image.jpg");
    create_test_image(&jpg_path, 1200, 600);

    let png_path = input_dir.join("content").join("media").join("image.png");
    create_test_image(&png_path, 1200, 600);

    // Create a content file
    fs::write(
        input_dir.join("content").join("post.md"),
        "---\ntitle: Test Post\n---\n# Test",
    )
    .unwrap();

    // Generate site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify JPG was resized
    let output_jpg = output_dir.join("media").join("image.jpg");
    assert!(output_jpg.exists(), "JPG image should exist in output");
    let (width, _) = get_image_dimensions(&output_jpg);
    assert_eq!(width, 800, "JPG should be resized to 800px width");

    // Verify PNG was resized
    let output_png = output_dir.join("media").join("image.png");
    assert!(output_png.exists(), "PNG image should exist in output");
    let (width, _) = get_image_dimensions(&output_png);
    assert_eq!(width, 800, "PNG should be resized to 800px width");
}
