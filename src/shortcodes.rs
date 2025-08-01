use log::{debug, warn};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

#[derive(Debug, Clone, Serialize)]
pub struct Shortcode {
    pub name: String,
    pub content: String,
    pub is_html: bool,
    pub description: Option<String>,
}

pub struct ShortcodeProcessor {
    pub shortcodes: HashMap<String, Shortcode>,
    pub pattern: Regex,
}

impl ShortcodeProcessor {
    pub fn new(pattern: Option<&str>) -> Self {
        let default_pattern = r"<!-- \.(\w+)(\s+[^>]+)?\s*-->";
        let pattern =
            Regex::new(pattern.unwrap_or(default_pattern)).expect("Invalid shortcode pattern");

        Self {
            shortcodes: HashMap::new(),
            pattern,
        }
    }

    /// Add shortcodes to Tera instance
    pub fn add_shortcodes_to_tera(&self, tera: &mut Tera) -> Result<(), String> {
        for (name, shortcode) in &self.shortcodes {
            if shortcode.is_html {
                tera.add_raw_template(&format!("shortcodes/{name}"), &shortcode.content)
                    .map_err(|e| format!("Failed to add shortcode template '{name}': {e}"))?;
            }
        }
        Ok(())
    }

    /// Collect shortcodes from the `input_dir/shortcodes` directory
    pub fn collect_shortcodes(&mut self, input_dir: &Path) -> Result<(), String> {
        let shortcodes_dir = input_dir.join("shortcodes");

        if !shortcodes_dir.exists() {
            debug!(
                "No shortcodes directory found at {}",
                shortcodes_dir.display()
            );
            return Ok(());
        }

        // Add builtin shortcodes first
        self.add_builtin_shortcodes();

        // Then add user shortcodes (which can override builtins)
        let entries = fs::read_dir(&shortcodes_dir)
            .map_err(|e| format!("Failed to read shortcodes directory: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "html" || extension == "md" {
                        self.load_shortcode(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn load_shortcode(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read shortcode file {}: {e}", path.display()))?;

        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid file name: {}", path.display()))?;

        let is_html = path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext == "html");

        // Extract description from Tera comment on first line
        let description = self.extract_description(&content);

        if is_html {
            // For HTML files, validate that they contain macros
            if !content.contains("{% macro") {
                return Err(format!(
                    "HTML shortcode file {} must contain at least one macro",
                    path.display()
                ));
            }

            // Validate that the file contains a macro with the same name as the filename
            let macro_pattern =
                Regex::new(r"\{%\s*macro\s+(\w+)\s*\(").expect("Invalid macro pattern");
            let macro_names: Vec<String> = macro_pattern
                .captures_iter(&content)
                .map(|cap| cap[1].to_string())
                .collect();

            if !macro_names.contains(&file_name.to_string()) {
                return Err(format!(
                    "HTML shortcode file {} must contain a macro named '{}'. Found macros: {:?}",
                    path.display(),
                    file_name,
                    macro_names
                ));
            }
        }

        let shortcode = Shortcode {
            name: file_name.to_string(),
            content: content.clone(),
            is_html,
            description,
        };

        debug!("Loaded shortcode: {file_name}");
        self.shortcodes.insert(file_name.to_string(), shortcode);

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn extract_description(&self, content: &str) -> Option<String> {
        // Check if the first line is a Tera comment {# ... #}
        let first_line = content.lines().next()?;
        let trimmed = first_line.trim();

        if trimmed.starts_with("{#") && trimmed.ends_with("#}") {
            // Extract content between {# and #}
            let desc = trimmed
                .strip_prefix("{#")?
                .strip_suffix("#}")?
                .trim()
                .to_string();

            if !desc.is_empty() {
                return Some(desc);
            }
        }

        None
    }

    fn add_builtin_shortcodes(&mut self) {
        // TOC shortcode
        let toc_macro = r#"{% macro toc() %}
<nav class="table-of-contents">
<div class="content-toc">
{{ content.toc | safe }}
</div>
</nav>
{% endmacro toc %}"#;

        self.shortcodes.insert(
            "toc".to_string(),
            Shortcode {
                name: "toc".to_string(),
                content: toc_macro.to_string(),
                is_html: true,
                description: Some("Display table of contents for the current content".to_string()),
            },
        );

        // YouTube shortcode
        let youtube_macro = r#"{% macro youtube(id, width="560", height="315") %}
{% if id is not starting_with("http") %}
{% set id = "https://www.youtube.com/embed/" ~ id %}
{% endif %}
<p><iframe width="{{width}}" height="{{height}}" src="{{id}}" frameBorder="0" allow="accelerometer;" allowFullScreen></iframe></p>
{% endmacro youtube %}"#;

        self.shortcodes.insert(
            "youtube".to_string(),
            Shortcode {
                name: "youtube".to_string(),
                content: youtube_macro.to_string(),
                is_html: true,
                description: Some(
                    "Embed YouTube videos. Params: id, width=560, height=315".to_string(),
                ),
            },
        );

        // Authors shortcode
        let authors_macro = r#"{% macro authors() %}
<ul class="authors-list">
{% for author_name, posts in site_data.author.map %}
<li><a href="/author-{{ author_name | slugify }}.html">{{ author_name }}</a> ({{ posts | length }} posts)</li>
{% endfor %}
</ul>
{% endmacro authors %}"#;

        self.shortcodes.insert(
            "authors".to_string(),
            Shortcode {
                name: "authors".to_string(),
                content: authors_macro.to_string(),
                is_html: true,
                description: Some(
                    "Display a list of all authors with post counts. Params: ord=asc, items=0"
                        .to_string(),
                ),
            },
        );

        // Streams shortcode
        let streams_macro = r#"{% macro streams(ord="asc", items=0) %}
<ul class="streams-list">
{% for stream_name, posts in site_data.stream.map %}
<li><a href="/{{ stream_name }}.html">{{ stream_name }}</a> ({{ posts | length }} posts)</li>
{% endfor %}
</ul>
{% endmacro streams %}"#;

        self.shortcodes.insert(
            "streams".to_string(),
            Shortcode {
                name: "streams".to_string(),
                content: streams_macro.to_string(),
                is_html: true,
                description: Some(
                    "Display a list of all content streams with post counts. Params: ord=asc, items=0".to_string(),
                ),
            },
        );
    }

    /// Process shortcodes in HTML content
    pub fn process_shortcodes(&self, html: &str, context: &Context, tera: &Tera) -> String {
        let mut result = html.to_string();

        for captures in self.pattern.captures_iter(html) {
            let full_match = &captures[0];
            let shortcode_name = &captures[1];
            let params = captures.get(2).map_or("", |m| m.as_str().trim());

            match self.render_shortcode(shortcode_name, params, context, tera) {
                Ok(rendered) => {
                    result = result.replace(full_match, &rendered);
                }
                Err(e) => {
                    warn!("Shortcode '{shortcode_name}' not found or failed to render: {e}");
                }
            }
        }

        result
    }

    fn render_shortcode(
        &self,
        name: &str,
        params: &str,
        context: &Context,
        tera: &Tera,
    ) -> Result<String, String> {
        let shortcode = self
            .shortcodes
            .get(name)
            .ok_or_else(|| format!("Shortcode '{name}' not found"))?;

        if shortcode.is_html {
            // Parse parameters into macro arguments
            let macro_args = if params.is_empty() {
                String::new()
            } else {
                // Parse key=value pairs
                let mut args = Vec::new();
                for param in params.split_whitespace() {
                    if let Some((key, value)) = param.split_once('=') {
                        // Quote the value if it's not already quoted
                        let quoted_value = if value.starts_with('"') && value.ends_with('"') {
                            value.to_string()
                        } else {
                            format!("\"{value}\"")
                        };
                        args.push(format!("{key}={quoted_value}"));
                    }
                }
                args.join(", ")
            };

            // Render HTML shortcode using Tera macro
            let shortcode_template = format!(
                "{{% import \"shortcodes/{name}\" as sc -%}}\n{{{{ sc::{name}({macro_args}) }}}}"
            );

            debug!("Rendering shortcode '{name}' with template: {shortcode_template}");
            debug!("Shortcode params: '{params}' -> macro_args: '{macro_args}'");

            let mut tera_clone = tera.clone();
            tera_clone
                .render_str(&shortcode_template, context)
                .map_err(|e| format!("Failed to render shortcode '{name}': {e}"))
        } else {
            // Render markdown shortcode
            let mut tera_clone = tera.clone();
            let rendered = tera_clone
                .render_str(&shortcode.content, context)
                .map_err(|e| format!("Failed to render markdown shortcode '{name}': {e}"))?;

            // Convert markdown to HTML
            let default_parser_options = crate::config::ParserOptions::default();
            Ok(crate::parser::get_html_with_options(
                &rendered,
                &default_parser_options,
            ))
        }
    }

    /// Get list of available shortcodes with descriptions
    pub fn list_shortcodes_with_descriptions(&self) -> Vec<(&str, Option<&str>)> {
        let mut shortcodes: Vec<(&str, Option<&str>)> = self
            .shortcodes
            .iter()
            .map(|(name, sc)| (name.as_str(), sc.description.as_deref()))
            .collect();
        shortcodes.sort_by_key(|(name, _)| *name);
        shortcodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shortcode_pattern() {
        let processor = ShortcodeProcessor::new(None);
        let html = r#"<p>Some text</p>
<!-- .youtube id=abc123 -->
<p>More text</p>
<!-- .toc -->
<!-- .authors -->"#;

        let matches: Vec<_> = processor.pattern.captures_iter(html).collect();
        assert_eq!(matches.len(), 3);
        assert_eq!(&matches[0][1], "youtube");
        assert_eq!(&matches[1][1], "toc");
        assert_eq!(&matches[2][1], "authors");
    }

    #[test]
    fn test_builtin_shortcodes() {
        let mut processor = ShortcodeProcessor::new(None);
        processor.add_builtin_shortcodes();

        assert!(processor.shortcodes.contains_key("toc"));
        assert!(processor.shortcodes.contains_key("youtube"));
        assert!(processor.shortcodes.contains_key("authors"));
        assert!(processor.shortcodes.contains_key("streams"));
    }

    #[test]
    fn test_load_shortcode_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let shortcodes_dir = temp_dir.path().join("shortcodes");
        fs::create_dir(&shortcodes_dir).unwrap();

        // Create a test HTML shortcode
        let test_html = r#"{% macro test() %}
<div>Test shortcode</div>
{% endmacro test %}"#;
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
        let wrong_macro = r#"{% macro bar() %}
<div>Wrong macro name</div>
{% endmacro bar %}"#;
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
        let multi_macro = r#"{% macro helper() %}
<span>Helper</span>
{% endmacro helper %}

{% macro multi() %}
<div>Correct macro</div>
{% endmacro multi %}"#;
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
        let md_with_desc =
            "{# List of recent posts #}\n## Recent Posts\n\nThis is markdown content.";
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

        // Check that we have the expected builtin shortcodes
        assert_eq!(shortcodes.len(), 4);

        // Check they're sorted alphabetically
        let names: Vec<&str> = shortcodes.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, vec!["authors", "streams", "toc", "youtube"]);

        // Check descriptions are present
        for (name, desc) in shortcodes {
            assert!(desc.is_some(), "Shortcode {name} should have a description");
        }
    }
}
