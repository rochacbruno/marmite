use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_marmite_binary_help() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
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
        .args(["run", "--quiet", "--", "--version"])
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

    // Verify output
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("test.html").exists());
}

#[test]
fn test_site_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("new_site");

    // Initialize new site
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify structure was created
    assert!(site_dir.exists());
    assert!(site_dir.join("marmite.yaml").exists());
    assert!(site_dir.join("content").exists());
    assert!(site_dir.join("content").join("media").exists());
    assert!(site_dir.join("custom.css").exists());
    assert!(site_dir.join("custom.js").exists());
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
        .args([
            "run",
            "--quiet",
            "--",
            input_dir.to_str().unwrap(),
            "--show-urls",
        ])
        .output()
        .expect("Failed to execute marmite");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://example.com/page1.html"));
    assert!(stdout.contains("https://example.com/page2.html"));
    assert!(stdout.contains("https://example.com/index.html"));
}

#[test]
fn test_standard_site_atproto_generation() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");

    // Create input directory structure
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(input_dir.join("content")).unwrap();

    // Create config file with atproto configured
    let config_path = input_dir.join("marmite.yaml");
    fs::write(
        &config_path,
        r#"name: Test ATProto Blog
url: https://myblog.com
atproto:
  handle: test.bsky.social
  publication_uri: at://did:plc:123/site.standard.publication/456
"#,
    )
    .unwrap();

    // Create a post
    let content_path = input_dir.join("content").join("first-post.md");
    fs::write(&content_path, "# First Post\n\nHello standard.site!").unwrap();

    // Create a mock state file
    let state_path = input_dir.join(".marmite-atproto-state.json");
    fs::write(
        &state_path,
        r#"{"posts":{"first-post":{"content_hash":"abc","at_uri":"at://did:plc:123/site.standard.document/first-post","last_published":"2026-06-17T17:08:42Z"}}}"#
    ).unwrap();

    // Generate site using marmite binary
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

    // 1. Verify verification file was generated under .well-known
    let wk_file = output_dir
        .join(".well-known")
        .join("site.standard.publication");
    assert!(wk_file.exists());
    let wk_content = fs::read_to_string(&wk_file).unwrap();
    assert_eq!(
        wk_content.trim(),
        "at://did:plc:123/site.standard.publication/456"
    );

    // 2. Verify base layout has the link rel="site.standard.publication" tag
    let index_file = output_dir.join("index.html");
    assert!(index_file.exists());
    let index_content = fs::read_to_string(&index_file).unwrap();
    assert!(index_content.contains(r#"<link rel="site.standard.publication" href="at://did:plc:123/site.standard.publication/456">"#));

    // 3. Verify post has the link rel="site.standard.document" tag
    let post_file = output_dir.join("first-post.html");
    assert!(post_file.exists());
    let post_content = fs::read_to_string(&post_file).unwrap();
    assert!(post_content.contains(r#"<link rel="site.standard.document" href="at://did:plc:123/site.standard.document/first-post">"#));
}
