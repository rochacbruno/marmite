use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_gallery_order_default() {
    assert_eq!(GalleryOrder::default(), GalleryOrder::Asc);
}

#[test]
fn test_is_image_file() {
    assert!(is_image_file(Path::new("test.jpg")));
    assert!(is_image_file(Path::new("test.JPEG")));
    assert!(is_image_file(Path::new("test.png")));
    assert!(is_image_file(Path::new("test.webp")));
    assert!(is_image_file(Path::new("test.gif")));
    assert!(is_image_file(Path::new("test.bmp")));
    assert!(is_image_file(Path::new("test.tiff")));

    assert!(!is_image_file(Path::new("test.txt")));
    assert!(!is_image_file(Path::new("test")));
    assert!(!is_image_file(Path::new("test.mp4")));
}

#[test]
fn test_load_gallery_config_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("gallery.yaml");

    let config = load_gallery_config(&config_path);
    assert!(config.name.is_none());
    assert!(config.ord.is_none());
    assert!(config.cover.is_none());
    assert!(config.images.is_none());
}

#[test]
fn test_load_gallery_config_valid() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("gallery.yaml");

    let yaml_content = r#"
name: "My Amazing Summer"
ord: desc
cover: "sunset.jpg"
"#;
    fs::write(&config_path, yaml_content).unwrap();

    let config = load_gallery_config(&config_path);
    assert_eq!(config.name, Some("My Amazing Summer".to_string()));
    assert_eq!(config.ord, Some(GalleryOrder::Desc));
    assert_eq!(config.cover, Some("sunset.jpg".to_string()));
}

#[test]
fn test_process_galleries_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let media_path = temp_dir.path();
    fs::create_dir(media_path.join("gallery")).unwrap();

    let galleries = process_galleries(media_path, "gallery", false, 50);
    assert!(galleries.is_empty());
}

#[test]
fn test_process_single_gallery_without_images() {
    let temp_dir = TempDir::new().unwrap();
    let gallery_path = temp_dir.path();

    let gallery = process_single_gallery(gallery_path, "test", false, 50);
    assert_eq!(gallery.name, "test");
    assert!(gallery.files.is_empty());
    assert_eq!(gallery.cover, "");
    assert_eq!(gallery.ord, GalleryOrder::Asc);
}

#[test]
fn test_process_single_gallery_with_config() {
    let temp_dir = TempDir::new().unwrap();
    let gallery_path = temp_dir.path();

    let config_content = r#"
name: "Test Gallery"
ord: desc
cover: "main.jpg"
"#;
    fs::write(gallery_path.join("gallery.yaml"), config_content).unwrap();

    let gallery = process_single_gallery(gallery_path, "test", false, 50);
    assert_eq!(gallery.name, "Test Gallery");
    assert_eq!(gallery.ord, GalleryOrder::Desc);
    assert_eq!(gallery.cover, "main.jpg");
}

#[test]
fn test_copy_galleries() {
    let temp_dir = TempDir::new().unwrap();
    let input_media = temp_dir.path().join("input_media");
    let output_media = temp_dir.path().join("output_media");
    let gallery_dir = input_media.join("gallery").join("summer");

    fs::create_dir_all(&gallery_dir).unwrap();
    fs::write(gallery_dir.join("test.jpg"), "fake image data").unwrap();
    fs::create_dir_all(&output_media).unwrap();

    copy_galleries(&input_media, &output_media, "gallery");

    let copied_file = output_media.join("gallery").join("summer").join("test.jpg");
    assert!(copied_file.exists());
}

#[test]
fn test_process_galleries_with_images() {
    use image::{ImageBuffer, Rgb};

    let temp_dir = TempDir::new().unwrap();
    let media_path = temp_dir.path();
    let gallery_dir = media_path.join("gallery").join("test-gallery");
    fs::create_dir_all(&gallery_dir).unwrap();

    // Create a small test image
    let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(100, 100);
    img.save(gallery_dir.join("test1.jpg")).unwrap();
    img.save(gallery_dir.join("test2.png")).unwrap();

    // Create gallery config
    let config_content = r#"
name: "Test Gallery"
ord: asc
cover: "test1.jpg"
"#;
    fs::write(gallery_dir.join("gallery.yaml"), config_content).unwrap();

    let galleries = process_galleries(media_path, "gallery", true, 50);

    assert_eq!(galleries.len(), 1);
    assert!(galleries.contains_key("test-gallery"));

    let gallery = &galleries["test-gallery"];
    assert_eq!(gallery.name, "Test Gallery");
    assert_eq!(gallery.cover, "test1.jpg");
    assert_eq!(gallery.ord, GalleryOrder::Asc);
    assert_eq!(gallery.files.len(), 2);

    // Check thumbnails were created in thumbnails directory
    assert!(gallery_dir.join("thumbnails").join("test1.jpg").exists());
    assert!(gallery_dir.join("thumbnails").join("test2.png").exists());
}

#[test]
fn test_generate_thumbnail() {
    use image::{ImageBuffer, Rgb};

    let temp_dir = TempDir::new().unwrap();
    let image_path = temp_dir.path().join("test.jpg");
    let thumbnails_dir = temp_dir.path().join("thumbnails");
    fs::create_dir(&thumbnails_dir).unwrap();

    // Create a test image
    let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(200, 200);
    img.save(&image_path).unwrap();

    let thumb_name = generate_thumbnail(&image_path, &thumbnails_dir, 50);

    assert!(thumb_name.is_some());
    assert_eq!(thumb_name.unwrap(), "test.jpg");
    assert!(thumbnails_dir.join("test.jpg").exists());
}

#[test]
fn test_generate_thumbnail_existing() {
    use image::{ImageBuffer, Rgb};

    let temp_dir = TempDir::new().unwrap();
    let image_path = temp_dir.path().join("test.jpg");
    let thumbnails_dir = temp_dir.path().join("thumbnails");
    fs::create_dir(&thumbnails_dir).unwrap();
    let thumb_path = thumbnails_dir.join("test.jpg");

    // Create a test image
    let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(200, 200);
    img.save(&image_path).unwrap();

    // Create existing thumbnail
    let thumb = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(50, 50);
    thumb.save(&thumb_path).unwrap();

    let thumb_name = generate_thumbnail(&image_path, &thumbnails_dir, 50);

    assert!(thumb_name.is_some());
    assert_eq!(thumb_name.unwrap(), "test.jpg");
}
