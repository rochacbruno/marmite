use serde_json::Value;
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
    assert!(
        site_dir.join("content").join("pages").exists(),
        "content/pages/ should be created"
    );
    assert!(
        site_dir.join("content").join("posts").exists(),
        "content/posts/ should be created"
    );
    assert!(
        site_dir
            .join("content")
            .join("pages")
            .join("about.md")
            .exists(),
        "about.md should be in content/pages/"
    );
    assert!(
        site_dir
            .join("content")
            .join("posts")
            .join("welcome.md")
            .exists(),
        "welcome.md should be in content/posts/"
    );
    assert!(site_dir.join("custom.css").exists());
    assert!(site_dir.join("custom.js").exists());
}

#[test]
fn test_new_with_directory_flag() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("dir_site");

    // Initialize site first
    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to init site");

    // Create a post in a specific directory
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Test Post",
            "-d",
            "posts",
        ])
        .output()
        .expect("Failed to create post");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("posts"),
        "Output file path should contain 'posts' directory: {file_path}"
    );
    assert_eq!(json["title"].as_str().unwrap(), "Test Post");
    assert_eq!(json["slug"].as_str().unwrap(), "test-post");
    assert!(
        json["date"].as_str().is_some(),
        "Posts should have a date field"
    );

    // Create a page in the pages directory
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Contact",
            "-p",
            "-d",
            "pages",
        ])
        .output()
        .expect("Failed to create page");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        site_dir
            .join("content")
            .join("pages")
            .join("contact.md")
            .exists(),
        "Page should be created in content/pages/"
    );

    // Create in a new directory (should auto-create it)
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Tutorial One",
            "-d",
            "tutorials",
        ])
        .output()
        .expect("Failed to create in new dir");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        site_dir.join("content").join("tutorials").exists(),
        "Directory should be auto-created"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("tutorials"),
        "Output file path should contain 'tutorials' directory: {file_path}"
    );
}

#[test]
fn test_new_json_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("json_site");

    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to init site");

    // Create a post and verify JSON output
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "My Post",
            "-t",
            "rust,web",
        ])
        .output()
        .expect("Failed to create post");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    assert_eq!(json["title"].as_str().unwrap(), "My Post");
    assert_eq!(json["slug"].as_str().unwrap(), "my-post");
    let file_path = json["file"].as_str().unwrap();
    assert!(file_path.ends_with(".md"));
    assert!(
        file_path.contains("posts"),
        "Post should auto-detect posts/ directory: {file_path}"
    );
    assert!(json["date"].as_str().is_some());
    assert_eq!(json["tags"].as_str().unwrap(), "rust,web");

    // Create a page and verify no date field
    // Use "Contact" instead of "About" because --init-site already creates pages/about.md
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Contact",
            "-p",
        ])
        .output()
        .expect("Failed to create page");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    assert_eq!(json["title"].as_str().unwrap(), "Contact");
    assert!(json["date"].is_null(), "Pages should not have a date field");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("pages"),
        "Page should auto-detect pages/ directory: {file_path}"
    );
}

#[test]
fn test_new_with_lang_standalone() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("lang_site");

    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to init site");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Ola Mundo",
            "--lang",
            "pt",
        ])
        .output()
        .expect("Failed to create post with lang");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    assert_eq!(json["language"].as_str().unwrap(), "pt");

    let file_path = json["file"].as_str().unwrap();
    let content = fs::read_to_string(file_path).unwrap();
    assert!(
        content.contains("language: pt"),
        "Frontmatter should contain language field"
    );
}

#[test]
fn test_new_with_invalid_lang() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("invalid_lang_site");

    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to init site");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Test",
            "--lang",
            "xyz",
        ])
        .output()
        .expect("Failed to run command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid language code"),
        "Should report invalid language code: {stderr}"
    );
}

#[test]
fn test_new_translation_subfolder() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("trans_sub_site");

    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to init site");

    // Create original post in a subfolder
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Hello World",
            "-p",
            "-d",
            "hello-world",
        ])
        .output()
        .expect("Failed to create original post");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Create translation - should auto-place in the same subfolder
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Ola Mundo",
            "--lang",
            "pt",
            "--translates",
            "hello-world",
        ])
        .output()
        .expect("Failed to create translation");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    assert_eq!(json["language"].as_str().unwrap(), "pt");
    assert_eq!(json["translates"].as_str().unwrap(), "hello-world");

    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("hello-world/pt-ola-mundo.md"),
        "Translation should be in the original's subfolder with lang prefix: {file_path}"
    );
    assert!(
        std::path::Path::new(file_path).exists(),
        "Translation file should exist"
    );

    // Verify frontmatter does NOT include translates (subfolder grouping handles it)
    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains("language: pt"));
    assert!(
        !content.contains("translates:"),
        "Subfolder translations should not have translates frontmatter"
    );
}

#[test]
fn test_new_translation_root_level() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("trans_root_site");

    // Manually create a content-folder project without posts/pages subdirs
    // so the page stays at the content root level
    let content_dir = site_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        site_dir.join("marmite.yaml"),
        "name: Test Site\ntagline: Test",
    )
    .unwrap();

    // Create original post at root level (page so filename is predictable)
    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "About Page",
            "-p",
        ])
        .output()
        .expect("Failed to create original page");

    // Create translation
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Pagina Sobre",
            "-p",
            "--lang",
            "pt",
            "--translates",
            "about-page",
        ])
        .output()
        .expect("Failed to create translation");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");

    let file_path = json["file"].as_str().unwrap();
    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains("language: pt"));
    assert!(
        content.contains("translates: about-page"),
        "Root-level translations should have translates frontmatter: {content}"
    );
}

#[test]
fn test_new_translation_nonexistent_slug() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("trans_noexist_site");

    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--init-site",
        ])
        .output()
        .expect("Failed to init site");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "Translation",
            "--lang",
            "pt",
            "--translates",
            "nonexistent-slug",
        ])
        .output()
        .expect("Failed to run command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cannot find content with slug"),
        "Should report missing original content: {stderr}"
    );
}

#[test]
fn test_new_auto_detects_posts_directory() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("auto_posts_site");
    let content_dir = site_dir.join("content");

    fs::create_dir_all(content_dir.join("posts")).unwrap();
    fs::write(
        site_dir.join("marmite.yaml"),
        "name: Test Site\ntagline: Test",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "My Post",
        ])
        .output()
        .expect("Failed to create post");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("content/posts/"),
        "Post should auto-detect posts/ directory: {file_path}"
    );
    assert!(
        std::path::Path::new(file_path).exists(),
        "File should exist at the auto-detected path"
    );
}

#[test]
fn test_new_auto_detects_pages_directory() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("auto_pages_site");
    let content_dir = site_dir.join("content");

    fs::create_dir_all(content_dir.join("pages")).unwrap();
    fs::write(
        site_dir.join("marmite.yaml"),
        "name: Test Site\ntagline: Test",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "About",
            "-p",
        ])
        .output()
        .expect("Failed to create page");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("content/pages/"),
        "Page should auto-detect pages/ directory: {file_path}"
    );
    assert!(
        std::path::Path::new(file_path).exists(),
        "File should exist at the auto-detected path"
    );
}

#[test]
fn test_new_no_auto_detect_without_subdirs() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("no_subdirs_site");
    let content_dir = site_dir.join("content");

    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        site_dir.join("marmite.yaml"),
        "name: Test Site\ntagline: Test",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "My Post",
        ])
        .output()
        .expect("Failed to create post");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        !file_path.contains("posts"),
        "Without posts/ dir, file should stay in content root: {file_path}"
    );
    assert!(
        file_path.contains("content/"),
        "File should be in the content directory: {file_path}"
    );
}

#[test]
fn test_new_explicit_d_overrides_auto_detect() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("override_site");
    let content_dir = site_dir.join("content");

    fs::create_dir_all(content_dir.join("posts")).unwrap();
    fs::write(
        site_dir.join("marmite.yaml"),
        "name: Test Site\ntagline: Test",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "My Post",
            "-d",
            "other",
        ])
        .output()
        .expect("Failed to create post");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        file_path.contains("content/other/"),
        "Explicit -d should override auto-detection: {file_path}"
    );
    assert!(
        !file_path.contains("posts"),
        "Should not use posts/ when -d is explicit: {file_path}"
    );
}

#[test]
fn test_new_flat_project_unchanged() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("flat_site");

    fs::create_dir_all(&site_dir).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            "--new",
            "My Post",
        ])
        .output()
        .expect("Failed to create post");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).expect("Output should be valid JSON");
    let file_path = json["file"].as_str().unwrap();
    assert!(
        !file_path.contains("content"),
        "Flat project should not use content/ directory: {file_path}"
    );
    assert!(
        !file_path.contains("posts"),
        "Flat project should not use posts/ directory: {file_path}"
    );
    assert!(
        std::path::Path::new(file_path).exists(),
        "File should exist in the flat project root"
    );
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
