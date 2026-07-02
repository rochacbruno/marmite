use crate::embedded::{preprocess_template, EMBEDDED_SHORTCODES};
use crate::highlight::MarmiteHighlighter;
use crate::re;
use log::{debug, warn};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

#[derive(Debug, Clone)]
pub struct MacroParam {
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Shortcode {
    pub name: String,
    pub content: String,
    pub is_html: bool,
    pub description: Option<String>,
    #[serde(skip)]
    pub body: Option<String>,
    #[serde(skip)]
    pub params: Vec<MacroParam>,
}

fn insert_typed(context: &mut Context, key: String, value: &str) {
    if let Ok(n) = value.parse::<i64>() {
        context.insert(key, &n);
    } else if let Ok(f) = value.parse::<f64>() {
        context.insert(key, &f);
    } else if value == "true" {
        context.insert(key, &true);
    } else if value == "false" {
        context.insert(key, &false);
    } else {
        context.insert(key, value);
    }
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

    /// Extract the body and parameter list from a shortcode definition.
    /// Supports both `{% shortcode name() %}...{% endshortcode %}` (recommended)
    /// and `{% macro name() %}...{% endmacro %}` (backward compatible).
    fn extract_shortcode_body(
        content: &str,
        shortcode_name: &str,
    ) -> Option<(String, Vec<MacroParam>)> {
        let header_re = Regex::new(&format!(
            r"\{{% *(?:macro|shortcode) +{shortcode_name}\s*\(([^)]*)\)\s*%\}}"
        ))
        .ok()?;

        let header_match = header_re.captures(content)?;
        let params_str = &header_match[1];
        let header_end = header_match.get(0)?.end();

        let end_re = Regex::new(&format!(
            r"\{{% *(?:endmacro|endshortcode)(?: +{shortcode_name})?\s*%\}}"
        ))
        .ok()?;
        let end_match = end_re.find(&content[header_end..])?;
        let body = content[header_end..header_end + end_match.start()].to_string();

        let mut params = Vec::new();
        if !params_str.trim().is_empty() {
            for param in params_str.split(',') {
                let param = param.trim();
                if let Some((name, default)) = param.split_once('=') {
                    let default = default.trim().trim_matches('"').trim_matches('\'');
                    params.push(MacroParam {
                        name: name.trim().to_string(),
                        default: Some(default.to_string()),
                    });
                } else {
                    params.push(MacroParam {
                        name: param.to_string(),
                        default: None,
                    });
                }
            }
        }

        Some((body, params))
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

        let mut body = None;
        let mut params = Vec::new();

        if is_html {
            if !content.contains("{% macro") && !content.contains("{% shortcode") {
                return Err(format!(
                    "HTML shortcode file {} must contain a shortcode or macro definition",
                    path.display()
                ));
            }

            let def_pattern =
                Regex::new(re::CAPTURE_SHORTCODE_DEF).expect("Invalid shortcode pattern");
            let def_names: Vec<String> = def_pattern
                .captures_iter(&content)
                .map(|cap| cap[1].to_string())
                .collect();

            if !def_names.contains(&file_name.to_string()) {
                return Err(format!(
                    "HTML shortcode file {} must contain a definition named '{}'. Found: {:?}",
                    path.display(),
                    file_name,
                    def_names
                ));
            }

            if let Some((b, p)) = Self::extract_shortcode_body(&content, file_name) {
                body = Some(b);
                params = p;
            }
        }

        let shortcode = Shortcode {
            name: file_name.to_string(),
            content: content.clone(),
            is_html,
            description,
            body,
            params,
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

            let mut body = None;
            let mut params = Vec::new();
            if is_html {
                if let Some((b, p)) = Self::extract_shortcode_body(content, shortcode_name) {
                    body = Some(b);
                    params = p;
                }
            }

            let shortcode = Shortcode {
                name: shortcode_name.to_string(),
                content: content.to_string(),
                is_html,
                description,
                body,
                params,
            };

            debug!("Loaded embedded shortcode: {shortcode_name}");
            self.shortcodes
                .insert(shortcode_name.to_string(), shortcode);
        }
    }

    /// Process shortcodes in HTML content
    pub fn process_shortcodes(
        &self,
        html: &str,
        context: &Context,
        tera: &Tera,
        highlighter: Option<&MarmiteHighlighter>,
    ) -> String {
        let mut result = html.to_string();

        let preview_end = {
            let mut end = html.len().min(1500);
            while end > 0 && !html.is_char_boundary(end) {
                end -= 1;
            }
            end
        };
        debug!(
            "Searching for shortcode pattern in HTML (first 1500 chars): {}",
            &html[..preview_end]
        );

        for captures in self.pattern.captures_iter(html) {
            let full_match = &captures[0];
            let shortcode_name = &captures[1];
            let params = captures.get(2).map_or("", |m| m.as_str().trim());

            log::info!(
                "Processing shortcode: name='{shortcode_name}', params='{params}', full_match='{full_match}'"
            );

            match self.render_shortcode(shortcode_name, params, context, tera, highlighter) {
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
        highlighter: Option<&MarmiteHighlighter>,
    ) -> Result<String, String> {
        let shortcode = self
            .shortcodes
            .get(name)
            .ok_or_else(|| format!("Shortcode '{name}' not found"))?;

        if shortcode.is_html {
            let body = shortcode
                .body
                .as_ref()
                .ok_or_else(|| format!("Shortcode '{name}' has no extracted body"))?;

            // Build context: clone caller's context, then inject shortcode parameters
            let mut sc_context = context.clone();
            let parsed_params = if params.is_empty() {
                Vec::new()
            } else {
                Self::parse_parameters(params)
            };
            let caller_params: HashMap<String, String> = parsed_params
                .into_iter()
                .map(|(k, v)| {
                    let unquoted = v.trim_matches('"').trim_matches('\'').to_string();
                    (k, unquoted)
                })
                .collect();

            let param_defs: Vec<(String, Option<String>)> = shortcode
                .params
                .iter()
                .map(|p| (p.name.clone(), p.default.clone()))
                .collect();
            for (param_name, param_default) in param_defs {
                if let Some(value) = caller_params.get(&param_name) {
                    insert_typed(&mut sc_context, param_name, value);
                } else if let Some(default) = param_default {
                    insert_typed(&mut sc_context, param_name, &default);
                }
            }

            let preprocessed = preprocess_template(body);

            debug!("Rendering shortcode '{name}' body with context");

            tera.render_str(&preprocessed, &sc_context, false)
                .map_err(|e| format!("Failed to render shortcode '{name}': {e}"))
        } else {
            // Render markdown shortcode
            let rendered = tera
                .render_str(&shortcode.content, context, false)
                .map_err(|e| format!("Failed to render markdown shortcode '{name}': {e}"))?;

            // Convert markdown to HTML
            let default_parser_options = crate::config::ParserOptions::default();
            Ok(crate::parser::get_html_with_options(
                &rendered,
                &default_parser_options,
                highlighter,
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
