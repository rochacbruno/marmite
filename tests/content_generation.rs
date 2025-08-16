use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_site_generation_with_posts() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config with pagination
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog\npagination: 2").unwrap();

    // Create multiple posts with dates
    let post1 = r"---
date: 2024-01-01
---
# First Post
Content 1";
    fs::write(input_dir.join("content").join("2024-01-01-post1.md"), post1).unwrap();

    let post2 = r"---
date: 2024-01-02
---
# Second Post
Content 2";
    fs::write(input_dir.join("content").join("2024-01-02-post2.md"), post2).unwrap();

    let post3 = r"---
date: 2024-01-03
---
# Third Post
Content 3";
    fs::write(input_dir.join("content").join("2024-01-03-post3.md"), post3).unwrap();

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

    // Verify posts were generated (using slug from filename without date prefix)
    assert!(output_dir.join("post1.html").exists());
    assert!(output_dir.join("post2.html").exists());
    assert!(output_dir.join("post3.html").exists());

    // Verify pagination
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("index-2.html").exists());
}

#[test]
fn test_site_generation_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();

    // Create posts with tags
    let post1 = r"---
date: 2024-01-01
tags: [rust, programming]
---
# Rust Post";
    fs::write(input_dir.join("content").join("rust-post.md"), post1).unwrap();

    let post2 = r"---
date: 2024-01-02
tags: [rust, testing]
---
# Testing Post";
    fs::write(input_dir.join("content").join("test-post.md"), post2).unwrap();

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

    // Verify tag pages were generated
    assert!(output_dir.join("tag-rust.html").exists());
    assert!(output_dir.join("tag-programming.html").exists());
    assert!(output_dir.join("tag-testing.html").exists());
    assert!(output_dir.join("tags.html").exists());
}

#[test]
fn test_draft_posts_excluded() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();

    // Create a normal post
    let post = r"---
date: 2024-01-01
---
# Published Post";
    fs::write(input_dir.join("content").join("post.md"), post).unwrap();

    // Create a draft post with draft- prefix
    let draft = r"---
date: 2024-01-02
---
# Draft Post";
    fs::write(
        input_dir.join("content").join("draft-2024-01-02-draft.md"),
        draft,
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

    // Verify published post exists
    assert!(output_dir.join("post.html").exists());

    // Verify draft post does NOT exist
    assert!(!output_dir.join("draft-2024-01-02-draft.html").exists());
}

#[test]
fn test_date_prefixed_filename_to_slug() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();

    // Create posts with date-prefixed filenames
    // Date should be extracted from filename and slug should not include date
    let post1 = "# First Post\nContent here";
    fs::write(
        input_dir
            .join("content")
            .join("2024-01-15-my-first-post.md"),
        post1,
    )
    .unwrap();

    let post2 = "# Second Post\nMore content";
    fs::write(
        input_dir.join("content").join("2024-02-20-another-post.md"),
        post2,
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

    // Verify posts are generated without date prefix in filename
    assert!(output_dir.join("my-first-post.html").exists());
    assert!(output_dir.join("another-post.html").exists());

    // Verify date-prefixed files are NOT generated
    assert!(!output_dir.join("2024-01-15-my-first-post.html").exists());
    assert!(!output_dir.join("2024-02-20-another-post.html").exists());
}

#[test]
fn test_datetime_prefixed_filename_to_slug() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();

    // Create posts with datetime-prefixed filenames
    let post1 = "# Morning Post\nWritten in the morning";
    fs::write(
        input_dir
            .join("content")
            .join("2024-01-15-09-30-45-morning-update.md"),
        post1,
    )
    .unwrap();

    let post2 = "# Evening Post\nWritten in the evening";
    fs::write(
        input_dir
            .join("content")
            .join("2024-01-15-18-45-00-evening-summary.md"),
        post2,
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

    // Verify posts are generated without datetime prefix in filename
    assert!(output_dir.join("morning-update.html").exists());
    assert!(output_dir.join("evening-summary.html").exists());

    // Verify datetime-prefixed files are NOT generated
    assert!(!output_dir
        .join("2024-01-15-09-30-45-morning-update.html")
        .exists());
    assert!(!output_dir
        .join("2024-01-15-18-45-00-evening-summary.html")
        .exists());
}
