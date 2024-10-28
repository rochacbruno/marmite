use crate::content::{get_date, get_slug, get_tags, get_title, Content};
use crate::site::Data;
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter};
use std::fs;
use std::path::Path;

pub fn process_file(path: &Path, site_data: &mut Data) -> Result<(), String> {
    let content = get_content(path)?;

    if content.date.is_some() {
        site_data.posts.push(content);
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

pub fn get_content(path: &Path) -> Result<Content, String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = parse_front_matter(&file_content)?;
    let html = get_html(markdown);
    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, path);
    let date = get_date(&frontmatter, path);
    let extra = frontmatter.get("extra").map(std::borrow::ToOwned::to_owned);
    let content = Content {
        title,
        slug,
        html,
        tags,
        date,
        extra,
    };
    Ok(content)
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

    markdown_to_html(markdown, &options)
}

fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
    if content.starts_with("---") {
        extract(content).map_err(|e| e.to_string())
    } else {
        Ok((Frontmatter::new(), content))
    }
}
