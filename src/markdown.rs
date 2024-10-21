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
    options.render.unsafe_ = true; // Allow raw html
    let html = markdown_to_html(markdown, &options);

    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, path);
    let date = get_date(&frontmatter, path);

    let content = Content { title, slug, html, tags, date };

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
