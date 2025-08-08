use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_streams_feature() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();
    
    // Create posts in different streams using frontmatter
    let tutorial_post = r#"---
date: 2024-01-01
stream: tutorial
---
# Tutorial Post"#;
    fs::write(input_dir.join("content").join("getting-started.md"), tutorial_post).unwrap();
    
    let news_post = r#"---
date: 2024-01-02
stream: news
---
# News Post"#;
    fs::write(input_dir.join("content").join("update.md"), news_post).unwrap();
    
    // Generate site
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify stream pages were generated
    assert!(output_dir.join("tutorial.html").exists());
    assert!(output_dir.join("news.html").exists());
    assert!(output_dir.join("streams.html").exists());
}

#[test]
fn test_stream_date_prefixed_filename() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();
    
    // Create posts with stream-date-prefixed filenames
    // Format: stream-YYYY-MM-DD-slug.md -> stream-slug.html
    let news_post = "# Breaking News\nImportant news update";
    fs::write(input_dir.join("content").join("news-2024-01-15-breaking-story.md"), news_post).unwrap();
    
    let tutorial_post = "# Tutorial\nLearn something new";
    fs::write(input_dir.join("content").join("tutorial-2024-02-20-getting-started.md"), tutorial_post).unwrap();
    
    // Generate site
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify posts are generated with stream prefix but without date
    assert!(output_dir.join("news-breaking-story.html").exists());
    assert!(output_dir.join("tutorial-getting-started.html").exists());
    
    // Verify stream pages are generated
    assert!(output_dir.join("news.html").exists());
    assert!(output_dir.join("tutorial.html").exists());
    assert!(output_dir.join("streams.html").exists());
}

#[test]
fn test_stream_s_prefixed_filename() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();
    
    // Create posts with stream-S-prefixed filenames (stream without date in filename, but needs date in frontmatter)
    // Format: stream-S-slug.md -> stream-slug.html
    let guide_post = r#"---
date: 2024-03-01
---
# Complete Guide
A comprehensive guide"#;
    fs::write(input_dir.join("content").join("guide-S-complete-tutorial.md"), guide_post).unwrap();
    
    let howto_post = r#"---
date: 2024-03-02
---
# How To
Step by step instructions"#;
    fs::write(input_dir.join("content").join("howto-S-install-rust.md"), howto_post).unwrap();
    
    // Generate site
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify posts are generated with stream prefix
    assert!(output_dir.join("guide-complete-tutorial.html").exists());
    assert!(output_dir.join("howto-install-rust.html").exists());
    
    // Verify stream pages are generated
    assert!(output_dir.join("guide.html").exists());
    assert!(output_dir.join("howto.html").exists());
    assert!(output_dir.join("streams.html").exists());
}