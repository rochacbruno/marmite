use std::fs;
use std::process;
use std::path::{Path, PathBuf};
use frontmatter_gen::{extract, Frontmatter, Value};
use chrono::{NaiveDate, NaiveDateTime};
use comrak::{markdown_to_html, ComrakOptions};
use crate::site_data::SiteData;
use crate::site_data::Content;

pub fn process_files(folder: &PathBuf, site_data: &mut SiteData) -> Result<(), String> {
    for entry in walkdir::WalkDir::new(folder.join(site_data.site.content_path)) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    process_file(path, site_data)?;
                }
            }
            Err(e) => eprintln!("Error reading entry: {}", e),
        }
    }
    Ok(())
}

fn process_file(path: &Path, site_data: &mut SiteData) -> Result<(), String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = parse_front_matter(&file_content)?;
    let html = markdown_to_html(markdown, &ComrakOptions::default());

    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, &path);
    let date = get_date(&frontmatter, &path);
    let show_in_menu = get_show_in_menu(&frontmatter);

    let content = Content {
        title,
        slug,
        tags,
        html,
        date,
        show_in_menu,
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

fn get_title(frontmatter: &Frontmatter, markdown: &str) -> String {
    match frontmatter.get("title") {
        Some(Value::String(t)) => t.to_string(),
        _ => markdown
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches("#")
            .trim()
            .to_string(),
    }
}

fn get_slug(frontmatter: &Frontmatter, path: &Path) -> String {
    match frontmatter.get("slug") {
        Some(Value::String(slug)) => slug.to_string(),
        _ => path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_string(),
    }
}

fn get_tags(frontmatter: &Frontmatter) -> Vec<String> {
    match frontmatter.get("tags") {
        Some(Value::Array(tags)) => tags
            .iter()
            .map(Value::to_string)
            .map(|t| t.trim_matches('"').to_string())
            .collect(),
        Some(Value::String(tags)) => tags
            .split(',')
            .map(|t| t.trim().to_string())
            .collect(),
        _ => Vec::new(),
    }
}

fn get_date(frontmatter: &Frontmatter, path: &Path) -> Option<NaiveDateTime> {
    if let Some(input) = frontmatter.get("date") {
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M:%S")
        {
            return Some(date);
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M")
        {
            return Some(date);
        } else if let Ok(date) = NaiveDate::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d") {
            return date.and_hms_opt(0, 0, 0);
        } else {
            eprintln!(
                "ERROR: Invalid date format {} when parsing {}",
                input.to_string_representation(),
                path.display()
            );
            process::exit(1);
        }
    }
    None
}

fn get_show_in_menu(frontmatter: &Frontmatter) -> bool {
    frontmatter
        .get("show_in_menu")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

