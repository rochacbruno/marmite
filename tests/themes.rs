use std::fs;
use std::process::Command;
use tempfile::TempDir;

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

    assert!(
        output.status.success(),
        "Site init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    assert!(
        output.status.success(),
        "Start theme failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify theme structure was created
    let theme_dir = site_dir.join(theme_name);
    assert!(theme_dir.exists(), "Theme directory not created");
    assert!(
        theme_dir.join("templates").exists(),
        "Theme templates directory not created"
    );
    assert!(
        theme_dir.join("templates").join("base.html").exists(),
        "base.html not created"
    );
    assert!(
        theme_dir.join("templates").join("content.html").exists(),
        "content.html not created"
    );
    assert!(
        theme_dir.join("templates").join("list.html").exists(),
        "list.html not created"
    );
    assert!(
        theme_dir.join("static").exists(),
        "Theme static directory not created"
    );
    assert!(
        theme_dir.join("static").join("custom.css").exists(),
        "custom.css not created"
    );
    assert!(
        theme_dir.join("theme.json").exists(),
        "theme.json not created"
    );

    // Step 3: Customize the theme - modify base.html template
    let base_template_path = theme_dir.join("templates").join("base.html");
    let base_content = fs::read_to_string(&base_template_path).unwrap();

    // Add a custom marker to the template that we can verify later
    let custom_base = base_content.replace("<body>", "<body>\n    <!-- CUSTOM_THEME_MARKER -->");
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

    assert!(
        output.status.success(),
        "Set theme failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify theme was added to config
    let config_content = fs::read_to_string(site_dir.join("marmite.yaml")).unwrap();
    assert!(
        config_content.contains(&format!("theme: {}", theme_name)),
        "Theme not set in config"
    );

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

    assert!(
        output.status.success(),
        "Site generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 8: Verify the generated site uses the custom theme

    // Check that pages were generated (slug is based on title)
    assert!(
        output_dir.join("test-post-with-theme.html").exists(),
        "Post not generated"
    );
    assert!(
        output_dir.join("about-page.html").exists(),
        "Page not generated"
    );
    assert!(
        output_dir.join("index.html").exists(),
        "Index not generated"
    );

    // Check that custom CSS was copied from site root to static
    assert!(
        output_dir.join("static").join("custom.css").exists(),
        "Custom CSS not copied"
    );

    // Verify custom CSS content
    let output_css = fs::read_to_string(output_dir.join("static").join("custom.css")).unwrap();
    assert!(
        output_css.contains(".custom-theme-class"),
        "Custom CSS class not found"
    );
    assert!(
        output_css.contains("background-color: #f5f5f5"),
        "Custom CSS rule not found"
    );

    // Check that theme static files were copied
    assert!(
        output_dir.join("static").join("theme.css").exists(),
        "Theme CSS not copied"
    );
    let theme_css = fs::read_to_string(output_dir.join("static").join("theme.css")).unwrap();
    assert!(
        theme_css.contains(".theme-specific"),
        "Theme CSS content not found"
    );

    // Verify the custom template marker is in generated HTML
    let post_html = fs::read_to_string(output_dir.join("test-post-with-theme.html")).unwrap();
    assert!(
        post_html.contains("<!-- CUSTOM_THEME_MARKER -->"),
        "Custom template marker not found in post"
    );

    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(
        index_html.contains("<!-- CUSTOM_THEME_MARKER -->"),
        "Custom template marker not found in index"
    );

    // Verify that the post content is present
    assert!(
        post_html.contains("Test Post with Theme"),
        "Post title not found"
    );
    assert!(
        post_html.contains("This is a test post"),
        "Post content not found"
    );

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

    assert!(
        output.status.success(),
        "Site generation with --theme failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the theme was applied via CLI flag
    assert!(
        output_dir2.join("test-post-with-theme.html").exists(),
        "Post not generated with --theme flag"
    );
    let post_html2 = fs::read_to_string(output_dir2.join("test-post-with-theme.html")).unwrap();
    assert!(
        post_html2.contains("<!-- CUSTOM_THEME_MARKER -->"),
        "Custom template not applied with --theme flag"
    );
}
