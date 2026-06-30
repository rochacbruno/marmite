use super::*;
use tempfile::TempDir;

#[test]
fn test_shortcode_pattern() {
    let processor = ShortcodeProcessor::new(None);
    let html = r"<p>Some text</p>
<!-- .youtube id=abc123 -->
<p>More text</p>
<!-- .toc -->
<!-- .authors -->";

    let matches: Vec<_> = processor.pattern.captures_iter(html).collect();
    assert_eq!(matches.len(), 3);
    assert_eq!(&matches[0][1], "youtube");
    assert_eq!(&matches[1][1], "toc");
    assert_eq!(&matches[2][1], "authors");
}

#[test]
fn test_shortcode_pattern_with_params() {
    let processor = ShortcodeProcessor::new(None);
    let html = r#"<p>Some text</p>
<!-- .posts -->
<!-- .posts ord=asc items=5 -->
<!-- .youtube id=abc123 width=400 height=300 -->
<!-- .test foo=noquote bar='single' zaz="double" -->
"#;

    let matches: Vec<_> = processor.pattern.captures_iter(html).collect();
    assert_eq!(matches.len(), 4);

    // First match: .posts with no params
    assert_eq!(&matches[0][1], "posts");
    assert!(matches[0].get(2).is_none());

    // Second match: .posts with params
    assert_eq!(&matches[1][1], "posts");
    let params = matches[1].get(2).unwrap().as_str().trim();
    assert_eq!(params, "ord=asc items=5");

    // Third match: .youtube with params
    assert_eq!(&matches[2][1], "youtube");
    let params = matches[2].get(2).unwrap().as_str().trim();
    assert_eq!(params, "id=abc123 width=400 height=300");

    // Fourth match: .test with params
    assert_eq!(&matches[3][1], "test");
    let params = matches[3].get(2).unwrap().as_str().trim();
    assert_eq!(params, "foo=noquote bar='single' zaz=\"double\"");
}

#[test]
fn test_builtin_shortcodes() {
    let mut processor = ShortcodeProcessor::new(None);
    processor.add_builtin_shortcodes();

    assert!(processor.shortcodes.contains_key("toc"));
    assert!(processor.shortcodes.contains_key("youtube"));
    assert!(processor.shortcodes.contains_key("authors"));
    assert!(processor.shortcodes.contains_key("streams"));
    assert!(processor.shortcodes.contains_key("series"));
    assert!(processor.shortcodes.contains_key("posts"));
    assert!(processor.shortcodes.contains_key("pages"));
    assert!(processor.shortcodes.contains_key("tags"));
    assert!(processor.shortcodes.contains_key("socials"));
}

#[test]
fn test_load_shortcode_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let shortcodes_dir = temp_dir.path().join("shortcodes");
    fs::create_dir(&shortcodes_dir).unwrap();

    // Create a test HTML shortcode
    let test_html = r"{% macro test() %}
<div>Test shortcode</div>
{% endmacro test %}";
    fs::write(shortcodes_dir.join("test.html"), test_html).unwrap();

    // Create a test markdown shortcode
    let test_md = "# Test Markdown\n\nThis is a test.";
    fs::write(shortcodes_dir.join("testmd.md"), test_md).unwrap();

    let mut processor = ShortcodeProcessor::new(None);
    processor.collect_shortcodes(temp_dir.path()).unwrap();

    assert!(processor.shortcodes.contains_key("test"));
    assert!(processor.shortcodes.contains_key("testmd"));
    assert!(processor.shortcodes.get("test").unwrap().is_html);
    assert!(!processor.shortcodes.get("testmd").unwrap().is_html);
}

#[test]
fn test_html_shortcode_must_contain_macro_with_same_name() {
    let temp_dir = TempDir::new().unwrap();
    let shortcodes_dir = temp_dir.path().join("shortcodes");
    fs::create_dir(&shortcodes_dir).unwrap();

    // Create an HTML shortcode with wrong macro name
    let wrong_macro = r"{% macro bar() %}
<div>Wrong macro name</div>
{% endmacro bar %}";
    fs::write(shortcodes_dir.join("foo.html"), wrong_macro).unwrap();

    let mut processor = ShortcodeProcessor::new(None);
    let result = processor.collect_shortcodes(temp_dir.path());

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("must contain a macro named 'foo'"));
}

#[test]
fn test_html_shortcode_with_multiple_macros() {
    let temp_dir = TempDir::new().unwrap();
    let shortcodes_dir = temp_dir.path().join("shortcodes");
    fs::create_dir(&shortcodes_dir).unwrap();

    // Create an HTML shortcode with multiple macros including the correct one
    let multi_macro = r"{% macro helper() %}
<span>Helper</span>
{% endmacro helper %}

{% macro multi() %}
<div>Correct macro</div>
{% endmacro multi %}";
    fs::write(shortcodes_dir.join("multi.html"), multi_macro).unwrap();

    let mut processor = ShortcodeProcessor::new(None);
    processor.collect_shortcodes(temp_dir.path()).unwrap();

    assert!(processor.shortcodes.contains_key("multi"));
}

#[test]
fn test_html_shortcode_without_any_macro() {
    let temp_dir = TempDir::new().unwrap();
    let shortcodes_dir = temp_dir.path().join("shortcodes");
    fs::create_dir(&shortcodes_dir).unwrap();

    // Create an HTML shortcode without any macro
    let no_macro = "<div>Just HTML, no macro</div>";
    fs::write(shortcodes_dir.join("nomacro.html"), no_macro).unwrap();

    let mut processor = ShortcodeProcessor::new(None);
    let result = processor.collect_shortcodes(temp_dir.path());

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("must contain at least one macro"));
}

#[test]
fn test_shortcode_description_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let shortcodes_dir = temp_dir.path().join("shortcodes");
    fs::create_dir(&shortcodes_dir).unwrap();

    // Create an HTML shortcode with description
    let with_desc = r#"{# Display a custom alert box #}
{% macro alert(type="info", message) %}
<div class="alert alert-{{type}}">{{message}}</div>
{% endmacro alert %}"#;
    fs::write(shortcodes_dir.join("alert.html"), with_desc).unwrap();

    // Create an HTML shortcode without description
    let without_desc = r#"{% macro info() %}
<div class="info">Information</div>
{% endmacro info %}"#;
    fs::write(shortcodes_dir.join("info.html"), without_desc).unwrap();

    // Create a markdown shortcode with description
    let md_with_desc = "{# List of recent posts #}\n## Recent Posts\n\nThis is markdown content.";
    fs::write(shortcodes_dir.join("recent.md"), md_with_desc).unwrap();

    let mut processor = ShortcodeProcessor::new(None);
    processor.collect_shortcodes(temp_dir.path()).unwrap();

    // Check HTML shortcode with description
    let alert = processor.shortcodes.get("alert").unwrap();
    assert_eq!(
        alert.description,
        Some("Display a custom alert box".to_string())
    );

    // Check HTML shortcode without description
    let info = processor.shortcodes.get("info").unwrap();
    assert_eq!(info.description, None);

    // Check markdown shortcode with description
    let recent = processor.shortcodes.get("recent").unwrap();
    assert_eq!(recent.description, Some("List of recent posts".to_string()));
}

#[test]
fn test_list_shortcodes_with_descriptions() {
    let mut processor = ShortcodeProcessor::new(None);
    processor.add_builtin_shortcodes();

    let shortcodes = processor.list_shortcodes_with_descriptions();

    // Check they're sorted alphabetically
    let names: Vec<&str> = shortcodes.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        names,
        vec![
            "authors", "card", "gallery", "pages", "posts", "series", "socials", "spotify",
            "streams", "tags", "toc", "youtube"
        ]
    );

    // Check descriptions are present for HTML shortcodes
    for (name, desc) in shortcodes {
        if name != "socials" {
            // socials.md might not have description
            assert!(desc.is_some(), "Shortcode {name} should have a description");
        }
    }
}

#[test]
fn test_parse_parameters() {
    // Test simple parameters
    let params = ShortcodeProcessor::parse_parameters("key1=value1 key2=value2");
    assert_eq!(
        params,
        vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string())
        ]
    );

    // Test quoted parameters with spaces
    let params = ShortcodeProcessor::parse_parameters(
        r#"title="Custom Title" text='Single quoted' url=http://example.com"#,
    );
    assert_eq!(
        params,
        vec![
            ("title".to_string(), r#""Custom Title""#.to_string()),
            ("text".to_string(), "'Single quoted'".to_string()),
            ("url".to_string(), "http://example.com".to_string())
        ]
    );

    // Test mixed parameters
    let params = ShortcodeProcessor::parse_parameters(
        r#"slug=author-rochacbruno image="https://github.com/dynaconf.png" title="Custom Title" text='Custom Description' content_type="Author""#,
    );
    assert_eq!(
        params,
        vec![
            ("slug".to_string(), "author-rochacbruno".to_string()),
            (
                "image".to_string(),
                r#""https://github.com/dynaconf.png""#.to_string()
            ),
            ("title".to_string(), r#""Custom Title""#.to_string()),
            ("text".to_string(), "'Custom Description'".to_string()),
            ("content_type".to_string(), r#""Author""#.to_string())
        ]
    );

    // Test parameters with spaces around equals
    let params = ShortcodeProcessor::parse_parameters("key1 = value1   key2= \"quoted value\"");
    assert_eq!(
        params,
        vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), r#""quoted value""#.to_string())
        ]
    );

    // Test empty parameters
    let params = ShortcodeProcessor::parse_parameters("");
    assert_eq!(params, vec![]);

    // Test just whitespace
    let params = ShortcodeProcessor::parse_parameters("   ");
    assert_eq!(params, vec![]);
}

// --- extract_macro_body tests ---

#[test]
fn test_extract_macro_body_basic() {
    let input = r#"{% macro youtube(id, width="560") %}
<iframe src="{{id}}" width="{{width}}"></iframe>
{% endmacro youtube %}"#;
    let (body, params) = ShortcodeProcessor::extract_macro_body(input, "youtube").unwrap();
    assert!(body.contains("<iframe"));
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "id");
    assert!(params[0].default.is_none());
    assert_eq!(params[1].name, "width");
    assert_eq!(params[1].default.as_deref(), Some("560"));
}

#[test]
fn test_extract_macro_body_no_args() {
    let input = r#"{% macro toc() %}
<nav>Table of contents</nav>
{% endmacro toc %}"#;
    let (body, params) = ShortcodeProcessor::extract_macro_body(input, "toc").unwrap();
    assert!(body.contains("<nav>"));
    assert!(params.is_empty());
}

#[test]
fn test_extract_macro_body_preserves_body() {
    let input = r#"{% macro card(slug) %}
<div class="card">{{ slug }}</div>
{% endmacro card %}"#;
    let (body, _) = ShortcodeProcessor::extract_macro_body(input, "card").unwrap();
    assert!(body.contains(r#"<div class="card">{{ slug }}</div>"#));
}

#[test]
fn test_extract_macro_body_generic_endmacro() {
    let input = r#"{% macro simple() %}
<p>hello</p>
{% endmacro %}"#;
    let (body, params) = ShortcodeProcessor::extract_macro_body(input, "simple").unwrap();
    assert!(body.contains("<p>hello</p>"));
    assert!(params.is_empty());
}

#[test]
fn test_extract_macro_body_multiple_defaults() {
    let input = r#"{% macro gallery(path, ord="", width="100%", height="100%") %}
<div>gallery</div>
{% endmacro gallery %}"#;
    let (body, params) = ShortcodeProcessor::extract_macro_body(input, "gallery").unwrap();
    assert!(body.contains("<div>gallery</div>"));
    assert_eq!(params.len(), 4);
    assert_eq!(params[0].name, "path");
    assert!(params[0].default.is_none());
    assert_eq!(params[1].name, "ord");
    assert_eq!(params[1].default.as_deref(), Some(""));
    assert_eq!(params[2].name, "width");
    assert_eq!(params[2].default.as_deref(), Some("100%"));
    assert_eq!(params[3].name, "height");
    assert_eq!(params[3].default.as_deref(), Some("100%"));
}

#[test]
fn test_extract_macro_body_wrong_name_returns_none() {
    let input = r#"{% macro youtube(id) %}
<iframe></iframe>
{% endmacro youtube %}"#;
    assert!(ShortcodeProcessor::extract_macro_body(input, "gallery").is_none());
}

#[test]
fn test_extract_macro_body_real_youtube() {
    let input = r#"{# Embed a YouTube video #}
{% macro youtube(id, width="560", height="315") %}
{% if id is not starting_with("http") %}
{% set id = "https://www.youtube.com/embed/" ~ id %}
{% endif %}
<p><iframe width="{{width}}" height="{{height}}" src="{{id}}"></iframe></p>
{% endmacro youtube %}"#;
    let (body, params) = ShortcodeProcessor::extract_macro_body(input, "youtube").unwrap();
    assert!(body.contains("<p><iframe"));
    assert!(body.contains("starting_with"));
    assert_eq!(params.len(), 3);
    assert_eq!(params[0].name, "id");
    assert_eq!(params[1].default.as_deref(), Some("560"));
    assert_eq!(params[2].default.as_deref(), Some("315"));
}

// --- shortcode rendering with context access ---

#[test]
fn test_shortcode_body_can_access_context_variables() {
    let input = r#"{% macro greet(name) %}
Hello {{ name }}, welcome to {{ site_title }}!
{% endmacro greet %}"#;

    let (body, params) = ShortcodeProcessor::extract_macro_body(input, "greet").unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "name");

    let mut tera = tera::Tera::default();
    let mut context = tera::Context::new();
    context.insert("site_title", "My Site");
    context.insert("name", "World");

    let rendered = tera.render_str(&body, &context, false).unwrap();
    assert!(rendered.contains("Hello World"));
    assert!(rendered.contains("welcome to My Site"));
}
