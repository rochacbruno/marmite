use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn create_workspace(temp_dir: &std::path::Path) -> std::path::PathBuf {
    let ws_dir = temp_dir.join("workspace");
    fs::create_dir_all(ws_dir.join("blog/content")).unwrap();
    fs::create_dir_all(ws_dir.join("photos/content")).unwrap();

    fs::write(
        ws_dir.join("marmite-workspace.yaml"),
        r#"
sites:
  - name: blog
  - name: photos
default_site: blog
redirect: false
defaults:
  language: en
"#,
    )
    .unwrap();

    fs::write(
        ws_dir.join("blog/marmite.yaml"),
        "name: Test Blog\ntagline: Blog\n",
    )
    .unwrap();

    fs::write(
        ws_dir.join("photos/marmite.yaml"),
        "name: Test Photos\ntagline: Photos\n",
    )
    .unwrap();

    fs::write(
        ws_dir.join("blog/content/2024-01-01-hello.md"),
        "---\ntitle: Hello\ntags: [intro]\n---\nHello from [photos](photos::gallery.html).\n",
    )
    .unwrap();

    fs::write(
        ws_dir.join("blog/content/about.md"),
        "---\ntitle: About\n---\nAbout page.\n",
    )
    .unwrap();

    fs::write(
        ws_dir.join("photos/content/2024-01-01-sunset.md"),
        "---\ntitle: Sunset\ntags: [nature]\n---\nSunset photo. See [blog](blog::hello.html).\n",
    )
    .unwrap();

    fs::write(
        ws_dir.join("photos/content/gallery.md"),
        "---\ntitle: Gallery\n---\nGallery page.\n",
    )
    .unwrap();

    ws_dir
}

fn run_marmite(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to execute marmite")
}

#[test]
fn test_workspace_build_default_site_at_root() {
    let temp = TempDir::new().unwrap();
    let ws_dir = create_workspace(temp.path());
    let output_dir = temp.path().join("output");

    let result = run_marmite(&[ws_dir.to_str().unwrap(), output_dir.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Default site (blog) renders at root
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("hello.html").exists());
    assert!(output_dir.join("about.html").exists());

    // Photos renders in subdirectory
    assert!(output_dir.join("photos/index.html").exists());
    assert!(output_dir.join("photos/sunset.html").exists());
    assert!(output_dir.join("photos/gallery.html").exists());

    // sites.json is generated
    assert!(output_dir.join("sites.json").exists());
    let sites_json = fs::read_to_string(output_dir.join("sites.json")).unwrap();
    assert!(sites_json.contains("blog"));
    assert!(sites_json.contains("photos"));
}

#[test]
fn test_workspace_cross_site_links() {
    let temp = TempDir::new().unwrap();
    let ws_dir = create_workspace(temp.path());
    let output_dir = temp.path().join("output");

    let result = run_marmite(&[ws_dir.to_str().unwrap(), output_dir.to_str().unwrap()]);
    assert!(result.status.success());

    // Blog post should link to /photos/gallery.html
    let blog_post = fs::read_to_string(output_dir.join("hello.html")).unwrap();
    assert!(
        blog_post.contains("href=\"/photos/gallery.html\""),
        "Blog post should have cross-site link to photos. Content: {blog_post}"
    );

    // Photos post should link to /blog/hello.html
    let photos_post = fs::read_to_string(output_dir.join("photos/sunset.html")).unwrap();
    assert!(
        photos_post.contains("href=\"/blog/hello.html\""),
        "Photos post should have cross-site link to blog. Content: {photos_post}"
    );
}

#[test]
fn test_workspace_redirect_mode() {
    let temp = TempDir::new().unwrap();
    let ws_dir = create_workspace(temp.path());
    let output_dir = temp.path().join("output");

    // Override config with redirect mode
    fs::write(
        ws_dir.join("marmite-workspace.yaml"),
        r#"
sites:
  - name: blog
  - name: photos
default_site: blog
redirect: true
"#,
    )
    .unwrap();

    let result = run_marmite(&[ws_dir.to_str().unwrap(), output_dir.to_str().unwrap()]);
    assert!(result.status.success());

    // Root index.html should be a redirect
    let root_index = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(root_index.contains("meta http-equiv=\"refresh\""));
    assert!(root_index.contains("/blog/"));

    // Blog should be in subdirectory
    assert!(output_dir.join("blog/index.html").exists());
    assert!(output_dir.join("blog/hello.html").exists());

    // Photos should be in subdirectory
    assert!(output_dir.join("photos/index.html").exists());
}

#[test]
fn test_workspace_show_urls() {
    let temp = TempDir::new().unwrap();
    let ws_dir = create_workspace(temp.path());

    let result = run_marmite(&[ws_dir.to_str().unwrap(), "--show-urls"]);
    assert!(result.status.success());

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("\"blog\""));
    assert!(stdout.contains("\"photos\""));
    assert!(stdout.contains("/blog/"));
    assert!(stdout.contains("/photos/"));
}

#[test]
fn test_workspace_unsupported_commands() {
    let temp = TempDir::new().unwrap();
    let ws_dir = create_workspace(temp.path());

    let result = run_marmite(&[ws_dir.to_str().unwrap(), "--init-templates"]);
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("not supported in workspace mode"));
}

#[test]
fn test_workspace_new_requires_site_flag() {
    let temp = TempDir::new().unwrap();
    let ws_dir = create_workspace(temp.path());

    let result = run_marmite(&[ws_dir.to_str().unwrap(), "--new", "My Post"]);
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("--site"));
}

#[test]
fn test_single_site_unaffected_by_workspace() {
    let temp = TempDir::new().unwrap();
    let input_dir = temp.path().join("site");
    let output_dir = temp.path().join("output");

    fs::create_dir_all(input_dir.join("content")).unwrap();
    fs::write(
        input_dir.join("marmite.yaml"),
        "name: Normal Site\ntagline: Test\n",
    )
    .unwrap();
    fs::write(
        input_dir.join("content/test.md"),
        "---\ntitle: Test\n---\nContent\n",
    )
    .unwrap();

    let result = run_marmite(&[input_dir.to_str().unwrap(), output_dir.to_str().unwrap()]);
    assert!(result.status.success());
    assert!(output_dir.join("test.html").exists());
    assert!(!output_dir.join("sites.json").exists());
}

#[test]
fn test_workspace_config_inheritance() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    fs::create_dir_all(ws_dir.join("site1/content")).unwrap();

    fs::write(
        ws_dir.join("marmite-workspace.yaml"),
        r#"
sites:
  - name: site1
defaults:
  name: Workspace Default Name
  tagline: Workspace Tagline
"#,
    )
    .unwrap();

    // Site1 overrides name but not tagline
    fs::write(ws_dir.join("site1/marmite.yaml"), "name: Site One\n").unwrap();
    fs::write(
        ws_dir.join("site1/content/page.md"),
        "---\ntitle: Page\n---\nContent\n",
    )
    .unwrap();

    let output_dir = temp.path().join("output");
    let result = run_marmite(&[ws_dir.to_str().unwrap(), output_dir.to_str().unwrap()]);
    assert!(result.status.success());

    // Check marmite.json for the merged config
    let build_info = fs::read_to_string(output_dir.join("marmite.json")).unwrap();
    let info: serde_json::Value = serde_json::from_str(&build_info).unwrap();
    assert_eq!(info["config"]["name"], "Site One");
    assert_eq!(info["config"]["tagline"], "Workspace Tagline");
}

#[test]
fn test_workspace_cross_site_shortcodes() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    fs::create_dir_all(ws_dir.join("blog/content")).unwrap();
    fs::create_dir_all(ws_dir.join("photos/content")).unwrap();

    fs::write(
        ws_dir.join("marmite-workspace.yaml"),
        "sites:\n  - name: blog\n  - name: photos\ndefault_site: blog\n",
    )
    .unwrap();
    fs::write(ws_dir.join("blog/marmite.yaml"), "name: Blog\n").unwrap();
    fs::write(ws_dir.join("photos/marmite.yaml"), "name: Photos\n").unwrap();

    fs::write(
        ws_dir.join("blog/content/2024-01-01-post.md"),
        "---\ntitle: Blog Post\ntags: [intro]\n---\n\n<!-- .posts site=\"photos\" -->\n\n<!-- .tags site=\"photos\" -->\n",
    )
    .unwrap();

    fs::write(
        ws_dir.join("photos/content/2024-02-01-sunset.md"),
        "---\ntitle: Sunset\ntags: [nature]\n---\nA photo.\n",
    )
    .unwrap();
    fs::write(
        ws_dir.join("photos/content/2024-03-01-mountain.md"),
        "---\ntitle: Mountain\ntags: [nature, landscape]\n---\nAnother photo.\n",
    )
    .unwrap();

    let output_dir = temp.path().join("output");
    let result = run_marmite(&[ws_dir.to_str().unwrap(), output_dir.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let blog_post = fs::read_to_string(output_dir.join("blog-post.html")).unwrap();

    // Cross-site posts shortcode should render photos site posts with correct URLs
    assert!(
        blog_post.contains("/photos/mountain.html"),
        "Should contain cross-site link to photos/mountain.html. Content:\n{blog_post}"
    );
    assert!(
        blog_post.contains("/photos/sunset.html"),
        "Should contain cross-site link to photos/sunset.html"
    );

    // Cross-site tags shortcode should render photos site tags
    assert!(
        blog_post.contains("nature"),
        "Should contain the 'nature' tag from photos site"
    );
}
