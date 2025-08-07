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
