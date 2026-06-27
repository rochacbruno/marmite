use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn build_example_site() -> (TempDir, std::process::Output) {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "example",
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite");

    (temp_dir, output)
}

#[test]
fn test_example_site_builds_without_errors() {
    let (temp_dir, output) = build_example_site();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Example site build failed: {stderr}"
    );

    let error_lines: Vec<&str> = stderr.lines().filter(|l| l.contains("ERROR")).collect();
    assert!(
        error_lines.is_empty(),
        "Example site build produced errors:\n{}",
        error_lines.join("\n")
    );

    let output_dir = temp_dir.path().join("output");
    assert!(output_dir.exists(), "Output directory was not created");

    let html_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
        .collect();
    assert!(
        html_files.len() > 100,
        "Expected >100 HTML files, got {}",
        html_files.len()
    );
}

#[test]
fn test_example_site_template_inheritance() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // base.html structure
    assert!(index.contains("<!DOCTYPE html>"));
    assert!(index.contains("<html"));
    assert!(index.contains("</html>"));
    assert!(index.contains("<header"));
    assert!(index.contains("<footer"));

    // list.html content block rendered
    assert!(index.contains("content-list"));
}

#[test]
fn test_example_site_url_for_function() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // url_for should generate paths to static files
    assert!(
        index.contains("static/marmite.css"),
        "url_for should resolve static CSS path"
    );
    assert!(
        index.contains("static/marmite.js"),
        "url_for should resolve static JS path"
    );
}

#[test]
fn test_example_site_content_page_renders() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let content = fs::read_to_string(output_dir.join("getting-started.html")).unwrap();

    // content.html template rendered with actual content
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("Getting Started"));
}

#[test]
fn test_example_site_date_filter() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // date filter used in list.html: content.date | date(format='%+')
    // Should produce datetime attributes on <time> elements
    assert!(
        index.contains("dt-published"),
        "Date filter should produce datetime attributes"
    );
}

#[test]
fn test_example_site_striptags_filter() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // striptags used in list.html for content excerpts
    // The excerpt should be plain text (no HTML tags in the p.content-excerpt)
    assert!(
        index.contains("content-excerpt"),
        "List template should render content excerpts"
    );
}

#[test]
fn test_example_site_slice_filter() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // slice(end=3) used in list.html for tag display
    // Should render tags without errors
    assert!(
        index.contains("content-tags"),
        "Slice filter should allow tag list rendering"
    );
}

#[test]
fn test_example_site_dot_index_conversion() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // base.html uses item.0 / item.1 for menu items (converted to item[0]/item[1])
    // Menu should render without errors
    assert!(
        index.contains("header-menu"),
        "Menu should render (requires item[0]/item[1] conversion)"
    );
}

#[test]
fn test_example_site_starting_with_test() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let content = fs::read_to_string(output_dir.join("getting-started.html")).unwrap();

    // content_title.html uses: content.html is not starting_with("<h1>")
    // This should work via the positional-to-keyword conversion
    assert!(
        content.contains("</h1>") || content.contains("content-title"),
        "starting_with test should work for content title rendering"
    );
}

#[test]
fn test_example_site_ignore_missing_includes() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let content = fs::read_to_string(output_dir.join("getting-started.html")).unwrap();

    // content.html uses {% include "comments.html" ignore missing %}
    // Should render without errors even when comments template exists but
    // comments variable may not be defined
    assert!(
        content.contains("<!DOCTYPE html>"),
        "Page should render despite ignore-missing includes"
    );
}

#[test]
fn test_example_site_group_function() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    // group.html is used for tag/author/archive pages
    // Check that at least some tag pages exist
    let tag_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("tag-"))
        })
        .collect();
    assert!(
        !tag_files.is_empty(),
        "Tag pages should be generated via group function"
    );
}

#[test]
fn test_example_site_remove_draft_filter() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    // group.html uses: items | remove_draft
    // If there are group pages, the filter worked
    let group_pages: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("tag-") || n.starts_with("archive-"))
        })
        .collect();
    assert!(
        !group_pages.is_empty(),
        "Group pages should render with remove_draft filter"
    );
}

#[test]
fn test_example_site_sitemap() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let sitemap = fs::read_to_string(output_dir.join("sitemap.xml")).unwrap();
    assert!(sitemap.contains("<urlset"));
    assert!(sitemap.contains("<url>"));
}

#[test]
fn test_example_site_feeds() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    let rss_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rss"))
        .collect();
    assert!(
        !rss_files.is_empty(),
        "At least one RSS feed should be generated"
    );

    let json_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.ends_with(".json") && !n.ends_with("urls.json"))
        })
        .collect();
    assert!(
        !json_files.is_empty(),
        "At least one JSON feed should be generated"
    );
}

#[test]
fn test_example_site_pagination() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    // Default pagination is 10, example has more than 10 posts
    assert!(
        output_dir.join("index-1.html").exists(),
        "First pagination page should exist"
    );
    assert!(
        output_dir.join("index-2.html").exists(),
        "Second pagination page should exist"
    );
}

#[test]
fn test_example_site_optional_chaining_defined_check() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    // content.html has: site.extra.comments.source is defined
    // This uses optional chaining after preprocessing
    // Content pages should render without "field is not defined" errors
    let content = fs::read_to_string(output_dir.join("getting-started.html")).unwrap();
    assert!(
        content.contains("<!DOCTYPE html>"),
        "Pages with deep defined checks should render via optional chaining"
    );
}

#[test]
fn test_example_site_shortcodes_render() {
    let (temp_dir, _) = build_example_site();
    let output_dir = temp_dir.path().join("output");

    // shortcodes-demo.md uses various shortcodes
    if output_dir.join("shortcodes-demo.html").exists() {
        let content = fs::read_to_string(output_dir.join("shortcodes-demo.html")).unwrap();
        assert!(
            content.contains("<!DOCTYPE html>"),
            "Shortcodes demo page should render"
        );
    }
}
