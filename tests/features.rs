use std::fs;
use std::process::Command;
use tempfile::TempDir;

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
    fs::write(
        input_dir.join("static").join("style.css"),
        "body { color: red; }",
    )
    .unwrap();
    fs::write(
        input_dir.join("static").join("script.js"),
        "console.log('test');",
    )
    .unwrap();

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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
    fs::write(
        input_dir.join("content").join("media").join("image.jpg"),
        "fake image data",
    )
    .unwrap();

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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
    fs::write(
        &config_path,
        "name: Site\nbuild_sitemap: true\nurl: https://example.com",
    )
    .unwrap();

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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify sitemap was generated
    assert!(output_dir.join("sitemap.xml").exists());

    // Verify sitemap content
    let sitemap_content = fs::read_to_string(output_dir.join("sitemap.xml")).unwrap();
    assert!(sitemap_content.contains("<urlset"));
    assert!(sitemap_content.contains("page1.html"));
    assert!(sitemap_content.contains("page2.html"));
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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify author pages were generated
    assert!(output_dir.join("author-alice.html").exists());
    assert!(output_dir.join("author-bob.html").exists());
    assert!(output_dir.join("authors.html").exists());
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
    fs::write(
        input_dir.join("README.md"),
        "# About\n\nThis is the about page.",
    )
    .unwrap();

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

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify mapped file exists at destination
    assert!(output_dir.join("about.html").exists());
}
