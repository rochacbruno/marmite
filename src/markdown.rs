use crate::content::{get_date, get_slug, get_tags, get_title, Content};
use crate::site::Data;
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter};
use std::fs;
use std::path::Path;

pub fn process_file(path: &Path, site_data: &mut Data) -> Result<(), String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = parse_front_matter(&file_content)?;
    let mut options = ComrakOptions::default();

    // TODO: Make the following options configurable?
    options.render.unsafe_ = true; // Allow raw html
    options.extension.tagfilter = false;
    options.extension.strikethrough = true; // ~~text~~
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true; // - [ ] item
    options.extension.footnotes = true; // note[^`1]
    options.extension.description_lists = true;
    options.extension.multiline_block_quotes = true; // >>>\ntext\n>>>
    options.extension.underline = true; // __under__
    options.extension.spoiler = true; // this is ||secret|| (depends on css)
    options.extension.greentext = true; // >not a quote
    options.extension.shortcodes = true; // >not a quote

    // The following 3 options breaks MathJax on themes
    // options.extension.superscript = true; // 3^2^
    // options.extension.math_dollars = true; // depends on css
    // options.extension.math_code = true; // depends on css

    let html = markdown_to_html(markdown, &options);

    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, path);
    let date = get_date(&frontmatter, path);

    let extra = match frontmatter.get("extra") {
        Some(v) => Some(v.to_owned()),
        _ => None,
    };

    let content = Content {
        title,
        slug,
        html,
        tags,
        date,
        extra,
    };

    if date.is_some() {
        site_data.posts.push(content);
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
    if content.starts_with("---") {
        extract(content).map_err(|e| e.to_string())
    } else {
        Ok((Frontmatter::new(), content))
    }
}
