// Note: These integration tests require marmite to be a library crate.
// Since marmite is currently a binary-only crate, these tests cannot access
// internal modules directly. To properly test the binary, we would need to:
// 1. Either convert marmite to have both lib.rs and main.rs
// 2. Or use command-line testing with std::process::Command
//
// For now, these tests are commented out but preserved for future implementation.

/*
use marmite::site;
use marmite::config::Marmite;
use marmite::cli::Cli;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_minimal_site_generation() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create input directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config file
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "title: Test Site\ntagline: Test").unwrap();

    // Create a simple content file
    let content_path = input_dir.join("content").join("test.md");
    fs::write(&content_path, "# Test Page\n\nThis is a test.").unwrap();

    // Generate site
    let cli_args = Arc::new(Cli::default());
    let result = site::generate(
        &Arc::new(input_dir.clone()),
        &Arc::new(output_dir.clone()),
        &Arc::new(config_path),
        &cli_args,
        false,
        false,
    );

    assert!(result.is_ok());

    // Verify output
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("test.html").exists());
}

// ... other tests omitted for brevity ...
*/

// Example of how to test the binary using Command:
#[test]
fn test_marmite_binary_help() {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute marmite");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("marmite"));
}
