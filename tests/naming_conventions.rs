use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn build_site(input_dir: &std::path::Path) -> (std::path::PathBuf, std::process::Output) {
    let output_dir = input_dir.parent().unwrap().join("output");
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
    (output_dir, output)
}

#[test]
fn test_mixed_naming_conventions_all_at_once() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let content_dir = input_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(
        input_dir.join("marmite.yaml"),
        "name: Test Site\nlanguage: en\n",
    )
    .unwrap();

    // Simple filename - page (no date)
    fs::write(content_dir.join("about.md"), "# About\n\nThis is a page.\n").unwrap();

    // Date prefix YYYY-MM-DD
    fs::write(
        content_dir.join("2024-01-15-my-post.md"),
        "# My Post\n\nDated post.\n",
    )
    .unwrap();

    // Datetime prefix YYYY-MM-DD-HH-MM
    fs::write(
        content_dir.join("2024-01-15-09-30-morning-update.md"),
        "# Morning Update\n\nDatetime post.\n",
    )
    .unwrap();

    // Full datetime prefix YYYY-MM-DD-HH-MM-SS
    fs::write(
        content_dir.join("2024-01-15-09-30-45-evening-update.md"),
        "# Evening Update\n\nFull datetime post.\n",
    )
    .unwrap();

    // Stream+date prefix
    fs::write(
        content_dir.join("news-2024-02-01-breaking.md"),
        "# Breaking News\n\nStream from filename.\n",
    )
    .unwrap();

    // Stream+S prefix (needs date in frontmatter to be a post)
    fs::write(
        content_dir.join("guide-S-complete-tutorial.md"),
        "---\ndate: 2024-03-01\n---\n# Complete Tutorial\n\nStream S pattern.\n",
    )
    .unwrap();

    // JSON frontmatter
    fs::write(
        content_dir.join("2024-04-01-json-test.md"),
        "{\"title\": \"JSON Format\"}\n\n# JSON Format\n\nJSON frontmatter works.\n",
    )
    .unwrap();

    // TOML frontmatter
    fs::write(
        content_dir.join("toml-test.md"),
        "+++\ntitle = \"TOML Format\"\ndate = 2024-04-02\n+++\n\nTOML frontmatter works.\n",
    )
    .unwrap();

    // No frontmatter at all
    fs::write(
        content_dir.join("no-frontmatter.md"),
        "# No Frontmatter\n\nJust markdown.\n",
    )
    .unwrap();

    // Fragment - should NOT produce HTML output
    fs::write(content_dir.join("_hero.md"), "# Hero Section\n").unwrap();

    // Date in frontmatter overrides filename date
    fs::write(
        content_dir.join("2024-01-01-override.md"),
        "---\ndate: 2025-06-01\ntitle: Overridden Date\n---\n# Overridden\n\nFrontmatter date wins.\n",
    )
    .unwrap();

    // Subfolder with frontmatter.yaml
    let python_dir = content_dir.join("python");
    fs::create_dir_all(&python_dir).unwrap();
    fs::write(
        python_dir.join("frontmatter.yaml"),
        "date: 2026-01-01\nstream: python\ntags:\n  - python\n",
    )
    .unwrap();
    fs::write(
        python_dir.join("basics.md"),
        "---\ntitle: Python Basics\n---\n# Python Basics\n\nLearn Python.\n",
    )
    .unwrap();

    // Subfolder translation group (no frontmatter.yaml)
    let hello_dir = content_dir.join("hello");
    fs::create_dir_all(&hello_dir).unwrap();
    fs::write(
        hello_dir.join("hello.md"),
        "---\ntitle: Hello\ndate: 2026-01-01\n---\n# Hello\n\nHello world.\n",
    )
    .unwrap();
    fs::write(
        hello_dir.join("pt-ola.md"),
        "---\ntitle: Ola\ndate: 2026-01-01\n---\n# Ola\n\nOla mundo.\n",
    )
    .unwrap();

    // Dated subfolder
    let dated_dir = content_dir.join("2024-06-15-dated-folder");
    fs::create_dir_all(&dated_dir).unwrap();
    fs::write(
        dated_dir.join("dated-post.md"),
        "---\ntitle: Dated Folder Post\n---\n# Dated Folder Post\n\nDate from folder name.\n",
    )
    .unwrap();

    let (output_dir, output) = build_site(&input_dir);

    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Simple page
    assert!(
        output_dir.join("about.html").exists(),
        "Simple page should render"
    );

    // Date prefix - slug strips date
    assert!(
        output_dir.join("my-post.html").exists(),
        "Date-prefixed post should strip date from slug"
    );
    assert!(
        !output_dir.join("2024-01-15-my-post.html").exists(),
        "Date should not appear in output filename"
    );

    // Datetime prefix - slug strips datetime
    assert!(
        output_dir.join("morning-update.html").exists(),
        "Datetime-prefixed post should strip datetime from slug"
    );

    // Full datetime prefix
    assert!(
        output_dir.join("evening-update.html").exists(),
        "Full datetime-prefixed post should strip datetime from slug"
    );

    // Stream+date prefix
    assert!(
        output_dir.join("news-breaking.html").exists(),
        "Stream+date post should have stream-prefixed slug"
    );
    assert!(
        output_dir.join("news.html").exists(),
        "Stream page should be generated"
    );

    // Stream+S prefix
    assert!(
        output_dir.join("guide-complete-tutorial.html").exists(),
        "Stream+S post should have stream-prefixed slug"
    );
    assert!(
        output_dir.join("guide.html").exists(),
        "Guide stream page should be generated"
    );

    // JSON frontmatter
    assert!(
        output_dir.join("json-format.html").exists(),
        "JSON frontmatter post should render with slug from title"
    );

    // TOML frontmatter
    assert!(
        output_dir.join("toml-format.html").exists(),
        "TOML frontmatter post should render with slug from title"
    );

    // No frontmatter
    assert!(
        output_dir.join("no-frontmatter.html").exists(),
        "File with no frontmatter should render as a page"
    );

    // Fragment should NOT produce an HTML file
    assert!(
        !output_dir.join("hero.html").exists(),
        "Fragment (_hero.md) should not produce output"
    );
    assert!(
        !output_dir.join("_hero.html").exists(),
        "Fragment should not produce output with underscore"
    );

    // Frontmatter date override
    assert!(
        output_dir.join("overridden-date.html").exists(),
        "Post with frontmatter date override should render"
    );

    // Subfolder with frontmatter.yaml
    assert!(
        output_dir.join("python-python-basics.html").exists(),
        "Subfolder post should get stream prefix from frontmatter.yaml"
    );
    assert!(
        output_dir.join("python.html").exists(),
        "Python stream page should exist"
    );

    // Translation group
    let has_hello =
        output_dir.join("hello.html").exists() || output_dir.join("en-hello.html").exists();
    let has_pt = output_dir.join("pt-ola.html").exists() || output_dir.join("ola.html").exists();
    assert!(has_hello, "English translation should render");
    assert!(has_pt, "Portuguese translation should render");

    // Dated subfolder
    assert!(
        output_dir.join("dated-folder-post.html").exists(),
        "Post in dated subfolder should render (date extracted from folder name)"
    );
}

#[test]
fn test_date_formats_in_filenames() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let content_dir = input_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Test").unwrap();

    // YYYY-MM-DD only
    fs::write(content_dir.join("2024-01-01-date-only.md"), "# Date Only\n").unwrap();

    // YYYY-MM-DD-HH-MM
    fs::write(
        content_dir.join("2024-02-15-14-30-with-time.md"),
        "# With Time\n",
    )
    .unwrap();

    // YYYY-MM-DD-HH-MM-SS
    fs::write(
        content_dir.join("2024-03-20-08-15-30-with-seconds.md"),
        "# With Seconds\n",
    )
    .unwrap();

    let (output_dir, output) = build_site(&input_dir);
    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_dir.join("date-only.html").exists(),
        "YYYY-MM-DD prefix should be stripped"
    );
    assert!(
        output_dir.join("with-time.html").exists(),
        "YYYY-MM-DD-HH-MM prefix should be stripped"
    );
    assert!(
        output_dir.join("with-seconds.html").exists(),
        "YYYY-MM-DD-HH-MM-SS prefix should be stripped"
    );
}

#[test]
fn test_stream_filename_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let content_dir = input_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Test").unwrap();

    // stream-YYYY-MM-DD-slug
    fs::write(
        content_dir.join("tutorial-2024-05-01-lesson-one.md"),
        "# Lesson One\n",
    )
    .unwrap();

    // stream-S-slug with date in frontmatter
    fs::write(
        content_dir.join("blog-S-my-thoughts.md"),
        "---\ndate: 2024-06-01\n---\n# My Thoughts\n",
    )
    .unwrap();

    // stream-YYYY-MM-DD-HH-MM-SS-slug
    fs::write(
        content_dir.join("news-2024-07-01-12-00-00-update.md"),
        "# Update\n",
    )
    .unwrap();

    let (output_dir, output) = build_site(&input_dir);
    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_dir.join("tutorial-lesson-one.html").exists(),
        "stream-date pattern should produce stream-slug"
    );
    assert!(
        output_dir.join("tutorial.html").exists(),
        "tutorial stream page should exist"
    );

    assert!(
        output_dir.join("blog-my-thoughts.html").exists(),
        "stream-S pattern should produce stream-slug"
    );
    assert!(
        output_dir.join("blog.html").exists(),
        "blog stream page should exist"
    );

    assert!(
        output_dir.join("news-update.html").exists(),
        "stream-datetime pattern should produce stream-slug"
    );
    assert!(
        output_dir.join("news.html").exists(),
        "news stream page should exist"
    );
}

#[test]
fn test_frontmatter_format_diversity() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let content_dir = input_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Test").unwrap();

    // YAML
    fs::write(
        content_dir.join("2024-01-01-yaml.md"),
        "---\ntitle: YAML Post\ntags: [test]\n---\n# YAML\n",
    )
    .unwrap();

    // TOML
    fs::write(
        content_dir.join("toml-page.md"),
        "+++\ntitle = \"TOML Page\"\ndate = 2024-01-02\n+++\n\n# TOML\n",
    )
    .unwrap();

    // JSON
    fs::write(
        content_dir.join("2024-01-03-json.md"),
        "{\"title\": \"JSON Post\", \"tags\": [\"test\"]}\n\n# JSON\n",
    )
    .unwrap();

    // No frontmatter
    fs::write(
        content_dir.join("plain.md"),
        "# Plain Page\n\nNo frontmatter here.\n",
    )
    .unwrap();

    let (output_dir, output) = build_site(&input_dir);
    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_dir.join("yaml-post.html").exists(),
        "YAML frontmatter should work"
    );
    assert!(
        output_dir.join("toml-page.html").exists(),
        "TOML frontmatter should work"
    );
    assert!(
        output_dir.join("json-post.html").exists(),
        "JSON frontmatter should work"
    );
    assert!(
        output_dir.join("plain.html").exists(),
        "No frontmatter should work"
    );
}

#[test]
fn test_messy_flat_directory() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let content_dir = input_dir.join("content");
    fs::create_dir_all(&content_dir).unwrap();
    fs::write(input_dir.join("marmite.yaml"), "name: Messy Blog").unwrap();

    // Mix of everything in a flat directory - the "zero config, just markdown" use case
    fs::write(content_dir.join("my-page.md"), "# My Page\n").unwrap();
    fs::write(content_dir.join("2024-01-01-first.md"), "# First Post\n").unwrap();
    fs::write(
        content_dir.join("2024-02-01-second.md"),
        "---\ntags: blog\n---\n# Second Post\n",
    )
    .unwrap();
    fs::write(
        content_dir.join("2024-03-01-15-30-third.md"),
        "# Third Post\n",
    )
    .unwrap();
    fs::write(
        content_dir.join("page-no-date.md"),
        "{\"title\": \"JSON Page\"}\n\n# JSON Page\n",
    )
    .unwrap();
    fs::write(
        content_dir.join("_fragment.md"),
        "This is a **fragment**.\n",
    )
    .unwrap();
    fs::write(
        content_dir.join("another-page.md"),
        "+++\ntitle = \"TOML Page\"\n+++\n\n# TOML\n",
    )
    .unwrap();

    let (output_dir, output) = build_site(&input_dir);
    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_dir.join("my-page.html").exists());
    assert!(output_dir.join("first.html").exists());
    assert!(output_dir.join("second.html").exists());
    assert!(output_dir.join("third.html").exists());
    assert!(output_dir.join("json-page.html").exists());
    assert!(output_dir.join("toml-page.html").exists());
    assert!(!output_dir.join("fragment.html").exists());
    assert!(!output_dir.join("_fragment.html").exists());

    assert!(
        output_dir.join("index.html").exists(),
        "Index should be generated for messy flat directory"
    );
}
