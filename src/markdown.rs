use crate::content::{get_date, get_description, get_slug, get_tags, get_title, Content};
use crate::site::Data;
use chrono::Datelike;
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter};
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn process_file(path: &Path, site_data: &mut Data) -> Result<(), String> {
    let content = get_content(path)?;

    if let Some(date) = content.date {
        site_data.posts.push(content.clone());
        // tags
        for tag in content.tags.clone() {
            site_data.tag.entry(tag).or_default().push(content.clone());
        }
        // archive by year
        let year = date.year().to_string();
        site_data
            .archive
            .entry(year)
            .or_default()
            .push(content.clone());
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

pub fn get_content(path: &Path) -> Result<Content, String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = parse_front_matter(&file_content)?;
    // Process markdown content and transform [[text]] into <./text.html>
    let markdown = process_markdown_obsidianlinks(markdown);
    let html = get_html(&markdown);
    let title = get_title(&frontmatter, &markdown);
    let description = get_description(&frontmatter);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, path);
    let date = get_date(&frontmatter, path);
    let extra = frontmatter.get("extra").map(std::borrow::ToOwned::to_owned);
    let links_to = get_links_to(&html);
    let back_links = Vec::new(); // will be mutated later
    let card_image = get_card_image(&frontmatter, &html);
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
    };
    Ok(content)
}


/// TODO(rochacbruno): 
fn process_markdown_obsidianlinks(markdown: &str) -> String {
    // Match code blocks and inline code
    let code_block_re = Regex::new(r"(```[\s\S]*?```|`[^`]*`)").unwrap();
    // Match [[slug]] but not [[slug|url]]
    let link_re = Regex::new(r"\[\[([^\|\]]+)\]\]").unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for code_caps in code_block_re.captures_iter(markdown) {
        let code_start = code_caps.get(0).unwrap().start();
        let code_end = code_caps.get(0).unwrap().end();

        // Text before the code block or inline code
        let text_before_code = &markdown[last_end..code_start];
        let processed_text = link_re.replace_all(text_before_code, |caps: &regex::Captures| {
            let slug = caps.get(1).unwrap().as_str().trim_end_matches(".html").trim_end_matches(".md");
            format!("[{}](./{}.html)", slug, slug)
        });
        result.push_str(&processed_text);

        // Add code block or inline code without modifications
        result.push_str(&markdown[code_start..code_end]);

        last_end = code_end;
    }

    // Remaining text after the last code block or inline code
    let remaining_text = &markdown[last_end..];
    let processed_text = link_re.replace_all(remaining_text, |caps: &regex::Captures| {
        let slug = caps.get(1).unwrap().as_str().trim_end_matches(".html").trim_end_matches(".md");
        format!("[{}](./{}.html)", slug, slug)
    });
    result.push_str(&processed_text);

    result
}

/// Capture `card_image` from frontmatter, then if not defined
/// take the first img src found in the post content
fn get_card_image(frontmatter: &Frontmatter, html: &str) -> Option<String> {
    if let Some(card_image) = frontmatter.get("card_image") {
        return Some(card_image.to_string());
    }
    // first <img> src attribute
    let img_regex = Regex::new(r#"<img[^>]*src="([^"]+)""#).unwrap();
    img_regex
        .captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn get_links_to(html: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let re = Regex::new(r#"href="\./(.*?)\.html""#).unwrap();
    for cap in re.captures_iter(html) {
        if let Some(m) = cap.get(1) {
            result.push(m.as_str().to_string());
        }
    }
    if result.is_empty() {
        return None;
    }
    Some(result)
}

pub fn get_html(markdown: &str) -> String {
    let mut options = ComrakOptions::default();
    options.render.unsafe_ = true;
    options.render.ignore_empty_links = true;
    options.render.figure_with_caption = true;
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
    options.extension.header_ids = Some("tos-".to_string());
    options.extension.wikilinks_title_before_pipe = true;

    markdown_to_html(markdown, &options)
}

fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
    if content.starts_with("---") {
        extract(content).map_err(|e| e.to_string())
    } else {
        Ok((Frontmatter::new(), content))
    }
}
