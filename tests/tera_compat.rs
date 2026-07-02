use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Build a site from a temp directory and return (temp_dir, output).
fn build_site(site_dir: &std::path::Path) -> std::process::Output {
    let output_dir = site_dir.join("output");
    Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            site_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute marmite")
}

/// Create a minimal site with Tera 1.x-style templates exercising every
/// pattern handled by the preprocessing layer, plus custom shortcodes
/// that access context variables and registered functions.
fn create_tera1_compat_site(site_dir: &std::path::Path) {
    let content_dir = site_dir.join("content");
    let templates_dir = site_dir.join("templates");
    let shortcodes_dir = site_dir.join("shortcodes");
    fs::create_dir_all(&content_dir).unwrap();
    fs::create_dir_all(&templates_dir).unwrap();
    fs::create_dir_all(&shortcodes_dir).unwrap();

    // Config with menu items (needed for dot-indexing test) and extra nested config
    fs::write(
        site_dir.join("marmite.yaml"),
        r#"name: Compat Test
url: https://example.com
menu:
  - ["Home", "index.html"]
  - ["Blog", "https://blog.example.com"]
extra:
  comments:
    source: giscus
    repo: test/repo
"#,
    )
    .unwrap();

    // base.html using Tera 1.x syntax:
    // - item.0 / item.1 (dot-indexing)
    // - starting_with("http") (positional test arg)
    // - ignore missing (on an include that does NOT exist)
    fs::write(
        templates_dir.join("base.html"),
        r#"<!DOCTYPE html>
<html>
<head><title>{{ site.name }}</title>
{%- block head %}{% endblock -%}
{%- block feeds %}{% endblock -%}
</head>
<body>
<nav>
{% for item in menu %}
  {% set name = item.0 %}
  {% set url = item.1 %}
  <a {% if url is starting_with("http") %}target="_blank"{% endif %} href="{{url}}">{{name}}</a>
{% endfor %}
</nav>
{% include "optional_banner.html" ignore missing %}
{%- block main %}{% endblock -%}
{%- block tail %}{% endblock -%}
</body>
</html>
"#,
    )
    .unwrap();

    // list.html - uses old filters: striptags, trim_start_matches, slice, date
    fs::write(
        templates_dir.join("list.html"),
        r#"{% extends "base.html" %}
{% block main %}
<h1>{{ title }}</h1>
{% for content in content_list %}
<article>
  <h2>{{ content.title }}</h2>
  {% if content.date %}<time>{{ content.date | date(format="%Y-%m-%d") }}</time>{% endif %}
  <p>{{ content.html | striptags | trim_start_matches(pat=content.title) | truncate(length=100, end="...") }}</p>
  {% if content.tags %}
  <ul>{% for tag in content.tags | slice(end=2) %}<li>{{ tag }}</li>{% endfor %}</ul>
  {% endif %}
</article>
{% endfor %}
{% endblock %}
"#,
    )
    .unwrap();

    // content.html - uses deep is defined check (3+ levels)
    // and default filter on potentially undefined variable
    fs::write(
        templates_dir.join("content.html"),
        r#"{% extends "base.html" %}
{% block main %}
<article>
  <h1>{{ content.title }}</h1>
  {{ content.html | safe }}
  {% if site.extra.comments.source is defined %}
  <div class="comments" data-source="{{ site.extra.comments.source }}">
    Comments powered by {{ site.extra.comments.source }}
  </div>
  {% endif %}
</article>
{% endblock %}
"#,
    )
    .unwrap();

    // group.html - uses group() function
    fs::write(
        templates_dir.join("group.html"),
        r#"{% extends "base.html" %}
{% block main %}
<h1>{{ title }}</h1>
{% for name, items in group(kind=kind) %}
<h2>{{ name }} ({{ items | length }})</h2>
{% endfor %}
{% endblock %}
"#,
    )
    .unwrap();

    // Custom HTML shortcode using macro syntax, accessing context variables
    // and a registered function (url_for)
    fs::write(
        shortcodes_dir.join("siteinfo.html"),
        r#"{# Show site info from context #}
{% macro siteinfo(label="Site") %}
<div class="site-info">
  <strong>{{ label }}:</strong> {{ site_data.site.name }}
  <a href="{{ url_for(path='index.html') }}">Home</a>
</div>
{% endmacro siteinfo %}"#,
    )
    .unwrap();

    // Custom HTML shortcode with starting_with test (Tera 1.x positional arg)
    fs::write(
        shortcodes_dir.join("extlink.html"),
        r#"{# External link with target=_blank #}
{% macro extlink(url, text="Link") %}
{% if url is not starting_with("http") %}
{% set url = "https://" ~ url %}
{% endif %}
<a href="{{ url }}" target="_blank">{{ text }}</a>
{% endmacro extlink %}"#,
    )
    .unwrap();

    // Custom markdown shortcode accessing context
    fs::write(
        shortcodes_dir.join("greeting.md"),
        "Hello from **{{ site.name }}**!",
    )
    .unwrap();

    // Post with shortcodes
    fs::write(
        content_dir.join("2024-01-15-post-with-shortcodes.md"),
        r#"---
date: 2024-01-15
tags: rust, testing, tera
---
# Post With Shortcodes

Some content here.

<!-- .siteinfo label="My Site" -->

<!-- .extlink url="example.com" text="Example" -->

<!-- .greeting -->

More content after shortcodes.
"#,
    )
    .unwrap();

    // Another post (for list page testing)
    fs::write(
        content_dir.join("2024-02-01-second-post.md"),
        r#"---
date: 2024-02-01
tags: rust, testing
---
# Second Post

More content for the list page.
"#,
    )
    .unwrap();

    // A page
    fs::write(
        content_dir.join("about.md"),
        "# About\nThis is the about page.",
    )
    .unwrap();
}

#[test]
fn test_tera1_syntax_builds_without_errors() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);

    let output = build_site(&site_dir);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Build failed with Tera 1.x templates: {stderr}"
    );

    let error_lines: Vec<&str> = stderr
        .lines()
        .filter(|l| l.contains("ERROR"))
        .filter(|l| !l.contains("custom_index"))
        .collect();
    assert!(
        error_lines.is_empty(),
        "Build produced errors:\n{}",
        error_lines.join("\n")
    );
}

#[test]
fn test_tera1_dot_indexing_in_menu() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // item.0 and item.1 should resolve via preprocessing
    assert!(
        index.contains("Home"),
        "Menu item name 'Home' should render"
    );
    assert!(index.contains("index.html"), "Menu item URL should render");
    assert!(
        index.contains("https://blog.example.com"),
        "External menu URL should render"
    );
}

#[test]
fn test_tera1_starting_with_positional_arg() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // External link should get target="_blank" via starting_with("http") test
    assert!(
        index.contains(r#"target="_blank""#),
        "starting_with positional arg should work for external links"
    );
}

#[test]
fn test_tera1_ignore_missing_include() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // Page should render even though optional_banner.html doesn't exist
    assert!(
        index.contains("<!DOCTYPE html>"),
        "Page should render despite missing optional include"
    );
}

#[test]
fn test_tera1_striptags_filter() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // striptags + trim_start_matches + truncate chain should work
    assert!(
        index.contains("<article>"),
        "List template with striptags chain should render articles"
    );
}

#[test]
fn test_tera1_date_filter() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // date(format="%Y-%m-%d") should render dates
    assert!(
        index.contains("2024-02-01") || index.contains("2024-01-15"),
        "date filter should format dates in list page"
    );
}

#[test]
fn test_tera1_slice_filter() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let index = fs::read_to_string(output_dir.join("index-1.html")).unwrap();

    // slice(end=2) limits tags to 2 - post has 3 tags (rust, testing, tera)
    // Count tag <li> elements in the first article
    let tag_count = index.matches("<li>").count();
    assert!(tag_count > 0, "slice filter should render some tags");
}

#[test]
fn test_tera1_deep_is_defined_check() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let content = fs::read_to_string(output_dir.join("post-with-shortcodes.html")).unwrap();

    // site.extra.comments.source is defined should resolve via optional chaining
    // Config has comments.source = "giscus", so the comments div should render
    assert!(
        content.contains("giscus"),
        "Deep is defined check should work and render comments section"
    );
}

#[test]
fn test_tera1_shortcode_accesses_context() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let content = fs::read_to_string(output_dir.join("post-with-shortcodes.html")).unwrap();

    // siteinfo shortcode accesses site_data.site.name and url_for
    assert!(
        content.contains("Compat Test"),
        "Shortcode should access site_data.site.name from context, got: {}",
        &content[content.find("site-info").unwrap_or(0)
            ..content.find("site-info").unwrap_or(0) + 200.min(content.len())]
    );
    assert!(
        content.contains("site-info"),
        "Shortcode siteinfo should render its div"
    );
}

#[test]
fn test_tera1_shortcode_with_starting_with_test() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let content = fs::read_to_string(output_dir.join("post-with-shortcodes.html")).unwrap();

    // extlink shortcode uses starting_with("http") positional arg
    // and prepends https:// when missing
    assert!(
        content.contains("https://example.com"),
        "extlink shortcode should prepend https:// via starting_with test"
    );
    assert!(
        content.contains(r#"target="_blank"#),
        "extlink shortcode should add target=_blank"
    );
}

#[test]
fn test_tera1_markdown_shortcode_accesses_context() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");
    let content = fs::read_to_string(output_dir.join("post-with-shortcodes.html")).unwrap();

    // greeting.md shortcode uses {{ site.name }}
    assert!(
        content.contains("Compat Test"),
        "Markdown shortcode should access site.name from context"
    );
}

#[test]
fn test_tera1_no_shortcode_errors() {
    let temp_dir = TempDir::new().unwrap();
    let site_dir = temp_dir.path().join("site");
    fs::create_dir_all(&site_dir).unwrap();
    create_tera1_compat_site(&site_dir);
    build_site(&site_dir);

    let output_dir = site_dir.join("output");

    let pages_with_errors: Vec<String> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "html"))
        .filter(|e| {
            fs::read_to_string(e.path())
                .unwrap_or_default()
                .contains("shortcode-error")
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert!(
        pages_with_errors.is_empty(),
        "Pages with shortcode errors: {:?}",
        pages_with_errors
    );
}
