use crate::config::ParserOptions;
use crate::re;
use crate::site::Data;
use comrak::{markdown_to_html, options::BrokenLinkReference, Options, ResolvedReference};
use frontmatter_gen::{detect_format, extract_raw_frontmatter, parse, Frontmatter};
use log::warn;
use regex::Regex;
use std::fmt::Write as _;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use url::Url;

pub fn append_references(content: &str, references_path: &Path) -> String {
    if references_path.exists() {
        let references = fs::read_to_string(references_path).unwrap_or_default();
        format!("{content}\n\n{references}")
    } else {
        content.to_string()
    }
}

/// Extract all the internal links from the html content
/// that point to a internal .html file (excluding http links)
/// and return them as a vector of strings
pub fn get_links_to(html: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let re = Regex::new(re::CAPTURE_SLUG_ANCHOR_FROM_HREF).expect("Links regex should compile");
    for cap in re.captures_iter(html) {
        if let Some(m) = cap.get(1) {
            let href = m.as_str();
            if !href.starts_with("http") {
                let page = href.trim_start_matches("./").to_string();
                let heading = cap.get(2).map_or("", |h| h.as_str());
                result.push(format!("{page}{heading}").to_string());
            }
        }
    }
    if result.is_empty() {
        return None;
    }
    Some(result)
}

#[allow(clippy::needless_pass_by_value)]
fn warn_broken_link(link_ref: BrokenLinkReference) -> Option<ResolvedReference> {
    let original = link_ref.original;
    let is_allowed = original
        .starts_with("http")  // external links
        || original.starts_with('!') // Callouts
        || original.starts_with('#') // anchors
        ||original.starts_with('^') // footnotes
        || original.starts_with('/') // absolute links
        || (original.len() == 1 && original.chars().next().is_some_and(|c| !c.is_ascii_digit())) // task checkboxes
        || original.is_empty(); // empty links
    if !is_allowed {
        warn!("Reference missing: [{original}] - add '[{original}]: url' to the end of your content file or to the '_references.md' file.");
    }
    None
}

pub fn get_table_of_contents_from_html(html: &str) -> String {
    let re = Regex::new(re::CAPTURE_LEVEL_ANCHOR_TEXT_FROM_H_TAG)
        .expect("Table of contents regex should compile");
    let mut toc = String::new();
    let mut last_level = 0;

    for cap in re.captures_iter(html) {
        let level = cap.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let title = cap.get(3).map_or("", |m| m.as_str());
        let slug = cap.get(2).map_or_else(
            || format!("#{}", slug::slugify(title)),
            |m| m.as_str().to_string(),
        );

        match level.cmp(&last_level) {
            std::cmp::Ordering::Greater => {
                for _ in last_level..level {
                    toc.push_str("<ul>\n");
                }
            }
            std::cmp::Ordering::Less => {
                for _ in level..last_level {
                    toc.push_str("</ul>\n");
                }
            }
            std::cmp::Ordering::Equal => {}
        }

        let _ = writeln!(toc, "<li><a href=\"{slug}\">{title}</a></li>");
        last_level = level;
    }

    for _ in 0..last_level {
        toc.push_str("</ul>\n");
    }

    toc
}

/// Convert markdown to html using comrak
#[allow(dead_code)]
pub fn get_html(markdown: &str) -> String {
    get_html_with_options(markdown, &ParserOptions::default())
}

/// Convert markdown to html using comrak with configurable options
pub fn get_html_with_options(markdown: &str, parser_options: &ParserOptions) -> String {
    let mut options = Options::default();

    // Apply configurable render options
    options.render.figure_with_caption = parser_options.render.figure_with_caption;
    options.render.ignore_empty_links = parser_options.render.ignore_empty_links;
    options.render.r#unsafe = parser_options.render.unsafe_;

    // Apply configurable parse options
    options.parse.broken_link_callback = Some(Arc::new(warn_broken_link)); // Not configurable
    options.parse.relaxed_tasklist_matching = parser_options.parse.relaxed_tasklist_matching;

    // Apply configurable extension options
    options.extension.alerts = parser_options.extension.alerts;
    options.extension.autolink = parser_options.extension.autolink;
    options.extension.description_lists = parser_options.extension.description_lists;
    options.extension.footnotes = parser_options.extension.footnotes;
    options.extension.greentext = parser_options.extension.greentext;
    options.extension.header_ids = Some(String::new()); // Not configurable
                                                        // options.extension.image_url_rewriter = TODO: implement this to point to a resized image
    options.extension.multiline_block_quotes = parser_options.extension.multiline_block_quotes;
    options.extension.tagfilter = parser_options.extension.tagfilter;
    options.extension.shortcodes = parser_options.extension.shortcodes;
    options.extension.spoiler = parser_options.extension.spoiler;
    options.extension.strikethrough = parser_options.extension.strikethrough;
    options.extension.table = parser_options.extension.table;
    options.extension.tasklist = parser_options.extension.tasklist;
    options.extension.underline = parser_options.extension.underline;
    options.extension.wikilinks_title_before_pipe =
        parser_options.extension.wikilinks_title_before_pipe;
    options.extension.wikilinks_title_after_pipe =
        parser_options.extension.wikilinks_title_after_pipe;

    fix_internal_links(&markdown_to_html(markdown, &options))
}

/// Takes the html content, finds all the internal links and
/// fixes them to point to the correct html file
/// Also removes the .md|.html extension from the text of the link
pub fn fix_internal_links(html: &str) -> String {
    let re = Regex::new(re::CAPTURE_LINK_AND_TEXT_FROM_A_TAG)
        .expect("Fix internal links regex should compile");
    re.replace_all(html, |caps: &regex::Captures| {
        let link = caps.get(0).map_or("", |m| m.as_str());
        let href = caps.get(1).map_or("", |m| m.as_str());
        let text = caps.get(2).map_or("", |m| m.as_str());
        // Check if this is a media file link
        let media_extensions = [
            ".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg", ".avif", ".bmp", ".tiff", ".tif",
            ".ico", ".pdf", ".mp4", ".mov", ".avi", ".mkv", ".webm", ".mp3", ".wav", ".ogg",
            ".flac", ".zip", ".tar", ".gz", ".7z", ".rar", ".doc", ".docx", ".xls", ".xlsx",
            ".ppt", ".pptx", ".txt", ".csv", ".json", ".xml", ".yaml", ".yml", ".toml",
        ];

        let href_lower = href.to_lowercase();
        let is_media_file = media_extensions.iter().any(|ext| href_lower.ends_with(ext));

        if link.contains("class=\"anchor\"")
            || link.contains("data-footnote-ref")
            || link.contains("footnote-backref")
            || link.starts_with('/')
            || href.starts_with('.')
            || is_media_file
        {
            return link.to_string();
        }

        if let Ok(url) = Url::parse(href) {
            if !url.scheme().is_empty() {
                return link.to_string();
            }
        }

        let new_href = if let Ok(parsed) = Url::parse(&format!("m://m/{href}")) {
            let path = slug::slugify(
                parsed
                    .path()
                    .trim_start_matches('/')
                    .trim_end_matches(".md")
                    .trim_end_matches(".html"),
            );
            let fragment = match parsed.fragment() {
                Some(f) => slug::slugify(f),
                None => String::new(),
            };

            let mut new_href = String::new();
            if !path.is_empty() {
                let _ = write!(new_href, "{path}.html");
            }
            if !fragment.is_empty() {
                let _ = write!(new_href, "#{fragment}");
            }
            new_href
        } else {
            href.to_string()
        };

        let new_text = text
            .trim_start_matches('#')
            .trim_end_matches(".md")
            .trim_end_matches(".html")
            .replace('#', " > ");

        link.replace(&format!("href=\"{href}\""), &format!("href=\"{new_href}\""))
            .replace(&format!(">{text}</a>"), &format!(">{new_text}</a>"))
    })
    .to_string()
}

/// Decode common HTML entities in text
/// Handles &amp;, &lt;, &gt;, &quot;, &#39; and numeric entities
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

/// Find content by title in site data (case-insensitive)
/// Returns the slug of the matching content if found
fn find_content_by_title(title: &str, site_data: &Data) -> Option<String> {
    let title_lower = title.to_lowercase();

    // Search in posts first
    for content in &site_data.posts {
        if content.title.to_lowercase() == title_lower {
            return Some(format!("{}.html", content.slug));
        }
    }

    // Search in pages
    for content in &site_data.pages {
        if content.title.to_lowercase() == title_lower {
            return Some(format!("{}.html", content.slug));
        }
    }

    None
}

/// Fix wikilinks in HTML by replacing filename-based hrefs with slug-based hrefs
/// This function processes HTML that contains wikilinks with data-wikilink="true"
/// and attempts to match the link titles with actual content titles to use proper slugs
pub fn fix_wikilinks(html: &str, site_data: &Data) -> String {
    let re =
        Regex::new(re::CAPTURE_WIKILINK_HREF_AND_TITLE).expect("Wikilink regex should compile");

    re.replace_all(html, |caps: &regex::Captures| {
        let original_link = caps.get(0).map_or("", |m| m.as_str());
        let original_href = caps.get(1).map_or("", |m| m.as_str());
        let link_title = caps.get(2).map_or("", |m| m.as_str());

        // Decode HTML entities from the link title
        let decoded_title = decode_html_entities(link_title);

        // Try to find matching content by title
        if let Some(proper_href) = find_content_by_title(&decoded_title, site_data) {
            // Replace the href with the proper slug-based href
            original_link.replace(
                &format!("href=\"{original_href}\""),
                &format!("href=\"{proper_href}\""),
            )
        } else {
            // No match found, keep original
            original_link.to_string()
        }
    })
    .to_string()
}

pub fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
    let content = content.trim_start_matches('\n');
    let has_frontmatter =
        content.starts_with("---") || content.starts_with("+++") || content.starts_with('{');
    if !has_frontmatter {
        return Ok((Frontmatter::new(), content));
    }
    extract_fm_content(content)
}

pub fn extract_fm_content(content: &str) -> Result<(Frontmatter, &str), String> {
    let (raw_frontmatter, remaining_content) = extract_raw_frontmatter(content)?;
    let format = detect_format(raw_frontmatter)?;
    let frontmatter = parse(raw_frontmatter, format)?;
    Ok((frontmatter, remaining_content))
}

#[cfg(test)]
#[path = "tests/parser.rs"]
mod tests;
