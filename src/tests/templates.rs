use super::*;
use tempfile::TempDir;

#[test]
fn test_initialize_templates() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    initialize_templates(input_folder);

    let templates_path = input_folder.join("templates");
    assert!(templates_path.exists());
    assert!(templates_path.is_dir());
}

#[test]
fn test_initialize_theme_assets() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    initialize_theme_assets(input_folder);

    let static_path = input_folder.join("static");
    assert!(static_path.exists());
    assert!(static_path.is_dir());
}

#[test]
fn test_initialize_theme_valid_name() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();
    let theme_name = "my_theme";

    initialize_theme(input_folder, theme_name);

    let theme_path = input_folder.join(theme_name);
    assert!(theme_path.exists());
    assert!(theme_path.is_dir());

    let templates_path = theme_path.join("templates");
    let static_path = theme_path.join("static");
    assert!(templates_path.exists());
    assert!(static_path.exists());
}

#[test]
fn test_initialize_theme_invalid_name() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();

    // Test empty name - should not create any directory
    let original_entries = std::fs::read_dir(input_folder).unwrap().count();
    initialize_theme(input_folder, "");
    let new_entries = std::fs::read_dir(input_folder).unwrap().count();
    assert_eq!(original_entries, new_entries);

    // Test name with slash
    initialize_theme(input_folder, "invalid/theme");
    assert!(!input_folder.join("invalid/theme").exists());

    // Test name with backslash
    initialize_theme(input_folder, "invalid\\theme");
    assert!(!input_folder.join("invalid\\theme").exists());
}

#[test]
fn test_initialize_theme_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let input_folder = temp_dir.path();
    let theme_name = "existing_theme";
    let theme_path = input_folder.join(theme_name);

    // Create the theme directory first
    fs::create_dir(&theme_path).unwrap();

    // Try to initialize theme with existing directory
    initialize_theme(input_folder, theme_name);

    // Should still exist but not be modified
    assert!(theme_path.exists());
}
