use crate::embedded::EMBEDDED_SHORTCODES;
use crate::re;
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
        let pattern = Regex::new(pattern.unwrap_or(re::SHORTCODE_HTML_COMMENT))
            .expect("Invalid shortcode pattern");

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

        // Add builtin shortcodes first (always load them)
        self.add_builtin_shortcodes();

        if !shortcodes_dir.exists() {
            debug!(
                "No shortcodes directory found at {}",
                shortcodes_dir.display()
            );
            return Ok(());
        }

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
                Regex::new(re::CAPTURE_TERA_MACRO_CALL).expect("Invalid macro pattern");
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
        // Load shortcodes from embedded files
        for (file_name, file_data) in EMBEDDED_SHORTCODES.iter() {
            let content = match std::str::from_utf8(file_data) {
                Ok(content) => content,
                Err(e) => {
                    warn!("Failed to read embedded shortcode '{file_name}': {e}");
                    continue;
                }
            };

            // Extract filename without extension
            let lowercase_name = file_name.to_lowercase();
            let shortcode_name = if let Some(name) = lowercase_name.strip_suffix(".html") {
                name
            } else if let Some(name) = lowercase_name.strip_suffix(".md") {
                name
            } else {
                warn!("Embedded shortcode '{file_name}' does not have .html or .md extension");
                continue;
            };

            // Determine if it's HTML or markdown
            let is_html = file_name.to_lowercase().ends_with(".html");

            // Extract description from content
            let description = self.extract_description(content);

            let shortcode = Shortcode {
                name: shortcode_name.to_string(),
                content: content.to_string(),
                is_html,
                description,
            };

            debug!("Loaded embedded shortcode: {shortcode_name}");
            self.shortcodes
                .insert(shortcode_name.to_string(), shortcode);
        }
    }

    /// Process shortcodes in HTML content
    pub fn process_shortcodes(&self, html: &str, context: &Context, tera: &Tera) -> String {
        let mut result = html.to_string();

        debug!(
            "Searching for shortcode pattern in HTML (first 1500 chars): {}",
            &html[..html.len().min(1500)]
        );

        for captures in self.pattern.captures_iter(html) {
            let full_match = &captures[0];
            let shortcode_name = &captures[1];
            let params = captures.get(2).map_or("", |m| m.as_str().trim());

            log::info!(
                "Processing shortcode: name='{shortcode_name}', params='{params}', full_match='{full_match}'"
            );

            match self.render_shortcode(shortcode_name, params, context, tera) {
                Ok(rendered) => {
                    debug!("Successfully rendered shortcode '{shortcode_name}': '{rendered}'");
                    result = result.replace(full_match, &rendered);
                }
                Err(e) => {
                    warn!("Shortcode '{shortcode_name}' failed to render: {e}");
                    // Render an error message in the HTML output
                    let escaped_error = e
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;")
                        .replace('\'', "&#39;");
                    let error_msg = format!(
                        r#"<div class="shortcode-error" style="border: 2px solid red; padding: 10px; margin: 10px 0; background-color: #ffeeee; color: #cc0000;">
                        <strong>Shortcode Error:</strong> {escaped_error}</div>"#
                    );
                    result = result.replace(full_match, &error_msg);
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
                // Parse key=value pairs with proper quote handling
                let mut args = Vec::new();
                let parsed_params = Self::parse_parameters(params);
                for (key, value) in parsed_params {
                    // Ensure value is properly quoted for Tera
                    let quoted_value = if value.starts_with('"') && value.ends_with('"') {
                        value
                    } else if value.starts_with('\'') && value.ends_with('\'') {
                        // Convert single quotes to double quotes for Tera
                        format!("\"{}\"", &value[1..value.len() - 1])
                    } else {
                        format!("\"{value}\"")
                    };
                    args.push(format!("{key}={quoted_value}"));
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

    /// Parse parameters from a string, handling quoted values correctly
    fn parse_parameters(params: &str) -> Vec<(String, String)> {
        let mut result = Vec::new();
        let mut chars = params.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_whitespace() {
                continue;
            }

            // Parse key
            let mut key = String::new();
            let mut current_char = ch;

            loop {
                if current_char == '=' {
                    break;
                } else if current_char.is_whitespace() {
                    // Skip whitespace before =
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_whitespace() {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if let Some(&'=') = chars.peek() {
                        chars.next(); // consume the =
                        break;
                    }
                    // No = found, this is not a valid key=value pair
                    break;
                }
                key.push(current_char);

                if let Some(next_ch) = chars.next() {
                    current_char = next_ch;
                } else {
                    break;
                }
            }

            if key.is_empty() {
                continue;
            }

            // Skip whitespace after =
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            // Parse value
            let mut value = String::new();

            if let Some(&quote_char) = chars.peek() {
                if quote_char == '"' || quote_char == '\'' {
                    // Handle quoted value
                    chars.next(); // consume opening quote
                    value.push(quote_char);

                    for ch in chars.by_ref() {
                        value.push(ch);
                        if ch == quote_char {
                            // Check if it's escaped
                            let mut backslash_count = 0;
                            let temp_chars: Vec<char> = value.chars().rev().skip(1).collect();
                            for &c in &temp_chars {
                                if c == '\\' {
                                    backslash_count += 1;
                                } else {
                                    break;
                                }
                            }

                            // If even number of backslashes (including 0), quote is not escaped
                            if backslash_count % 2 == 0 {
                                break;
                            }
                        }
                    }
                } else {
                    // Handle unquoted value
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_whitespace() {
                            break;
                        }
                        if let Some(ch) = chars.next() {
                            value.push(ch);
                        } else {
                            // This should not happen since we peeked successfully, but be safe
                            break;
                        }
                    }
                }
            }

            if !value.is_empty() {
                result.push((key, value));
            }
        }

        result
    }
}

#[cfg(test)]
#[path = "tests/shortcodes.rs"]
mod tests;
