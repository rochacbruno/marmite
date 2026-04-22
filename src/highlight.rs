use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Write as _};
use std::sync::{Arc, Mutex};

use arborium::theme::{builtin as builtin_themes, Theme};
use arborium::{Config, Error as ArboriumError, Highlighter, HtmlFormat};
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html;
use log::warn;

use crate::config::CodeHighlightConfig;

const CLIENT_RENDERED_LANGS: &[&str] = &["mermaid"];

pub struct MarmiteHighlighter {
    inner: Mutex<Highlighter>,
}

impl MarmiteHighlighter {
    fn new() -> Self {
        let config = Config {
            // Handle reasonable levels of nested content
            // e.g. code blocks in markdown, JS in HTML, etc.).
            max_injection_depth: 3,
            // Emit compact, custom HTML elements.
            html_format: HtmlFormat::CustomElements,
        };
        Self {
            inner: Mutex::new(Highlighter::with_config(config)),
        }
    }
}

impl SyntaxHighlighterAdapter for MarmiteHighlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let Some(raw_lang) = lang.map(str::trim).filter(|s| !s.is_empty()) else {
            return html::escape(output, code);
        };
        // Fence info strings can carry metadata after the language name
        // (e.g. ```rust main.rs); arborium wants just the language token.
        let lang = raw_lang.split_whitespace().next().unwrap_or(raw_lang);

        // Some fence "languages" are rendered client-side
        // (e.g. mermaid via MermaidJS) and intentionally bypass syntax highlighting.
        // Arborium should leave these alone.
        // comrak still adds `language-<lang>` to the <code> tag,
        // which is what the client-side renderer looks for.
        if CLIENT_RENDERED_LANGS.contains(&lang) {
            return html::escape(output, code);
        }

        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(poison) => {
                warn!("arborium highlighter mutex poisoned: {poison}; emitting plain code");
                return html::escape(output, code);
            }
        };
        match guard.highlight(lang, code) {
            Ok(highlighted) => output.write_str(&highlighted),
            Err(ArboriumError::UnsupportedLanguage { .. }) => {
                warn!("unsupported code fence language '{lang}' (want to contribute? https://github.com/bearcove/arborium/blob/main/ADDING_GRAMMARS.md); emitting plain code");
                html::escape(output, code)
            }
            Err(other) => {
                warn!("arborium failed to highlight '{lang}': {other}; emitting plain code");
                html::escape(output, code)
            }
        }
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        mut attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        attributes
            .entry("class")
            .and_modify(|v| *v = Cow::Owned(format!("marmite-code {v}")))
            .or_insert(Cow::Borrowed("marmite-code"));
        html::write_opening_tag(output, "pre", attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        mut attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        attributes
            .entry("class")
            .and_modify(|v| *v = Cow::Owned(format!("marmite-code-inner {v}")))
            .or_insert(Cow::Borrowed("marmite-code-inner"));
        html::write_opening_tag(output, "code", attributes)
    }
}

fn normalize(name: &str) -> String {
    slug::slugify(name)
}

fn theme_by_name(name: &str) -> Option<Theme> {
    let target = normalize(name);
    builtin_themes::all()
        .into_iter()
        .find(|t| normalize(&t.name) == target)
}

fn available_themes() -> Vec<String> {
    let mut names: Vec<String> = builtin_themes::all()
        .into_iter()
        .map(|t| normalize(&t.name))
        .collect();
    names.sort();
    names
}

pub fn build(config: &CodeHighlightConfig) -> Result<Arc<MarmiteHighlighter>, String> {
    ensure_theme(&config.light_theme, "light")?;
    ensure_theme(&config.dark_theme, "dark")?;
    Ok(Arc::new(MarmiteHighlighter::new()))
}

fn ensure_theme(name: &str, slot: &str) -> Result<(), String> {
    if theme_by_name(name).is_some() {
        Ok(())
    } else {
        Err(format!(
            "unknown code_highlight {slot}_theme '{name}'. Available themes: {}",
            available_themes().join(", ")
        ))
    }
}

/// Produce a stylesheet that pairs the light and dark themes with Marmite's
/// existing `data-theme` toggle on the `<html>` element (set by marmite.js),
/// falling back to `prefers-color-scheme` before JS runs.
///
/// `arborium::theme::Theme::to_css` wraps its output in a `{prefix} { ... }`
/// block and uses CSS nesting for the custom-element rules, so the prefix must
/// be a valid selector — an empty prefix yields an anonymous block browsers
/// ignore.
pub fn generate_css(config: &CodeHighlightConfig) -> Result<String, String> {
    let light =
        theme_by_name(&config.light_theme).ok_or_else(|| unknown_theme(&config.light_theme))?;
    let dark =
        theme_by_name(&config.dark_theme).ok_or_else(|| unknown_theme(&config.dark_theme))?;

    // Scope theme styles to the code-block wrapper class emitted by
    // MarmiteHighlighter::write_pre_tag; the rules include `background:` and
    // `color:` declarations that would otherwise recolor the whole page.
    let scope = "pre.marmite-code";

    let mut out = String::new();
    let _ = writeln!(
        out,
        "/* Generated by Marmite. Syntax highlighting via arborium.\n * light: {}\n * dark:  {}\n */",
        light.name, dark.name
    );
    // Baseline: light theme before JS runs and when no data-theme is set.
    out.push_str(&light.to_css(scope));
    out.push('\n');
    // OS-preference fallback for users whose JS hasn't yet applied data-theme.
    out.push_str("@media (prefers-color-scheme: dark) {\n");
    out.push_str(&dark.to_css(scope));
    out.push_str("\n}\n");
    // Explicit JS-set overrides — higher specificity than the baseline.
    out.push_str(&light.to_css(&format!("html[data-theme=\"light\"] {scope}")));
    out.push('\n');
    out.push_str(&dark.to_css(&format!("html[data-theme=\"dark\"] {scope}")));
    out.push('\n');
    Ok(out)
}

fn unknown_theme(name: &str) -> String {
    format!("unknown theme '{name}'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_themes_resolve() {
        let cfg = CodeHighlightConfig::default();
        assert!(theme_by_name(&cfg.light_theme).is_some());
        assert!(theme_by_name(&cfg.dark_theme).is_some());
    }

    #[test]
    fn theme_name_matching_is_slug_tolerant() {
        assert!(theme_by_name("github-light").is_some());
        assert!(theme_by_name("github_light").is_some());
        assert!(theme_by_name("GitHub Light").is_some());
        assert!(theme_by_name("catppuccin-frappe").is_some());
    }

    #[test]
    fn generate_css_contains_both_scopes() {
        let cfg = CodeHighlightConfig::default();
        let css = generate_css(&cfg).expect("default themes should generate CSS");
        assert!(
            css.contains("html[data-theme=\"light\"]"),
            "missing light scope"
        );
        assert!(
            css.contains("html[data-theme=\"dark\"]"),
            "missing dark scope"
        );
        assert!(
            css.contains("prefers-color-scheme: dark"),
            "missing media-query fallback"
        );
    }

    #[test]
    fn mermaid_fence_bypasses_highlighter() {
        let hl = MarmiteHighlighter::new();
        let body = "graph LR\n  A --> B\n";
        let mut out = String::new();
        hl.write_highlighted(&mut out, Some("mermaid"), body)
            .expect("write_highlighted should succeed for mermaid");

        let mut expected = String::new();
        html::escape(&mut expected, body).unwrap();
        assert_eq!(
            out, expected,
            "mermaid fences must pass through as escaped raw source for MermaidJS"
        );
    }

    #[test]
    fn build_rejects_unknown_theme() {
        let cfg = CodeHighlightConfig {
            enabled: true,
            light_theme: "no-such-theme-please".to_string(),
            dark_theme: "github-dark".to_string(),
        };
        assert!(build(&cfg).is_err());
    }
}
