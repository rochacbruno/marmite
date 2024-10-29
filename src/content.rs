use chrono::{NaiveDate, NaiveDateTime};
use frontmatter_gen::{Frontmatter, Value};
use log::error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::process;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Content {
    pub title: String,
    pub description: Option<String>,
    pub slug: String,
    pub html: String,
    pub tags: Vec<String>,
    pub date: Option<NaiveDateTime>,
    pub extra: Option<Value>,
}

pub fn get_title<'a>(frontmatter: &'a Frontmatter, html: &'a str) -> String {
    match frontmatter.get("title") {
        Some(Value::String(t)) => t.to_string(),
        _ => html
            .lines()
            .find(|line| !line.is_empty())
            .unwrap_or("")
            .trim_start_matches('#')
            .trim()
            .to_string(),
    }
}

pub fn get_description<'a>(frontmatter: &'a Frontmatter) -> Option<String> {
    if let Some(description) = frontmatter.get("description") {
        return Some(description.to_string());
    }
    None
}

pub fn get_slug<'a>(frontmatter: &'a Frontmatter, path: &'a Path) -> String {
    if let Some(slug) = frontmatter.get("slug") {
        return slugify(&slug.to_string());
    }
    if let Some(title) = frontmatter.get("title") {
        return slugify(&title.to_string());
    }

    let slug = path.file_stem().and_then(|stem| stem.to_str()).unwrap();
    if let Some(date) = extract_date_from_filename(path) {
        return slug.replace(&format!("{}-", date.date()), "").to_string();
    }

    slug.to_string()
}

pub fn get_tags(frontmatter: &Frontmatter) -> Vec<String> {
    let tags: Vec<String> = match frontmatter.get("tags") {
        Some(Value::Array(tags)) => tags
            .iter()
            .map(Value::to_string)
            .map(|t| t.trim_matches('"').to_string())
            .collect(),
        Some(Value::String(tags)) => tags.split(',').map(str::trim).map(String::from).collect(),
        _ => Vec::new(),
    };
    tags
}

pub fn group_by_tags(posts: Vec<Content>) -> Vec<(String, Vec<Content>)> {
    // Create a HashMap to store the tags and the corresponding Content items.
    let mut tag_map: HashMap<String, Vec<Content>> = HashMap::new();

    // Iterate over the posts
    for post in posts {
        // For each tag in the current post
        for tag in post.tags.clone() {
            // Insert the tag into the map or push the post into the existing vector
            tag_map.entry(tag).or_default().push(post.clone());
        }
    }

    // Convert the HashMap into a Vec<(String, Vec<Content>)>
    tag_map.into_iter().collect()
}

pub fn get_date(frontmatter: &Frontmatter, path: &Path) -> Option<NaiveDateTime> {
    if let Some(input) = frontmatter.get("date") {
        if let Ok(date) =
            NaiveDateTime::parse_from_str(input.as_str().unwrap(), "%Y-%m-%d %H:%M:%S")
        {
            return Some(date);
        }
        if let Ok(date) = NaiveDateTime::parse_from_str(input.as_str().unwrap(), "%Y-%m-%d %H:%M") {
            return Some(date);
        }
        if let Ok(date) = NaiveDate::parse_from_str(input.as_str().unwrap(), "%Y-%m-%d") {
            return date.and_hms_opt(0, 0, 0);
        }
        error!(
            "ERROR: Invalid date format {} when parsing {}",
            input.to_string_representation(),
            path.display()
        );
        process::exit(1);
    }

    if let Some(date) = extract_date_from_filename(path) {
        return Some(date);
    }

    None
}

pub fn extract_date_from_filename(path: &Path) -> Option<NaiveDateTime> {
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    date_re
        .find(path.to_str().unwrap())
        .and_then(|m| NaiveDate::parse_from_str(m.as_str(), "%Y-%m-%d").ok())
        .and_then(|dt| dt.and_hms_opt(0, 0, 0))
}

pub fn check_for_duplicate_slugs(contents: &Vec<&Content>) -> Result<(), String> {
    let mut seen = HashSet::new();

    for content in contents {
        if !seen.insert(&content.slug) {
            return Err(content.slug.clone());
        }
    }

    Ok(())
}

pub fn slugify(text: &str) -> String {
    let normalized = text.nfd().collect::<String>().to_lowercase();
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let slug = re.replace_all(&normalized, "-");
    slug.trim_matches('-').to_string()
}
