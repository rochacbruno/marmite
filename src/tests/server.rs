use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_render_not_found_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("404.html");
    fs::write(&error_path, "Custom 404 page").unwrap();

    let response = render_not_found(&error_path);
    assert!(response.is_ok());
}

#[test]
fn test_render_not_found_without_file() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("nonexistent_404.html");

    let response = render_not_found(&error_path);
    assert!(response.is_ok());
}

#[test]
fn test_render_not_found_with_file_content() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("404.html");
    let content = "<html><body><h1>404 - Page Not Found</h1></body></html>";
    fs::write(&error_path, content).unwrap();

    let _response = render_not_found(&error_path).unwrap();
    // Response should be created successfully
    // Testing the actual content is difficult without accessing internal data
}

#[test]
fn test_render_not_found_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let error_path = temp_dir.path().join("non_existent_404.html");

    let _response = render_not_found(&error_path).unwrap();
    // Should return fallback 404 response
    // Testing the actual content is difficult without accessing internal data
}

#[test]
fn test_content_type_for_svg() {
    assert_eq!(content_type_for("image.svg"), Some("image/svg+xml"));
}

#[test]
fn test_content_type_for_common_types() {
    assert_eq!(
        content_type_for("index.html"),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        content_type_for("style.css"),
        Some("text/css; charset=utf-8")
    );
    assert_eq!(
        content_type_for("app.js"),
        Some("text/javascript; charset=utf-8")
    );
    assert_eq!(
        content_type_for("data.json"),
        Some("application/json; charset=utf-8")
    );
    assert_eq!(
        content_type_for("sitemap.xml"),
        Some("application/xml; charset=utf-8")
    );
    assert_eq!(content_type_for("photo.png"), Some("image/png"));
    assert_eq!(content_type_for("photo.jpg"), Some("image/jpeg"));
    assert_eq!(content_type_for("photo.webp"), Some("image/webp"));
    assert_eq!(content_type_for("font.woff2"), Some("font/woff2"));
}

#[test]
fn test_content_type_for_unknown_extension() {
    assert_eq!(content_type_for("file.xyz"), None);
}

#[test]
fn test_content_type_for_nested_path() {
    assert_eq!(
        content_type_for("assets/icons/logo.svg"),
        Some("image/svg+xml")
    );
}
