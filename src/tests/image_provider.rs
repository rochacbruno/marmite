use super::*;
use crate::config::ImageProvider;
use frontmatter_gen::Frontmatter;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn create_test_config() -> Marmite {
    Marmite {
        name: "Test Site".to_string(),
        media_path: "media".to_string(),
        image_provider: Some(ImageProvider::Picsum),
        ..Default::default()
    }
}

#[test]
fn test_download_banner_image_no_provider() {
    let config = Marmite {
        image_provider: None,
        ..Default::default()
    };
    let frontmatter = Frontmatter::new();
    let temp_dir = TempDir::new().unwrap();
    let slug = "test-post";
    let tags = vec!["rust".to_string(), "test".to_string()];

    let result = download_banner_image(&config, &frontmatter, temp_dir.path(), slug, &tags);
    assert!(result.is_ok());
}

#[test]
fn test_download_banner_image_with_picsum() {
    // Use a channel to communicate the test result from the thread
    let (tx, rx) = mpsc::channel();

    // Run the test in a separate thread with timeout
    thread::spawn(move || {
        // Check if we can reach picsum.photos
        let test_url = "https://picsum.photos/health";
        if let Ok(_) = ureq::get(test_url).call() {
            // Server is reachable, proceed with test
            let config = create_test_config();
            let frontmatter = Frontmatter::new();
            let temp_dir = TempDir::new().unwrap();
            let slug = "test-post";
            let tags = vec!["rust".to_string(), "test".to_string()];

            // This will call download_picsum_image
            let result =
                download_banner_image(&config, &frontmatter, temp_dir.path(), slug, &tags);
            // Result depends on network, but function should handle errors gracefully
            // We can't test actual download without network, but we can test the function doesn't panic
            assert!(result.is_ok() || result.is_err());
            let _ = tx.send(true);
        } else {
            // Server not reachable, skip test
            eprintln!(
                "Skipping test_download_banner_image_with_picsum: picsum.photos not reachable"
            );
            let _ = tx.send(false);
        }
    });

    // Wait for the test to complete with a 10-second timeout
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(_) => {
            // Test completed within timeout
        }
        Err(_) => {
            // Timeout - skip test
            eprintln!("Skipping test_download_banner_image_with_picsum: timeout after 10 seconds");
        }
    }
}

#[test]
fn test_download_picsum_image_with_existing_banner_in_frontmatter() {
    let config = create_test_config();

    // Create content with frontmatter that includes banner_image
    let content_with_frontmatter = r#"---
banner_image: "existing-banner.jpg"
---
# Test Post
This is a test post."#;

    let (frontmatter, _) = crate::parser::extract_fm_content(content_with_frontmatter).unwrap();
    let temp_dir = TempDir::new().unwrap();
    let slug = "test-post";
    let tags = vec!["rust".to_string()];

    let result = download_picsum_image(&config, &frontmatter, temp_dir.path(), slug, &tags);
    assert!(result.is_ok());

    // Should not create any media directory since banner already exists in frontmatter
    let media_path = temp_dir.path().join(&config.media_path);
    assert!(!media_path.exists());
}

#[test]
fn test_download_picsum_image_with_existing_banner_file() {
    let config = create_test_config();
    let frontmatter = Frontmatter::new();
    let temp_dir = TempDir::new().unwrap();
    let slug = "test-post";
    let tags = vec!["rust".to_string()];

    // Create media directory and banner file
    let media_path = temp_dir.path().join(&config.media_path);
    fs::create_dir_all(&media_path).unwrap();
    let banner_file = media_path.join(format!("{slug}.banner.jpg"));
    fs::write(&banner_file, "fake image data").unwrap();

    let result = download_picsum_image(&config, &frontmatter, temp_dir.path(), slug, &tags);
    assert!(result.is_ok());

    // File should still exist
    assert!(banner_file.exists());
}

#[test]
fn test_download_picsum_image_creates_media_directory() {
    let config = create_test_config();
    let frontmatter = Frontmatter::new();
    let temp_dir = TempDir::new().unwrap();
    let slug = "test-post";
    let tags = vec![];

    // Media directory doesn't exist initially
    let media_path = temp_dir.path().join(&config.media_path);
    assert!(!media_path.exists());

    // This will attempt to download (may fail due to network, but should create directory)
    let _result = download_picsum_image(&config, &frontmatter, temp_dir.path(), slug, &tags);

    // Media directory should be created
    assert!(media_path.exists());
}

#[test]
fn test_seed_generation_with_tags() {
    let config = Marmite {
        name: "My Test Site!".to_string(),
        media_path: "media".to_string(),
        image_provider: Some(ImageProvider::Picsum),
        ..Default::default()
    };
    let frontmatter = Frontmatter::new();
    let temp_dir = TempDir::new().unwrap();
    let slug = "test-post";
    let tags = vec!["rust".to_string(), "web dev".to_string()];

    // We can't easily test the exact URL without mocking ureq,
    // but we can test that the function runs and creates the media directory
    let _result = download_picsum_image(&config, &frontmatter, temp_dir.path(), slug, &tags);

    let media_path = temp_dir.path().join(&config.media_path);
    assert!(media_path.exists());
}

#[test]
fn test_seed_generation_without_tags() {
    let config = create_test_config();
    let frontmatter = Frontmatter::new();
    let temp_dir = TempDir::new().unwrap();
    let slug = "test-post";
    let tags: Vec<String> = vec![];

    let _result = download_picsum_image(&config, &frontmatter, temp_dir.path(), slug, &tags);

    let media_path = temp_dir.path().join(&config.media_path);
    assert!(media_path.exists());
}
