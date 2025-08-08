use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_marmite_binary_help() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("marmite"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_marmite_version() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("marmite"));
}

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
    fs::write(&config_path, "name: Test Site\ntagline: Test").unwrap();
    
    // Create a simple content file
    let content_path = input_dir.join("content").join("test.md");
    fs::write(&content_path, "# Test Page\n\nThis is a test.").unwrap();
    
    // Generate site using marmite binary
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
    
    // Verify output
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("test.html").exists());
}

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
    let post1 = r#"---
date: 2024-01-01
---
# First Post
Content 1"#;
    fs::write(input_dir.join("content").join("2024-01-01-post1.md"), post1).unwrap();
    
    let post2 = r#"---
date: 2024-01-02
---
# Second Post
Content 2"#;
    fs::write(input_dir.join("content").join("2024-01-02-post2.md"), post2).unwrap();
    
    let post3 = r#"---
date: 2024-01-03
---
# Third Post
Content 3"#;
    fs::write(input_dir.join("content").join("2024-01-03-post3.md"), post3).unwrap();
    
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
    let post1 = r#"---
date: 2024-01-01
tags: [rust, programming]
---
# Rust Post"#;
    fs::write(input_dir.join("content").join("rust-post.md"), post1).unwrap();
    
    let post2 = r#"---
date: 2024-01-02
tags: [rust, testing]
---
# Testing Post"#;
    fs::write(input_dir.join("content").join("test-post.md"), post2).unwrap();
    
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
    
    // Verify tag pages were generated
    assert!(output_dir.join("tag-rust.html").exists());
    assert!(output_dir.join("tag-programming.html").exists());
    assert!(output_dir.join("tag-testing.html").exists());
    assert!(output_dir.join("tags.html").exists());
}

#[test]
fn test_site_generation_with_static_files() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("static")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Site").unwrap();
    
    // Create static files
    fs::write(input_dir.join("static").join("style.css"), "body { color: red; }").unwrap();
    fs::write(input_dir.join("static").join("script.js"), "console.log('test');").unwrap();
    
    // Create a content file
    fs::write(input_dir.join("content").join("page.md"), "# Page").unwrap();
    
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
    
    // Verify static files were copied
    assert!(output_dir.join("static").join("style.css").exists());
    assert!(output_dir.join("static").join("script.js").exists());
}

#[test]
fn test_site_generation_with_media() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::create_dir_all(input_dir.join("content").join("media")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Site").unwrap();
    
    // Create media files
    fs::write(input_dir.join("content").join("media").join("image.jpg"), "fake image data").unwrap();
    
    // Create content referencing media
    let content = r#"# Post with Image
![Test Image](media/image.jpg)"#;
    fs::write(input_dir.join("content").join("post.md"), content).unwrap();
    
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
    
    // Verify media was copied
    assert!(output_dir.join("media").join("image.jpg").exists());
}

#[test]
fn test_site_generation_with_sitemap() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config with sitemap enabled
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Site\nbuild_sitemap: true\nurl: https://example.com").unwrap();
    
    // Create content
    fs::write(input_dir.join("content").join("page1.md"), "# Page 1").unwrap();
    fs::write(input_dir.join("content").join("page2.md"), "# Page 2").unwrap();
    
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
    
    // Verify sitemap was generated
    assert!(output_dir.join("sitemap.xml").exists());
    
    // Verify sitemap content
    let sitemap_content = fs::read_to_string(output_dir.join("sitemap.xml")).unwrap();
    assert!(sitemap_content.contains("<urlset"));
    assert!(sitemap_content.contains("page1.html"));
    assert!(sitemap_content.contains("page2.html"));
}

#[test]
fn test_site_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("new_site");
    
    // Initialize new site
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify structure was created
    assert!(site_dir.exists());
    assert!(site_dir.join("marmite.yaml").exists());
    assert!(site_dir.join("content").exists());
    assert!(site_dir.join("content").join("media").exists());
    assert!(site_dir.join("custom.css").exists());
    assert!(site_dir.join("custom.js").exists());
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
    let post = r#"---
date: 2024-01-01
---
# Published Post"#;
    fs::write(input_dir.join("content").join("post.md"), post).unwrap();
    
    // Create a draft post with draft- prefix
    let draft = r#"---
date: 2024-01-02
---
# Draft Post"#;
    fs::write(input_dir.join("content").join("draft-2024-01-02-draft.md"), draft).unwrap();
    
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
    
    // Verify published post exists
    assert!(output_dir.join("post.html").exists());
    
    // Verify draft post does NOT exist
    assert!(!output_dir.join("draft-2024-01-02-draft.html").exists());
}

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
fn test_series_feature() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Blog").unwrap();
    
    // Create posts in a series
    let part1 = r#"---
date: 2024-01-01
series: python-tutorial
---
# Python Tutorial Part 1"#;
    fs::write(input_dir.join("content").join("python-part1.md"), part1).unwrap();
    
    let part2 = r#"---
date: 2024-01-02
series: python-tutorial
---
# Python Tutorial Part 2"#;
    fs::write(input_dir.join("content").join("python-part2.md"), part2).unwrap();
    
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
    
    // Verify series page was generated
    assert!(output_dir.join("series-python-tutorial.html").exists());
    assert!(output_dir.join("series.html").exists());
}

#[test]
fn test_fragments_feature() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Site").unwrap();
    
    // Create a fragment file (starts with underscore)
    let fragment = "## This is a fragment\n\nIt can be included in templates.";
    fs::write(input_dir.join("content").join("_hero.md"), fragment).unwrap();
    
    // Create a regular page
    fs::write(input_dir.join("content").join("page.md"), "# Regular Page").unwrap();
    
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
    
    // Verify fragment is NOT generated as a page
    assert!(!output_dir.join("_hero.html").exists());
    assert!(!output_dir.join("hero.html").exists());
    
    // Verify regular page exists
    assert!(output_dir.join("page.html").exists());
}

#[test]
fn test_authors_feature() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config with authors
    let config = r#"name: Blog
authors:
  alice:
    name: Alice Smith
    bio: Developer
    avatar: alice.jpg
  bob:
    name: Bob Jones
    bio: Designer
    avatar: bob.jpg
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();
    
    // Create posts with different authors
    let post1 = r#"---
date: 2024-01-01
author: alice
---
# Alice's Post"#;
    fs::write(input_dir.join("content").join("alice-post.md"), post1).unwrap();
    
    let post2 = r#"---
date: 2024-01-02
authors: [alice, bob]
---
# Collaborative Post"#;
    fs::write(input_dir.join("content").join("collab-post.md"), post2).unwrap();
    
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
    
    // Verify author pages were generated
    assert!(output_dir.join("author-alice.html").exists());
    assert!(output_dir.join("author-bob.html").exists());
    assert!(output_dir.join("authors.html").exists());
}

#[test]
fn test_show_urls_command() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config
    let config_path = input_dir.join("marmite.yaml");
    fs::write(&config_path, "name: Test\nurl: https://example.com").unwrap();
    
    // Create a few content files
    fs::write(input_dir.join("content").join("page1.md"), "# Page 1").unwrap();
    fs::write(input_dir.join("content").join("page2.md"), "# Page 2").unwrap();
    
    // Run show-urls command
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            "--show-urls",
        ])
        .output()
        .expect("Failed to execute marmite");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://example.com/page1.html"));
    assert!(stdout.contains("https://example.com/page2.html"));
    assert!(stdout.contains("https://example.com/index.html"));
}

#[test]
fn test_file_mapping_feature() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();
    
    // Create config with file mapping
    let config = r#"name: Site
file_mapping:
  - source: README.md
    dest: about.html
"#;
    fs::write(input_dir.join("marmite.yaml"), config).unwrap();
    
    // Create README.md
    fs::write(input_dir.join("README.md"), "# About\n\nThis is the about page.").unwrap();
    
    // Create a regular content file
    fs::write(input_dir.join("content").join("page.md"), "# Page").unwrap();
    
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
    
    // Verify mapped file exists at destination
    assert!(output_dir.join("about.html").exists());
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
    fs::write(input_dir.join("content").join("2024-01-15-my-first-post.md"), post1).unwrap();
    
    let post2 = "# Second Post\nMore content";
    fs::write(input_dir.join("content").join("2024-02-20-another-post.md"), post2).unwrap();
    
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
    fs::write(input_dir.join("content").join("2024-01-15-09-30-45-morning-update.md"), post1).unwrap();
    
    let post2 = "# Evening Post\nWritten in the evening";  
    fs::write(input_dir.join("content").join("2024-01-15-18-45-00-evening-summary.md"), post2).unwrap();
    
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
    
    // Verify posts are generated without datetime prefix in filename
    assert!(output_dir.join("morning-update.html").exists());
    assert!(output_dir.join("evening-summary.html").exists());
    
    // Verify datetime-prefixed files are NOT generated
    assert!(!output_dir.join("2024-01-15-09-30-45-morning-update.html").exists());
    assert!(!output_dir.join("2024-01-15-18-45-00-evening-summary.html").exists());
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

#[test]
fn test_theme_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("test_site");
    let theme_name = "my_custom_theme";
    
    // Step 1: Initialize a new site
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to initialize site");
    
    assert!(output.status.success(), "Site init failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(site_dir.join("marmite.yaml").exists());
    assert!(site_dir.join("content").exists());
    
    // Step 2: Start a new theme
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--start-theme",
            theme_name,
        ])
        .output()
        .expect("Failed to start theme");
    
    assert!(output.status.success(), "Start theme failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify theme structure was created
    let theme_dir = site_dir.join(theme_name);
    assert!(theme_dir.exists(), "Theme directory not created");
    assert!(theme_dir.join("templates").exists(), "Theme templates directory not created");
    assert!(theme_dir.join("templates").join("base.html").exists(), "base.html not created");
    assert!(theme_dir.join("templates").join("content.html").exists(), "content.html not created");
    assert!(theme_dir.join("templates").join("list.html").exists(), "list.html not created");
    assert!(theme_dir.join("static").exists(), "Theme static directory not created");
    assert!(theme_dir.join("static").join("custom.css").exists(), "custom.css not created");
    assert!(theme_dir.join("theme.json").exists(), "theme.json not created");
    
    // Step 3: Customize the theme - modify base.html template
    let base_template_path = theme_dir.join("templates").join("base.html");
    let base_content = fs::read_to_string(&base_template_path).unwrap();
    
    // Add a custom marker to the template that we can verify later
    let custom_base = base_content.replace(
        "<body>",
        "<body>\n    <!-- CUSTOM_THEME_MARKER -->"
    );
    fs::write(&base_template_path, custom_base).unwrap();
    
    // Step 4: Add custom CSS to the site root (not theme)
    // This gets copied to output/static/custom.css
    let custom_css = r#"
/* Custom theme styles */
.custom-theme-class {
    color: #ff6600;
    font-size: 18px;
}

body {
    background-color: #f5f5f5;
}
"#;
    fs::write(site_dir.join("custom.css"), custom_css).unwrap();
    
    // Also add a theme-specific CSS file in the theme's static directory
    let theme_css = r#"
/* Theme specific styles */
.theme-specific {
    color: #0066cc;
}
"#;
    fs::write(theme_dir.join("static").join("theme.css"), theme_css).unwrap();
    
    // Step 5: Create some content to test with
    let post_content = r#"---
date: 2024-01-15
title: Test Post with Theme
tags: [test, theme]
---

# Test Post with Theme

This is a test post to verify the custom theme is working correctly.

## Features

- Custom templates
- Custom CSS
- Theme configuration
"#;
    fs::write(site_dir.join("content").join("test-post.md"), post_content).unwrap();
    
    // Create a page without date
    let page_content = r#"---
title: About Page
---

# About

This is an about page using the custom theme.
"#;
    fs::write(site_dir.join("content").join("about.md"), page_content).unwrap();
    
    // Step 6: Set the theme in configuration
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--set-theme",
            theme_name,
        ])
        .output()
        .expect("Failed to set theme");
    
    assert!(output.status.success(), "Set theme failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify theme was added to config
    let config_content = fs::read_to_string(site_dir.join("marmite.yaml")).unwrap();
    assert!(config_content.contains(&format!("theme: {}", theme_name)), "Theme not set in config");
    
    // Step 7: Generate the site with the custom theme
    let output_dir = site_dir.join("site");
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to generate site");
    
    assert!(output.status.success(), "Site generation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Step 8: Verify the generated site uses the custom theme
    
    // Check that pages were generated (slug is based on title)
    assert!(output_dir.join("test-post-with-theme.html").exists(), "Post not generated");
    assert!(output_dir.join("about-page.html").exists(), "Page not generated");
    assert!(output_dir.join("index.html").exists(), "Index not generated");
    
    // Check that custom CSS was copied from site root to static
    assert!(output_dir.join("static").join("custom.css").exists(), "Custom CSS not copied");
    
    // Verify custom CSS content
    let output_css = fs::read_to_string(output_dir.join("static").join("custom.css")).unwrap();
    assert!(output_css.contains(".custom-theme-class"), "Custom CSS class not found");
    assert!(output_css.contains("background-color: #f5f5f5"), "Custom CSS rule not found");
    
    // Check that theme static files were copied
    assert!(output_dir.join("static").join("theme.css").exists(), "Theme CSS not copied");
    let theme_css = fs::read_to_string(output_dir.join("static").join("theme.css")).unwrap();
    assert!(theme_css.contains(".theme-specific"), "Theme CSS content not found");
    
    // Verify the custom template marker is in generated HTML
    let post_html = fs::read_to_string(output_dir.join("test-post-with-theme.html")).unwrap();
    assert!(post_html.contains("<!-- CUSTOM_THEME_MARKER -->"), "Custom template marker not found in post");
    
    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index_html.contains("<!-- CUSTOM_THEME_MARKER -->"), "Custom template marker not found in index");
    
    // Verify that the post content is present
    assert!(post_html.contains("Test Post with Theme"), "Post title not found");
    assert!(post_html.contains("This is a test post"), "Post content not found");
    
    // Step 9: Test generating with theme specified via CLI (override config)
    let output_dir2 = site_dir.join("site2");
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            output_dir2.to_str().unwrap(),
            "--theme",
            theme_name,
        ])
        .output()
        .expect("Failed to generate site with --theme flag");
    
    assert!(output.status.success(), "Site generation with --theme failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify the theme was applied via CLI flag
    assert!(output_dir2.join("test-post-with-theme.html").exists(), "Post not generated with --theme flag");
    let post_html2 = fs::read_to_string(output_dir2.join("test-post-with-theme.html")).unwrap();
    assert!(post_html2.contains("<!-- CUSTOM_THEME_MARKER -->"), "Custom template not applied with --theme flag");
}