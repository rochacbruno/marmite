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

        if is_html {
            // For HTML files, validate that they contain macros
            if !content.contains("{% macro") {
                return Err(format!(
                    "HTML shortcode file {} must contain at least one macro",
                    path.display()
                ));
            }
        }

        let shortcode = Shortcode {
            name: file_name.to_string(),
            content: content.clone(),
            is_html,
        };

        debug!("Loaded shortcode: {file_name}");
        self.shortcodes.insert(file_name.to_string(), shortcode);

        Ok(())
    }

    fn add_builtin_shortcodes(&mut self) {
        // TOC shortcode
        let toc_macro = r#"{% macro toc() %}
<nav class="table-of-contents">
{{ content.toc | safe }}
</nav>
{% endmacro toc %}"#;

        self.shortcodes.insert(
            "toc".to_string(),
            Shortcode {
                name: "toc".to_string(),
                content: toc_macro.to_string(),
                is_html: true,
            },
        );

        // YouTube shortcode
        let youtube_macro = r#"{% macro youtube(id, width="560", height="315") %}
{% if id is not starting_with("http") %}
{% set id = "https://www.youtube.com/embed/" ~ id %}
{% endif %}
<p><iframe width="{{width}}" height="{{height}}" src="{{id}}" title="" frameBorder="0" allow="accelerometer;" allowFullScreen></iframe></p>
{% endmacro youtube %}"#;

        self.shortcodes.insert(
            "youtube".to_string(),
            Shortcode {
                name: "youtube".to_string(),
                content: youtube_macro.to_string(),
                is_html: true,
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

    /// Get list of available shortcodes
    pub fn list_shortcodes(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .shortcodes
            .keys()
            .map(std::string::String::as_str)
            .collect();
        names.sort_unstable();
        names
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
}
