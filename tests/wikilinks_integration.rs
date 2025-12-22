use std::fs;
use tempfile::TempDir;

#[test]
fn test_wikilinks_integration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create content directory structure
    let content_dir = temp_path.join("content");
    fs::create_dir_all(&content_dir).unwrap();

    // Create a post that will be referenced via wikilink
    let target_post = content_dir.join("target-post.md");
    fs::write(
        &target_post,
        r#"---
title: "My Amazing Post"
slug: amazing-post-custom-slug
---

This is the target post.
"#,
    )
    .unwrap();

    // Create a post with wikilinks
    let source_post = content_dir.join("source-post.md");
    fs::write(
        &source_post,
        r#"---
title: "Source Post"
---

This post links to [[My Amazing Post]] using wikilinks.
Also links to [[Nonexistent Post]] which should remain unchanged.
"#,
    )
    .unwrap();

    // Create config file
    let config_file = temp_path.join("marmite.yaml");
    fs::write(
        &config_file,
        "
name: Test Blog
url: https://test.blog
",
    )
    .unwrap();

    // Generate the site
    let output_dir = temp_path.join("site");
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            temp_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            "--force",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to run marmite");

    assert!(status.success(), "Marmite generation failed");

    // Check the generated source post HTML
    let source_html_file = output_dir.join("source-post.html");
    assert!(source_html_file.exists(), "Source post HTML not generated");

    let html_content = fs::read_to_string(source_html_file).unwrap();

    // Check that the wikilink was properly resolved to use the custom slug
    assert!(
        html_content.contains(
            r#"<a href="amazing-post-custom-slug.html" data-wikilink="true">My Amazing Post</a>"#
        ),
        "Wikilink was not properly resolved to custom slug. HTML content: {html_content}"
    );

    // Check that the nonexistent wikilink remained unchanged
    assert!(
        html_content.contains(r#"data-wikilink="true">Nonexistent Post</a>"#),
        "Nonexistent wikilink should retain data-wikilink attribute. HTML content: {html_content}"
    );
}
