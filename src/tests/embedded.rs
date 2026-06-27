use super::*;
use std::io::Read;
use tempfile::TempDir;

#[test]
fn test_write_bytes_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    let test_data = b"Hello, World!";

    let result = write_bytes_to_file(&file_path, test_data);
    assert!(result.is_ok());

    let mut file = File::open(&file_path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    assert_eq!(contents, test_data);
}

#[test]
fn test_generate_static() {
    let temp_dir = TempDir::new().unwrap();
    let static_folder = temp_dir.path().join("static");

    generate_static(&static_folder);

    assert!(static_folder.exists());
}

#[test]
fn test_embedded_templates_exist() {
    let template_names: Vec<_> = Templates::iter().collect();
    assert!(!template_names.is_empty());
    for name in &template_names {
        let template = Templates::get(name.as_ref());
        assert!(template.is_some(), "Template {name} should exist");
    }
}

#[test]
fn test_embedded_static_initialization() {
    let static_files = &*EMBEDDED_STATIC;
    assert!(!static_files.is_empty());
}

#[test]
fn test_embedded_agent_skills_initialization() {
    let skill_files = &*EMBEDDED_AGENT_SKILLS;
    assert!(!skill_files.is_empty());
}

#[test]
fn test_get_skill_content() {
    let content = get_skill_content();
    assert!(content.is_some());
    let content = content.unwrap();
    assert!(content.contains("marmite"));
}

#[test]
fn test_install_skills_to_agents() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path();

    install_skills_to_agents(target);

    assert!(target.join(".agents/skills/marmite/SKILL.md").exists());
}

#[test]
fn test_install_skills_to_claude() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path();

    install_skills_to_claude(target);

    assert!(target.join(".claude/skills/marmite/SKILL.md").exists());
}

// --- strip_ignore_missing (template preprocessing) tests ---

#[test]
fn test_strip_ignore_missing_basic() {
    let input = r#"{% include "comments.html" ignore missing %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{% include "comments.html" %}"#);
}

#[test]
fn test_strip_ignore_missing_single_quotes() {
    let input = r#"{%include 'base_feeds.html' ignore missing%}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{%include 'base_feeds.html' %}"#);
}

#[test]
fn test_strip_ignore_missing_with_whitespace_trimming() {
    let input = r#"{%- include "foo.html" ignore missing -%}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{%- include "foo.html" -%}"#);
}

#[test]
fn test_strip_ignore_missing_preserves_normal_includes() {
    let input = r#"{% include "base.html" %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, input);
}

#[test]
fn test_strip_ignore_missing_multiple() {
    let input = r#"{% include "a.html" ignore missing %}
{% include "b.html" %}
{% include "c.html" ignore missing %}"#;
    let result = strip_ignore_missing(input);
    assert!(result.contains(r#"{% include "a.html" %}"#));
    assert!(result.contains(r#"{% include "b.html" %}"#));
    assert!(result.contains(r#"{% include "c.html" %}"#));
    assert!(!result.contains("ignore missing"));
}

#[test]
fn test_dot_index_to_bracket() {
    let input = r#"{% set name = item.0 %}
{% set url = item.1 %}"#;
    let result = strip_ignore_missing(input);
    assert!(result.contains("item[0]"));
    assert!(result.contains("item[1]"));
    assert!(!result.contains("item.0"));
    assert!(!result.contains("item.1"));
}

#[test]
fn test_dot_index_preserves_named_fields() {
    let input = r#"{{ content.title }}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, input);
}

#[test]
fn test_dot_index_nested_with_named_and_numeric() {
    let input = r#"{{ content.authors.0 }}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{{ content.authors[0] }}"#);
}

#[test]
fn test_starting_with_positional_to_keyword() {
    let input = r#"{% if url is starting_with("http") %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{% if url is starting_with(pat="http") %}"#);
}

#[test]
fn test_not_starting_with_positional_to_keyword() {
    let input = r#"{% if content.html is not starting_with("<h1>") %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(
        result,
        r#"{% if content.html is not starting_with(pat="<h1>") %}"#
    );
}

#[test]
fn test_ending_with_positional_to_keyword() {
    let input = r#"{% if name is ending_with(".html") %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{% if name is ending_with(pat=".html") %}"#);
}

#[test]
fn test_containing_positional_to_keyword() {
    let input = r#"{% if text is containing("hello") %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{% if text is containing(pat="hello") %}"#);
}

#[test]
fn test_starting_with_single_quotes() {
    let input = r#"{% if url is starting_with('http') %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{% if url is starting_with(pat='http') %}"#);
}

#[test]
fn test_starting_with_already_keyword_unchanged() {
    let input = r#"{% if url is starting_with(pat="http") %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, input);
}

#[test]
fn test_deep_defined_check_optional_chaining() {
    let input = r#"{% if site.extra.comments.source is defined %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(
        result,
        r#"{% if site?.extra?.comments?.source is defined %}"#
    );
}

#[test]
fn test_deep_not_defined_check_optional_chaining() {
    let input = r#"{% if site.extra.comments is not defined %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{% if site?.extra?.comments is not defined %}"#);
}

#[test]
fn test_shallow_defined_check_unchanged() {
    let input = r#"{% if comments is defined %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, input);
}

#[test]
fn test_two_level_defined_check_unchanged() {
    let input = r#"{% if site.url is defined %}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, input);
}

#[test]
fn test_default_filter_optional_chaining() {
    let input = r#"{{author.name | default(value=username)}}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, r#"{{author?.name | default(value=username)}}"#);
}

#[test]
fn test_default_filter_with_whitespace_trim() {
    let input = r#"{{- author.name | default(value="unknown") }}"#;
    let result = strip_ignore_missing(input);
    assert!(result.contains("author?.name"));
}

#[test]
fn test_non_default_filter_unchanged() {
    let input = r#"{{ content.title | upper }}"#;
    let result = strip_ignore_missing(input);
    assert_eq!(result, input);
}

#[test]
fn test_collect_ignore_missing_includes() {
    let input = r#"{% include "a.html" ignore missing %}
{% include "b.html" %}
{%include 'c.html' ignore missing%}"#;
    let result = collect_ignore_missing_includes(input);
    assert_eq!(result, vec!["a.html", "c.html"]);
}

#[test]
fn test_collect_ignore_missing_includes_none() {
    let input = r#"{% include "a.html" %}"#;
    let result = collect_ignore_missing_includes(input);
    assert!(result.is_empty());
}

#[test]
fn test_all_transformations_combined() {
    let input = r#"{% include "comments.html" ignore missing %}
{% set name = item.0 %}
{% if url is starting_with("http") %}
{% if site.extra.comments.source is defined %}
{{author.name | default(value=username)}}"#;
    let result = strip_ignore_missing(input);
    assert!(result.contains(r#"{% include "comments.html" %}"#));
    assert!(result.contains("item[0]"));
    assert!(result.contains(r#"starting_with(pat="http")"#));
    assert!(result.contains("site?.extra?.comments?.source is defined"));
    assert!(result.contains("author?.name | default"));
}
