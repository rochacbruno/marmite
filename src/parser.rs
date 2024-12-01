use crate::content::slugify;

use comrak::{markdown_to_html, BrokenLinkReference, ComrakOptions, ResolvedReference};
use frontmatter_gen::{detect_format, extract_raw_frontmatter, parse, Frontmatter};
use log::warn;
use regex::Regex;

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
