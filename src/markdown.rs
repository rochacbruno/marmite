use crate::config::Marmite;
use crate::content::{
    get_authors, get_date, get_description, get_slug, get_stream, get_tags, get_title, slugify,
    Content,
};

use comrak::{markdown_to_html, BrokenLinkReference, ComrakOptions, ResolvedReference};
use frontmatter_gen::{detect_format, extract_raw_frontmatter, parse, Frontmatter};
use log::warn;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use url::Url;

/// From the file content, extract the frontmatter and the markdown content
/// then parse the markdown content to html and return a Content struct
/// if the file is a fragment, the markdown content will be modified to include the references
/// if is a regular content then content will be modified to include the `markdown_header`
/// and `markdown_footer` and references
pub fn get_content(
    path: &Path,
    fragments: Option<&HashMap<String, String>>,
    site: &Marmite,
) -> Result<Content, String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, raw_markdown) = parse_front_matter(&file_content)?;
    let (title, markdown_without_title) = get_title(&frontmatter, raw_markdown);

    let is_fragment = path.file_name().unwrap().to_str().unwrap().starts_with('_');
    let html = if is_fragment {
        let references_path = path.with_file_name("_references.md");
        let mut raw_markdown = raw_markdown.to_string();
        if path != references_path {
            raw_markdown = append_references(&raw_markdown, &references_path);
        }
        get_html(&raw_markdown)
    } else if fragments.is_some() {
        let mut markdown_without_title = markdown_without_title.to_string();
        if let Some(header) = fragments.and_then(|f| f.get("markdown_header")) {
            markdown_without_title.insert_str(0, format!("{header}\n\n").as_str());
        }
        if let Some(footer) = fragments.and_then(|f| f.get("markdown_footer")) {
            markdown_without_title.push_str(format!("\n\n{footer}").as_str());
        }
        if let Some(references) = fragments.and_then(|f| f.get("references")) {
            markdown_without_title.push_str(format!("\n\n{references}").as_str());
        }
        get_html(&markdown_without_title)
    } else {
        get_html(&markdown_without_title)
    };

    let description = get_description(&frontmatter);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, path);
    let date = get_date(&frontmatter, path);
    let extra = frontmatter.get("extra").map(std::borrow::ToOwned::to_owned);
    let links_to = get_links_to(&html);
    let back_links = Vec::new(); // will be mutated later
    let card_image = get_card_image(&frontmatter, &html, path, &slug);
    let banner_image = get_banner_image(&frontmatter, path, &slug);
    let authors = get_authors(&frontmatter, Some(site.default_author.clone()));
    let pinned = frontmatter
        .get("pinned")
        .map_or(false, |p| p.as_bool().unwrap_or(false));

    let toc = if frontmatter
        .get("toc")
        .map_or(site.toc, |t| t.as_bool().unwrap_or(site.toc))
    {
        Some(get_table_of_contents_from_html(&html))
    } else {
        None
    };

    let stream = if date.is_some() {
        get_stream(&frontmatter)
    } else {
        None
    };

    let content = Content {
        title,
        description,
        slug,
        html,
        tags,
        date,
        extra,
        links_to,
        back_links,
        card_image,
        banner_image,
        authors,
        stream,
        pinned,
        toc,
    };
    Ok(content)
}

pub fn append_references(content: &str, references_path: &Path) -> String {
    if references_path.exists() {
        let references = fs::read_to_string(references_path).unwrap_or_default();
        format!("{content}\n\n{references}")
    } else {
        content.to_string()
    }
}

/// Capture `card_image` from frontmatter, then if not defined
/// take the first img src found in the post content
pub fn get_card_image(
    frontmatter: &Frontmatter,
    html: &str,
    path: &Path,
    slug: &str,
) -> Option<String> {
    if let Some(card_image) = frontmatter.get("card_image") {
        return Some(card_image.to_string());
    }

    // Try to find image matching the slug
    if let Some(value) = find_matching_file(slug, path, "card", &["png", "jpg", "jpeg"]) {
        return Some(value);
    }

    // try banner_image
    if let Some(banner_image) = get_banner_image(frontmatter, path, slug) {
        return Some(banner_image);
    }

    // first <img> src attribute
    let img_regex = Regex::new(r#"<img[^>]*src="([^"]+)""#).unwrap();
    img_regex
        .captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn find_matching_file(slug: &str, path: &Path, kind: &str, exts: &[&str]) -> Option<String> {
    // check if a file named {slug}.card.{png,jpg,jpeg} exists in the same directory
    for ext in exts {
        let image_filename = format!("{slug}.{kind}.{ext}");
        let mut path = path.to_path_buf();
        path.pop();
        path.push("media");
        path.push(&image_filename);
        if path.exists() {
            return Some(format!("media/{image_filename}"));
        }
    }
    None
}

fn get_banner_image(frontmatter: &Frontmatter, path: &Path, slug: &str) -> Option<String> {
    if let Some(banner_image) = frontmatter.get("banner_image") {
        return Some(banner_image.as_str().unwrap().trim_matches('"').to_string());
    }

    // Try to find image matching the slug
    if let Some(value) = find_matching_file(slug, path, "banner", &["png", "jpg", "jpeg"]) {
        return Some(value);
    }

    // attempt to get extra.banner_image
    if let Some(extra) = frontmatter.get("extra") {
        if let Some(extra) = extra.as_object() {
            if let Some(banner_image) = extra.get("banner_image") {
                let url = banner_image.to_string();
                // trim start and end quotes
                return Some(url.trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Extract all the internal links from the html content
/// that point to a internal .html file (excluding http links)
/// and return them as a vector of strings
pub fn get_links_to(html: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let re = Regex::new(r#"href="([^"]+)\.html""#).unwrap();
    for cap in re.captures_iter(html) {
        if let Some(m) = cap.get(1) {
            let href = m.as_str();
            if !href.starts_with("http") {
                result.push(href.trim_start_matches("./").to_string());
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
        || (original.len() == 1 && !original.chars().next().unwrap().is_ascii_digit()) // task checkboxes
        || original.is_empty(); // empty links
    if !is_allowed {
        warn!("Reference missing: [{original}] - add '[{original}]: url' to the end of your content file or to the '_references.md' file.");
    }
    None
}

pub fn get_table_of_contents_from_html(html: &str) -> String {
    let re =
        Regex::new(r#"<h([1-6])[^>]*>(?:<a[^>]*href="([^"]+)"[^>]*></a>)?(.*?)</h[1-6]>"#).unwrap();
    let mut toc = String::new();
    let mut last_level = 0;

    for cap in re.captures_iter(html) {
        let level = cap.get(1).map_or(0, |m| m.as_str().parse().unwrap());
        let title = cap.get(3).map_or("", |m| m.as_str());
        let slug = cap.get(2).map_or_else(
            || format!("#{}", slugify(title)),
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

        toc.push_str(&format!("<li><a href=\"{slug}\">{title}</a></li>\n"));
        last_level = level;
    }

    for _ in 0..last_level {
        toc.push_str("</ul>\n");
    }

    toc
}

/// Convert markdown to html using comrak
pub fn get_html(markdown: &str) -> String {
    let mut options = ComrakOptions::default();
    options.render.unsafe_ = true;
    options.render.ignore_empty_links = true;
    options.render.figure_with_caption = true;
    options.parse.relaxed_tasklist_matching = true;
    options.parse.broken_link_callback = Some(Arc::new(warn_broken_link));
    options.extension.tagfilter = false;
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.multiline_block_quotes = true;
    options.extension.underline = true;
    options.extension.spoiler = true;
    options.extension.greentext = true;
    options.extension.shortcodes = true;
    options.extension.header_ids = Some(String::new());
    options.extension.wikilinks_title_before_pipe = true;
    // options.extension.image_url_rewriter = TODO: implement this to point to a resized image

    fix_internal_links(&markdown_to_html(markdown, &options))
}

/// Takes the html content, finds all the internal links and
/// fixes them to point to the correct html file
/// Also removes the .md|.html extension from the text of the link
pub fn fix_internal_links(html: &str) -> String {
    let re = Regex::new(r#"<a[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#).unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let link = caps.get(0).map_or("", |m| m.as_str());
        let href = caps.get(1).map_or("", |m| m.as_str());
        let text = caps.get(2).map_or("", |m| m.as_str());
        if link.contains("class=\"anchor\"")
            || link.contains("data-footnote-ref")
            || link.contains("footnote-backref")
            || link.starts_with('/')
            || href.starts_with('.')
        {
            return link.to_string();
        }

        if let Ok(url) = Url::parse(href) {
            if !url.scheme().is_empty() {
                return link.to_string();
            }
        }

        let new_href = if let Ok(parsed) = Url::parse(&format!("m://m/{href}")) {
            let path = slugify(
                parsed
                    .path()
                    .trim_start_matches('/')
                    .trim_end_matches(".md")
                    .trim_end_matches(".html"),
            );
            let fragment = match parsed.fragment() {
                Some(f) => slugify(f),
                None => String::new(),
            };

            let mut new_href = String::new();
            if !path.is_empty() {
                new_href.push_str(&format!("{path}.html"));
            }
            if !fragment.is_empty() {
                new_href.push_str(&format!("#{fragment}"));
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

fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
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
mod tests {
    use super::*;

    #[test]
    fn test_fix_internal_links_with_md_extension() {
        let html = r#"<a href="test.md">test.md</a>"#;
        let expected = r#"<a href="test.html">test</a>"#;
        assert_eq!(fix_internal_links(html), expected);
    }

    #[test]
    fn test_fix_internal_links_with_html_extension() {
        let html = r#"<a href="test.html">test.html</a>"#;
        let expected = r#"<a href="test.html">test</a>"#;
        assert_eq!(fix_internal_links(html), expected);
    }

    #[test]
    fn test_fix_internal_links_without_extension() {
        let html = r#"<a href="test">test</a>"#;
        let expected = r#"<a href="test.html">test</a>"#;
        assert_eq!(fix_internal_links(html), expected);
    }

    #[test]
    fn test_fix_internal_links_external_link() {
        let html = r#"<a href="http://example.com">example</a>"#;
        let expected = r#"<a href="http://example.com">example</a>"#;
        assert_eq!(fix_internal_links(html), expected);
    }

    #[test]
    fn test_fix_internal_links_mixed_content() {
        let html = r#"<a href="test.md">test.md</a> and <a href="http://example.com">example</a>"#;
        let expected =
            r#"<a href="test.html">test</a> and <a href="http://example.com">example</a>"#;
        assert_eq!(fix_internal_links(html), expected);
    }

    #[test]
    fn test_get_links_to_with_internal_links() {
        let html = r#"<a href="./test1.html">test1</a> <a href="./test2.html">test2</a>"#;
        let expected = Some(vec!["test1".to_string(), "test2".to_string()]);
        assert_eq!(get_links_to(html), expected);
    }

    #[test]
    fn test_get_links_to_with_internal_links_no_slash() {
        let html = r#"<a href="test1.html">test1</a> <a href="test2.html">test2</a>"#;
        let expected = Some(vec!["test1".to_string(), "test2".to_string()]);
        assert_eq!(get_links_to(html), expected);
    }

    #[test]
    fn test_get_links_to_with_no_internal_links() {
        let html = r#"<a href="http://example.com">example</a>"#;
        let expected: Option<Vec<String>> = None;
        assert_eq!(get_links_to(html), expected);
    }

    #[test]
    fn test_get_links_to_with_mixed_links() {
        let html = r#"<a href="./test1.html">test1</a> <a href="test2.html">test2</a> <a href="http://example.com">example</a>"#;
        let expected = Some(vec!["test1".to_string(), "test2".to_string()]);
        assert_eq!(get_links_to(html), expected);
    }

    #[test]
    fn test_get_links_to_with_no_links() {
        let html = r"<p>No links here</p>";
        let expected: Option<Vec<String>> = None;
        assert_eq!(get_links_to(html), expected);
    }

    #[test]
    fn test_get_links_to_with_empty_string() {
        let html = "";
        let expected: Option<Vec<String>> = None;
        assert_eq!(get_links_to(html), expected);
    }

    #[test]
    fn test_get_html_basic_markdown() {
        let markdown = "# Title\n\nThis is a paragraph.";
        let expected = "<h1><a href=\"#title\" aria-hidden=\"true\" class=\"anchor\" id=\"title\"></a>Title</h1>\n<p>This is a paragraph.</p>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_links() {
        let markdown = "[example](http://example.com)";
        let expected = "<p><a href=\"http://example.com\">example</a></p>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_internal_relative_links() {
        let markdown = "[internal](./test.md)";
        let expected = "<p><a href=\"./test.md\">internal</a></p>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_internal_links_no_slash() {
        let markdown = "[internal](test.md)";
        let expected = "<p><a href=\"test.html\">internal</a></p>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_images() {
        let markdown = "![alt text](media/image.jpg)";
        let expected = "<p><figure><img src=\"media/image.jpg\" alt=\"alt text\" /></figure></p>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_code_block() {
        let markdown = "```\nlet x = 1;\n```";
        let expected = "<pre><code>let x = 1;\n</code></pre>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_task_list() {
        let markdown = "- [x] Task 1\n- [ ] Task 2";
        let expected = "<ul>\n<li><input type=\"checkbox\" checked=\"\" disabled=\"\" /> Task 1</li>\n<li><input type=\"checkbox\" disabled=\"\" /> Task 2</li>\n</ul>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_html_with_table() {
        let markdown = "| Header1 | Header2 |\n| ------- | ------- |\n| Cell1   | Cell2   |";
        let expected = "<table>\n<thead>\n<tr>\n<th>Header1</th>\n<th>Header2</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>Cell1</td>\n<td>Cell2</td>\n</tr>\n</tbody>\n</table>\n";
        assert_eq!(get_html(markdown), expected);
    }

    #[test]
    fn test_get_card_image_from_frontmatter() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert(
            "card_image".to_string(),
            frontmatter_gen::Value::String("media/image.jpg".to_string()),
        );
        let html = r#"<p>Some content</p><img src="media/other.jpg" />"#;
        let expected = Some("\"media/image.jpg\"".to_string());
        // assert_eq!(get_card_image(&frontmatter, html, ), expected);
        assert_eq!(
            get_card_image(&frontmatter, html, Path::new("test"), "test"),
            expected
        );
    }

    #[test]
    fn test_get_card_image_from_html() {
        let frontmatter = Frontmatter::new();
        let html = r#"<p>Some content</p><img src="media/image.jpg" />"#;
        let expected = Some("media/image.jpg".to_string());
        assert_eq!(
            get_card_image(&frontmatter, html, Path::new("test"), "test"),
            expected
        );
    }

    #[test]
    fn test_get_card_image_no_image() {
        let frontmatter = Frontmatter::new();
        let html = "<p>Some content</p>";
        let expected: Option<String> = None;
        assert_eq!(
            get_card_image(&frontmatter, html, Path::new("test"), "test"),
            expected
        );
    }

    #[test]
    fn test_get_card_image_with_multiple_images() {
        let frontmatter = Frontmatter::new();
        let html = r#"<p>Some content</p><img src="image1.jpg" /><img src="image2.jpg" />"#;
        let expected = Some("image1.jpg".to_string());
        assert_eq!(
            get_card_image(&frontmatter, html, Path::new("test"), "test"),
            expected
        );
    }

    #[test]
    fn test_get_card_image_with_invalid_html() {
        let frontmatter = Frontmatter::new();
        let html = r#"<p>Some content</p><img src="image.jpg"#;
        let expected: Option<String> = None;
        assert_eq!(
            get_card_image(&frontmatter, html, Path::new("test"), "test"),
            expected
        );
    }

    #[test]
    fn test_get_content_with_valid_frontmatter() {
        let path = Path::new("test_get_content_with_valid_frontmatter.md");
        let content = r#"
---
title: Test Title
description: "Test Description"
tags: ["tag1", "tag2"]
slug: "test-title"
date: "2023-01-01"
---
# Test Content
This is a test content.
"#;
        fs::write(path, content).unwrap();
        let result = get_content(path, None, &Marmite::default()).unwrap();
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.description, Some("\"Test Description\"".to_string()));
        assert_eq!(result.slug, "test-title");
        assert_eq!(result.tags, vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(result.date.unwrap().to_string(), "2023-01-01 00:00:00");
        assert_eq!(result.html, "<h1><a href=\"#test-content\" aria-hidden=\"true\" class=\"anchor\" id=\"test-content\"></a>Test Content</h1>\n<p>This is a test content.</p>\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_get_content_with_invalid_frontmatter() {
        let path = Path::new("test_get_content_with_invalid_frontmatter.md");
        let content = r#"
---
title: "Test Title"
description: "Test Description"
tags: ["tag1", "tag2"
slug: "test-title"
date: "2023-01-01"
extra: "extra content"
---
# Test Content
This is a test content.
"#;
        fs::write(path, content).unwrap();
        let result = get_content(path, None, &Marmite::default());
        assert!(result.is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_get_content_without_frontmatter() {
        let path = Path::new("test_get_content_without_frontmatter.md");
        let content = r"
# Test Content
This is a test content.
";
        fs::write(path, content).unwrap();
        let result = get_content(path, None, &Marmite::default()).unwrap();
        assert_eq!(result.title, "Test Content".to_string());
        assert_eq!(result.description, None);
        assert_eq!(result.slug, "test_get_content_without_frontmatter");
        assert!(result.tags.is_empty());
        assert!(result.date.is_none());
        assert!(result.extra.is_none());
        assert_eq!(result.html, "<p>This is a test content.</p>\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_get_content_with_empty_file() {
        let path = Path::new("test_get_content_with_empty_file.md");
        let content = "";
        fs::write(path, content).unwrap();
        let result = get_content(path, None, &Marmite::default()).unwrap();
        assert_eq!(result.slug, "test_get_content_with_empty_file".to_string());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_get_table_of_contents_from_html_with_single_header() {
        let html = r##"<h1><a href="#header1"></a>Header 1</h1>"##;
        let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n</ul>\n";
        assert_eq!(get_table_of_contents_from_html(html), expected);
    }

    #[test]
    fn test_get_table_of_contents_from_html_with_multiple_headers() {
        let html = r##"
            <h1><a href="#header1"></a>Header 1</h1>
            <h2><a href="#header2"></a>Header 2</h2>
            <h3><a href="#header3"></a>Header 3</h3>
        "##;
        let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n<ul>\n<li><a href=\"#header2\">Header 2</a></li>\n<ul>\n<li><a href=\"#header3\">Header 3</a></li>\n</ul>\n</ul>\n</ul>\n";
        assert_eq!(get_table_of_contents_from_html(html), expected);
    }

    #[test]
    fn test_get_table_of_contents_from_html_with_nested_headers() {
        let html = r##"
            <h1><a href="#header1"></a>Header 1</h1>
            <h2><a href="#header2"></a>Header 2</h2>
            <h1><a href="#header3"></a>Header 3</h1>
        "##;
        let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n<ul>\n<li><a href=\"#header2\">Header 2</a></li>\n</ul>\n<li><a href=\"#header3\">Header 3</a></li>\n</ul>\n";
        assert_eq!(get_table_of_contents_from_html(html), expected);
    }

    #[test]
    fn test_get_table_of_contents_from_html_with_no_headers() {
        let html = r"<p>No headers here</p>";
        let expected = "";
        assert_eq!(get_table_of_contents_from_html(html), expected);
    }

    #[test]
    fn test_get_table_of_contents_from_html_with_mixed_content() {
        let html = r##"
            <h1><a href="#header1"></a>Header 1</h1>
            <p>Some content</p>
            <h2><a href="#header2"></a>Header 2</h2>
            <p>More content</p>
        "##;
        let expected = "<ul>\n<li><a href=\"#header1\">Header 1</a></li>\n<ul>\n<li><a href=\"#header2\">Header 2</a></li>\n</ul>\n</ul>\n";
        assert_eq!(get_table_of_contents_from_html(html), expected);
    }
}
