use crate::cli::Cli;
use crate::config::Marmite;
use crate::image_provider;
use crate::parser::{
    append_references, get_html_with_options, get_links_to, get_table_of_contents_from_html,
    parse_front_matter,
};
use crate::re;
use crate::site::{get_content_folder, Data};
use chrono::{NaiveDate, NaiveDateTime};
use frontmatter_gen::{Frontmatter, Value};
use log::error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub enum Kind {
    Tag,
    Archive,
    Author,
    Stream,
    Series,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize)]
pub struct GroupedContent {
    pub kind: Kind,
    pub map: HashMap<String, Vec<Content>>,
}

impl GroupedContent {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            map: HashMap::new(),
        }
    }

    pub fn entry(&mut self, key: String) -> Entry<'_, String, Vec<Content>> {
        self.map.entry(key)
    }

    pub fn sort_all(&mut self) {
        for contents in self.map.values_mut() {
            match self.kind {
                Kind::Series => {
                    // Series should be sorted chronologically (oldest to newest)
                    contents.sort_by(|a, b| a.date.cmp(&b.date));
                }
                _ => {
                    // All other content types sort newest first
                    contents.sort_by(|a, b| b.date.cmp(&a.date));
                }
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, Vec<Content>)> {
        // Assuming the content is already sorted by date on a previous call of .sort_all
        let mut vec: Vec<(&String, Vec<Content>)> =
            self.map.iter().map(|(k, v)| (k, v.clone())).collect();

        match self.kind {
            Kind::Tag => {
                // sort by number of contents
                vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
            }
            Kind::Archive => {
                // sort by year, newest first
                vec.sort_by(|a, b| b.0.cmp(a.0));
            }
            Kind::Author | Kind::Stream | Kind::Series => {
                // sort alphabetically
                vec.sort_by(|a, b| a.0.cmp(b.0));
            }
        }
        vec.into_iter()
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, Default)]
pub struct Content {
    pub title: String,
    pub description: Option<String>,
    pub slug: String,
    pub html: String,
    pub tags: Vec<String>,
    pub date: Option<NaiveDateTime>,
    pub extra: Option<Value>,
    pub links_to: Option<Vec<String>>,
    pub back_links: Vec<Self>,
    pub card_image: Option<String>,
    pub banner_image: Option<String>,
    pub authors: Vec<String>,
    pub stream: Option<String>,
    pub series: Option<String>,
    pub pinned: bool,
    pub toc: Option<String>,
    pub modified_time: Option<i64>,
    pub comments: Option<bool>,
    pub next: Option<Box<Content>>,
    pub previous: Option<Box<Content>>,
    pub source_path: Option<std::path::PathBuf>,
}

impl Content {
    /// From the file content, extract the frontmatter and the markdown content
    /// then parse the markdown content to html and return a Content struct
    /// if the file is a fragment, the markdown content will be modified to include the references
    /// if is a regular content then content will be modified to include the `markdown_header`
    /// and `markdown_footer` and references
    pub fn from_markdown(
        path: &Path,
        fragments: Option<&HashMap<String, String>>,
        site: &Marmite,
        modified_time: Option<i64>,
    ) -> Result<Content, String> {
        let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (frontmatter, raw_markdown) = parse_front_matter(&file_content)?;
        let (title, markdown_without_title) = get_title(&frontmatter, raw_markdown);

        let is_fragment = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|s| s.starts_with('_'));
        let default_parser_options = crate::config::ParserOptions::default();
        let parser_options = site
            .markdown_parser
            .as_ref()
            .unwrap_or(&default_parser_options);
        let html = if is_fragment {
            let references_path = path.with_file_name("_references.md");
            let mut raw_markdown = raw_markdown.to_string();
            if path != references_path {
                raw_markdown = append_references(&raw_markdown, &references_path);
            }
            get_html_with_options(&raw_markdown, parser_options)
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
            get_html_with_options(&markdown_without_title, parser_options)
        } else {
            get_html_with_options(&markdown_without_title, parser_options)
        };

        let description = get_description(&frontmatter);
        let tags = get_tags(&frontmatter);
        let slug = get_slug(&frontmatter, path);
        let date = get_date(&frontmatter, path);
        let extra = frontmatter.get("extra").map(std::borrow::ToOwned::to_owned);
        let links_to = get_links_to(&html);
        let back_links = Vec::new(); // will be mutated later

        // Download banner image if image provider is configured and this is a post (has date)
        if date.is_some() {
            if let Some(parent) = path.parent() {
                let _ =
                    image_provider::download_banner_image(site, &frontmatter, parent, &slug, &tags);
            }
        }

        let card_image = get_card_image(&frontmatter, &html, path, &slug, &site.media_path);
        let banner_image = get_banner_image(&frontmatter, path, &slug, &site.media_path);
        let authors = get_authors(&frontmatter, Some(site.default_author.clone()));
        let pinned = frontmatter
            .get("pinned")
            .is_some_and(|p| p.as_bool().unwrap_or(false));

        let toc = if frontmatter
            .get("toc")
            .map_or(site.toc, |t| t.as_bool().unwrap_or(site.toc))
        {
            Some(get_table_of_contents_from_html(&html))
        } else {
            None
        };

        let stream = if date.is_some() {
            // For posts with dates, determine stream from frontmatter or filename patterns
            Some(determine_stream(&frontmatter, path))
        } else {
            // For pages without dates, stream is None (pages don't have streams)
            None
        };

        let series = determine_series(&frontmatter);

        let comments = get_comments(&frontmatter);
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
            series,
            pinned,
            toc,
            modified_time,
            comments,
            next: None,
            previous: None,
            source_path: Some(path.to_path_buf()),
        };
        Ok(content)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default)]
pub struct ContentBuilder {
    title: Option<String>,
    description: Option<String>,
    slug: Option<String>,
    html: Option<String>,
    tags: Option<Vec<String>>,
    date: Option<NaiveDateTime>,
    extra: Option<Value>,
    links_to: Option<Vec<String>>,
    back_links: Option<Vec<Content>>,
    card_image: Option<String>,
    banner_image: Option<String>,
    authors: Option<Vec<String>>,
    stream: Option<String>,
    series: Option<String>,
    pinned: Option<bool>,
    toc: Option<String>,
    comments: Option<bool>,
    source_path: Option<std::path::PathBuf>,
}

#[allow(dead_code)]
impl ContentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn slug(mut self, slug: String) -> Self {
        self.slug = Some(slug);
        self
    }

    pub fn html(mut self, html: String) -> Self {
        self.html = Some(html);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn date(mut self, date: NaiveDateTime) -> Self {
        self.date = Some(date);
        self
    }

    pub fn extra(mut self, extra: Value) -> Self {
        self.extra = Some(extra);
        self
    }

    pub fn links_to(mut self, links_to: Vec<String>) -> Self {
        self.links_to = Some(links_to);
        self
    }

    pub fn back_links(mut self, back_links: Vec<Content>) -> Self {
        self.back_links = Some(back_links);
        self
    }

    pub fn card_image(mut self, card_image: String) -> Self {
        self.card_image = Some(card_image);
        self
    }

    pub fn banner_image(mut self, banner_image: String) -> Self {
        self.banner_image = Some(banner_image);
        self
    }

    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.authors = Some(authors);
        self
    }

    pub fn stream(mut self, stream: String) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn series(mut self, series: String) -> Self {
        self.series = Some(series);
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = Some(pinned);
        self
    }

    pub fn toc(mut self, toc: String) -> Self {
        self.toc = Some(toc);
        self
    }

    pub fn comments(mut self, comments: bool) -> Self {
        self.comments = Some(comments);
        self
    }

    pub fn source_path(mut self, source_path: std::path::PathBuf) -> Self {
        self.source_path = Some(source_path);
        self
    }

    pub fn build(self) -> Content {
        Content {
            title: self.title.unwrap_or_default(),
            description: self.description,
            slug: self.slug.unwrap_or_default(),
            html: self.html.unwrap_or_default(),
            tags: self.tags.unwrap_or_default(),
            date: self.date,
            extra: self.extra,
            links_to: self.links_to,
            back_links: self.back_links.unwrap_or_default(),
            card_image: self.card_image,
            banner_image: self.banner_image,
            authors: self.authors.unwrap_or_default(),
            stream: self.stream,
            series: self.series,
            pinned: self.pinned.unwrap_or_default(),
            toc: self.toc,
            modified_time: None,
            comments: self.comments,
            next: None,
            previous: None,
            source_path: self.source_path,
        }
    }
}

/// Try to get the title from the frontmatter
/// If not found, get the first line of the markdown without the leading '#'
/// If no lines are found, return an empty string
/// return (title, markdown without title)
pub fn get_title<'a>(frontmatter: &'a Frontmatter, markdown: &'a str) -> (String, String) {
    let title = match frontmatter.get("title") {
        Some(Value::String(t)) => t.to_string(),
        _ => markdown
            .lines()
            .find(|line| !line.trim().is_empty() && !line.trim().starts_with("<!"))
            .unwrap_or("")
            .trim_start_matches('#')
            .trim()
            .to_string(),
    };
    let markdown = markdown
        .lines()
        .skip_while(|line| line.trim().is_empty() || line.trim_start_matches('#').trim() == title)
        .collect::<Vec<&str>>()
        .join("\n");
    (title, markdown)
}

pub fn get_description(frontmatter: &Frontmatter) -> Option<String> {
    if let Some(description) = frontmatter.get("description") {
        return Some(description.to_string());
    }
    None
}

pub fn get_comments(frontmatter: &Frontmatter) -> Option<bool> {
    if let Some(comments) = frontmatter.get("comments") {
        return comments.as_bool();
    }
    None
}

/// Try to get the slug from the frontmatter
/// If not found, get the title from the frontmatter
/// If not found, get the filename without the date and stream prefix
/// If a stream is not the default `index`, prepend it to the slug
/// return the slug
pub fn get_slug<'a>(frontmatter: &'a Frontmatter, path: &'a Path) -> String {
    let stream = determine_stream(frontmatter, path);
    let mut final_slug: String;

    if let Some(slug) = frontmatter.get("slug") {
        final_slug = slug::slugify(slug.to_string());
    } else if let Some(title) = frontmatter.get("title") {
        final_slug = slug::slugify(title.to_string());
    } else {
        final_slug = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("untitled")
            .to_string();
        final_slug = remove_stream_and_date_from_filename(&final_slug);
    }

    if stream != "index" {
        final_slug = format!("{stream}-{final_slug}");
    }

    final_slug
}

/// Determine stream from frontmatter or filename
fn determine_stream(frontmatter: &Frontmatter, path: &Path) -> String {
    // First try frontmatter
    if let Some(stream) = frontmatter.get("stream") {
        return stream
            .as_str()
            .unwrap_or("index")
            .trim_matches('"')
            .to_string();
    }

    // Then try filename patterns
    if let Some(filename_stream) = get_stream_from_filename(path) {
        return filename_stream;
    }

    // Default
    "index".to_string()
}

/// Determine series from frontmatter
fn determine_series(frontmatter: &Frontmatter) -> Option<String> {
    frontmatter
        .get("series")
        .and_then(|s| s.as_str().map(|s| s.trim_matches('"').to_string()))
}

// Remove date prefix from filename `2024-01-01-myfile.md` -> `myfile.md`
// Return filename if no date prefix is found
fn remove_date_from_filename(filename: &str) -> String {
    let date_prefix_re =
        Regex::new(re::MATCH_DATE_PREFIX_FROM_FILENAME).expect("Date prefix regex should compile");
    date_prefix_re.replace(filename, "").to_string()
}

// Remove stream and date prefix from filename
// Handles patterns: `stream-2024-01-01-myfile.md` -> `myfile` and `stream-S-myfile.md` -> `myfile`
fn remove_stream_and_date_from_filename(filename: &str) -> String {
    // Pattern 1: stream-date-slug -> slug (handle time components too)
    let stream_date_pattern = Regex::new(re::CAPTURE_SLUG_FROM_STREAM_DATED_FILENAME)
        .expect("Stream date pattern regex should compile");
    if let Some(captures) = stream_date_pattern.captures(filename) {
        if let Some(slug_match) = captures.get(1) {
            return slug_match.as_str().to_string();
        }
    }

    // Pattern 2: stream-S-slug -> slug
    let stream_s_pattern = Regex::new(re::CAPTURE_SLUG_FROM_STREAM_S_FILENAME)
        .expect("Stream S pattern regex should compile");
    if let Some(captures) = stream_s_pattern.captures(filename) {
        if let Some(slug_match) = captures.get(1) {
            return slug_match.as_str().to_string();
        }
    }

    // Fallback: remove date prefix only
    remove_date_from_filename(filename)
}

/// Extract stream from filename using patterns
/// Returns None if no stream pattern is detected
pub fn get_stream_from_filename(path: &Path) -> Option<String> {
    if let Some(filename) = path.file_stem().and_then(|stem| stem.to_str()) {
        // Pattern 1: {stream}-{date}-{slug} (single word before date)
        if let Some(stream) = extract_stream_from_date_pattern(filename) {
            return Some(stream);
        }

        // Pattern 2: {stream}-S-{slug} (single word before 'S' marker)
        if let Some(stream) = extract_stream_from_s_pattern(filename) {
            return Some(stream);
        }
    }
    None
}

/// Extract stream from filename pattern: {stream}-{date}-{slug}
/// Only accepts single word before date (no hyphens allowed in stream name)
fn extract_stream_from_date_pattern(filename: &str) -> Option<String> {
    let date_pattern = Regex::new(re::CAPTURE_STREAM_AND_DATE_FROM_FILENAME)
        .expect("Date pattern regex should compile");
    if let Some(captures) = date_pattern.captures(filename) {
        if let Some(stream_match) = captures.get(1) {
            return Some(stream_match.as_str().to_string());
        }
    }
    None
}

/// Extract stream from filename pattern: {stream}-S-{slug}
/// Only accepts single word before 'S' marker
fn extract_stream_from_s_pattern(filename: &str) -> Option<String> {
    let s_pattern =
        Regex::new(re::CAPTURE_STREAM_FROM_S_FILENAME).expect("S pattern regex should compile");
    if let Some(captures) = s_pattern.captures(filename) {
        if let Some(stream_match) = captures.get(1) {
            return Some(stream_match.as_str().to_string());
        }
    }
    None
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

    // Remove empty tags but keep original names
    tags.iter()
        .filter(|tag| !tag.is_empty())
        .map(|t| t.trim().to_string())
        .collect()
}

pub fn get_authors(frontmatter: &Frontmatter, default_author: Option<String>) -> Vec<String> {
    let mut authors: Vec<String> = match frontmatter.get("authors") {
        Some(Value::Array(authors)) => authors
            .iter()
            .map(Value::to_string)
            .map(|t| t.trim_matches('"').to_string())
            .collect(),
        Some(Value::String(authors)) => authors
            .split(',')
            .map(str::trim)
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    };
    // If authors is empty, try to get single author from frontmatter
    if authors.is_empty() {
        authors = match frontmatter.get("author") {
            Some(Value::Array(authors)) => authors
                .iter()
                .map(Value::to_string)
                .map(|t| t.trim_matches('"').to_string())
                .collect(),
            Some(Value::String(authors)) => authors
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect(),
            _ => Vec::new(),
        };
    }
    if authors.is_empty() {
        if let Some(default_author) = default_author {
            if !default_author.is_empty() {
                authors.push(default_author);
            }
        }
    }
    authors
}

/// Tries to get `date` from the front-matter metadata, else from filename
/// Input examples:
///   frontmatter = Frontmatter {date: Value("2024-10-10")}
///   path = "2024-01-01-myfile.md"
pub fn get_date(frontmatter: &Frontmatter, path: &Path) -> Option<NaiveDateTime> {
    if let Some(input) = frontmatter.get("date").and_then(|v| v.as_str()) {
        match try_to_parse_date(input) {
            Ok(date) => return Some(date),
            Err(e) => {
                error!(
                    "ERROR: Invalid date format {} when parsing {}, {}",
                    input,
                    path.display(),
                    e
                );
                process::exit(1);
            }
        }
    }
    extract_date_from_filename(path)
}

/// Tries to parse 3 different date formats or return Error.
/// input: "2024-01-01 15:40:56" | "2024-01-01 15:40" | "2024-01-01"
fn try_to_parse_date(input: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    // Fix input to match the format "2023-02-08 19:03:32" or "2023-02-08 19:03" or "2023-02-08"
    // even if the input is on format 2020-01-19T21:05:12.984Z or 2020-01-19T21:05:12+0000
    let re = Regex::new(re::CAPTURE_DATE_PREFIX_FROM_TEXT)
        .expect("Date extraction regex should compile");
    let input = re.find(input).map_or("", |m| m.as_str());

    input
        .parse::<NaiveDateTime>()
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M"))
        .or_else(|_| {
            NaiveDate::parse_from_str(input, "%Y-%m-%d").map(|d| {
                // and_hms_opt should always succeed with valid time values, but provide fallback
                d.and_hms_opt(0, 0, 0).unwrap_or_else(|| {
                    // This should never happen with valid inputs, but provide safe fallback
                    NaiveDate::from_ymd_opt(1970, 1, 1)
                        .unwrap_or_default()
                        .and_hms_opt(0, 0, 0)
                        .unwrap_or_default()
                })
            })
        })
}

/// Use regex to extract date from filename `2024-01-01-myfile.md` or `2024-01-01-15-30-myfile.md`
/// Also handles stream prefixes like `news-2024-01-15-site-update.md`
fn extract_date_from_filename(path: &Path) -> Option<NaiveDateTime> {
    if let Some(filename) = path.file_stem().and_then(|stem| stem.to_str()) {
        // First try direct date parsing (existing behavior for backward compatibility)
        if let Ok(date) = try_to_parse_date(filename) {
            return Some(date);
        }

        // Try to extract date from stream-date-slug pattern
        let stream_date_pattern = Regex::new(re::CAPTURE_DATE_FROM_STREAM_DATED_FILENAME)
            .expect("Stream date extraction regex should compile");
        if let Some(captures) = stream_date_pattern.captures(filename) {
            if let Some(date_match) = captures.get(1) {
                if let Ok(date) = try_to_parse_date(date_match.as_str()) {
                    return Some(date);
                }
            }
        }
    }
    None
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

/// Create a new file with the given text as title and slug
pub fn new(input_folder: &Path, text: &str, cli_args: &Arc<Cli>, config_path: &Path) {
    let content_folder = get_content_folder(&Data::from_file(config_path).site, input_folder);
    let mut path = content_folder.clone();
    let slug = slug::slugify(text);
    if cli_args.create.page {
        path.push(format!("{slug}.md"));
    } else {
        path.push(format!(
            "{}-{}.md",
            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S"),
            slug
        ));
    }
    if path.exists() {
        error!("File already exists: {}", path.display());
        return;
    }
    let mut file = match std::fs::File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create file: {e:?}");
            return;
        }
    };
    let content = if cli_args.create.tags.is_some() {
        format!(
            "---\n\
            tags: {tags}\n\
            ---\n\
            # {text}\n\
            \n\
            ",
            text = text,
            tags = cli_args.create.tags.as_deref().unwrap_or(""),
        )
    } else {
        format!("# {text}\n")
    };
    if let Err(e) = file.write_all(content.as_bytes()) {
        error!("Failed to write to file: {e:?}");
        return;
    }
    println!("{}", path.display());
    if cli_args.create.edit {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });
        let status = std::process::Command::new(editor).arg(&path).status();
        if let Err(e) = status {
            error!("Failed to open editor: {e:?}");
        }
    }
}

/// Capture `card_image` from frontmatter, then if not defined
/// take the first img src found in the post content
pub fn get_card_image(
    frontmatter: &Frontmatter,
    html: &str,
    path: &Path,
    slug: &str,
    media_path: &str,
) -> Option<String> {
    if let Some(card_image) = frontmatter.get("card_image") {
        return Some(card_image.to_string());
    }

    // Try to find image matching the slug
    if let Some(value) = find_matching_file(slug, path, "card", &["png", "jpg", "jpeg"], media_path)
    {
        return Some(value);
    }

    // try banner_image
    if let Some(banner_image) = get_banner_image(frontmatter, path, slug, media_path) {
        return Some(banner_image);
    }

    // first <img> src attribute
    let img_regex =
        Regex::new(re::CAPTURE_SRC_FROM_IMG_HTMLTAG).expect("Image src regex should compile");
    img_regex
        .captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Find a matching media file name in the media folder
/// {slug}.{kind}.{ext}
/// if the file exists, return the path to the file
/// if not found in media folder, try to find in the same directory
/// if the file does not exist, return None
fn find_matching_file(
    slug: &str,
    path: &Path,
    kind: &str,
    exts: &[&str],
    media_folder_name: &str,
) -> Option<String> {
    let parent_path = path.parent().unwrap_or(path);
    let media_path = parent_path.join(media_folder_name);
    for ext in exts {
        for image_filename in [format!("{slug}.{kind}.{ext}"), format!("{slug}.{ext}")] {
            let file_path = media_path.join(&image_filename);
            if file_path.exists() {
                return Some(format!("{media_folder_name}/{image_filename}"));
            }
            let file_path = parent_path.join(&image_filename);
            if file_path.exists() {
                return Some(image_filename.to_string());
            }
        }
    }
    None
}

fn get_banner_image(
    frontmatter: &Frontmatter,
    path: &Path,
    slug: &str,
    media_path: &str,
) -> Option<String> {
    if let Some(banner_image) = frontmatter.get("banner_image") {
        return Some(
            banner_image
                .as_str()
                .unwrap_or("")
                .trim_matches('"')
                .to_string(),
        );
    }

    // Try to find image matching the slug
    if let Some(value) =
        find_matching_file(slug, path, "banner", &["png", "jpg", "jpeg"], media_path)
    {
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

#[cfg(test)]
#[path = "tests/content.rs"]
mod tests;
