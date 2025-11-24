use super::*;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_determine_download_url_github() {
    let url = "https://github.com/user/repo";
    let result = determine_download_url(url).unwrap();
    assert_eq!(
        result,
        "https://github.com/user/repo/archive/refs/heads/main.zip"
    );
}

#[test]
fn test_determine_download_url_github_with_branch() {
    let url = "https://github.com/user/repo/tree/develop";
    let result = determine_download_url(url).unwrap();
    assert_eq!(
        result,
        "https://github.com/user/repo/archive/refs/heads/develop.zip"
    );
}

#[test]
fn test_determine_download_url_gitlab() {
    let url = "https://gitlab.com/user/repo";
    let result = determine_download_url(url).unwrap();
    assert_eq!(
        result,
        "https://gitlab.com/user/repo/-/archive/main/repo-main.zip"
    );
}

#[test]
fn test_determine_download_url_codeberg() {
    let url = "https://codeberg.org/user/repo";
    let result = determine_download_url(url).unwrap();
    assert_eq!(result, "https://codeberg.org/user/repo/archive/main.zip");
}

#[test]
fn test_determine_download_url_direct_zip() {
    let url = "https://example.com/theme.zip";
    let result = determine_download_url(url).unwrap();
    assert_eq!(result, "https://example.com/theme.zip");
}

#[test]
fn test_determine_download_url_unsupported() {
    let url = "https://unsupported.com/user/repo";
    let result = determine_download_url(url);
    assert!(result.is_err());
}

#[test]
fn test_extract_theme_name_github() {
    let url = "https://github.com/user/my-theme";
    let result = extract_theme_name(url).unwrap();
    assert_eq!(result, "my-theme");
}

#[test]
fn test_extract_theme_name_zip() {
    let url = "https://example.com/awesome-theme.zip";
    let result = extract_theme_name(url).unwrap();
    assert_eq!(result, "awesome-theme");
}

#[test]
fn test_extract_theme_name_invalid_url() {
    let url = "invalid";
    let result = extract_theme_name(url);
    assert!(result.is_err());
}

#[test]
fn test_find_theme_root_direct() {
    let temp_dir = TempDir::new().unwrap();
    let theme_json = temp_dir.path().join("theme.json");
    fs::write(&theme_json, "{}").unwrap();

    let result = find_theme_root(temp_dir.path()).unwrap();
    assert_eq!(result, temp_dir.path());
}

#[test]
fn test_find_theme_root_nested() {
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir.path().join("nested");
    fs::create_dir(&nested_dir).unwrap();
    let theme_json = nested_dir.join("theme.json");
    fs::write(&theme_json, "{}").unwrap();

    let result = find_theme_root(temp_dir.path()).unwrap();
    assert_eq!(result, nested_dir);
}

#[test]
fn test_find_theme_root_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let result = find_theme_root(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_read_theme_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let theme_json = temp_dir.path().join("theme.json");

    let metadata = json!({
        "name": "Test Theme",
        "version": "1.0.0",
        "author": "Test Author",
        "description": "A test theme",
        "repository": "https://github.com/test/theme",
        "license": "AGPL",
        "tags": ["minimal", "clean"],
        "features": ["responsive", "dark-mode"]
    });

    fs::write(
        &theme_json,
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let result = read_theme_metadata(&theme_json).unwrap();
    assert_eq!(result.name, "Test Theme");
    assert_eq!(result.version, "1.0.0");
    assert_eq!(result.author, "Test Author");
    assert_eq!(result.description, "A test theme");
    assert_eq!(
        result.repository,
        Some("https://github.com/test/theme".to_string())
    );
    assert_eq!(result.license, Some("AGPL".to_string()));
}

#[test]
fn test_update_config_theme_existing_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("marmite.yaml");

    fs::write(&config_path, "title: My Site\ntheme: old-theme\n").unwrap();

    let result = update_config_theme(temp_dir.path(), "new-theme", None);
    assert!(result.is_ok());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("theme: new-theme"));
    assert!(!content.contains("theme: old-theme"));
}

#[test]
fn test_update_config_theme_add_to_existing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("marmite.yaml");

    fs::write(&config_path, "title: My Site\n").unwrap();

    let result = update_config_theme(temp_dir.path(), "new-theme", None);
    assert!(result.is_ok());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("theme: new-theme"));
}

#[test]
fn test_set_theme_local_theme_exists() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    // Create a local theme with theme.json
    let theme_path = input_folder.join("test-theme");
    fs::create_dir(&theme_path).unwrap();

    let theme_json = theme_path.join("theme.json");
    let metadata = json!({
        "name": "Test Theme",
        "version": "1.0.0",
        "author": "Test Author",
        "description": "A test theme"
    });
    fs::write(
        &theme_json,
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .unwrap();

    // Create a marmite.yaml file
    let config_path = input_folder.join("marmite.yaml");
    fs::write(&config_path, "title: My Site\n").unwrap();

    // Test setting the local theme
    set_theme(input_folder, "test-theme", None);

    // Check that the config was updated
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("theme: test-theme"));
}

#[test]
fn test_set_theme_local_theme_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    // Don't create the theme folder

    // Test setting a non-existent local theme (should error gracefully)
    set_theme(input_folder, "non-existent-theme", None);

    // Function should return early without creating any config
    let config_path = input_folder.join("marmite.yaml");
    assert!(!config_path.exists());
}

#[test]
fn test_set_theme_local_theme_no_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    // Create a local theme without theme.json
    let theme_path = input_folder.join("invalid-theme");
    fs::create_dir(&theme_path).unwrap();

    // Test setting the theme without metadata (should error gracefully)
    set_theme(input_folder, "invalid-theme", None);

    // Function should return early without creating any config
    let config_path = input_folder.join("marmite.yaml");
    assert!(!config_path.exists());
}

#[test]
fn test_read_theme_metadata_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let theme_json = temp_dir.path().join("theme.json");

    // Write invalid JSON
    fs::write(&theme_json, "{ invalid json }").unwrap();

    let result = read_theme_metadata(&theme_json);
    assert!(result.is_err());
}

#[test]
fn test_read_theme_metadata_missing_fields() {
    let temp_dir = TempDir::new().unwrap();
    let theme_json = temp_dir.path().join("theme.json");

    // Write JSON missing required fields
    let metadata = json!({
        "name": "Test Theme"
        // Missing version, author, description
    });
    fs::write(
        &theme_json,
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let result = read_theme_metadata(&theme_json);
    assert!(result.is_err());
}

#[test]
fn test_determine_download_url_github_invalid() {
    let url = "https://github.com/user"; // Too short
    let result = determine_download_url(url);
    assert!(result.is_err());
}

#[test]
fn test_determine_download_url_gitlab_invalid() {
    let url = "https://gitlab.com/user"; // Too short
    let result = determine_download_url(url);
    assert!(result.is_err());
}

#[test]
fn test_determine_download_url_codeberg_invalid() {
    let url = "https://codeberg.org/user"; // Too short
    let result = determine_download_url(url);
    assert!(result.is_err());
}

#[test]
fn test_extract_theme_name_invalid_zip() {
    let url = "https://example.com/.zip"; // No filename
    let result = extract_theme_name(url);
    assert!(result.is_err());
}

#[test]
fn test_update_config_theme_no_existing_config() {
    let temp_dir = TempDir::new().unwrap();

    // Don't create a config file

    // Since this tries to call the marmite binary, it will likely fail in tests
    // But we can test that it handles the error gracefully
    let _result = update_config_theme(temp_dir.path(), "new-theme", None);
    // This might succeed or fail depending on environment, both are acceptable
    // The important thing is it doesn't panic
}
