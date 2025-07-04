use crate::cli::Cli;
use crate::config::Marmite;
use crate::parser::{
    append_references, get_html, get_links_to, get_table_of_contents_from_html, parse_front_matter,
};
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
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Serialize)]
pub enum Kind {
    Tag,
    Archive,
    Author,
    Stream,
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

    pub fn entry(&mut self, key: String) -> Entry<String, Vec<Content>> {
        self.map.entry(key)
    }

    pub fn sort_all(&mut self) {
        for contents in self.map.values_mut() {
            contents.sort_by(|a, b| b.date.cmp(&a.date));
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
            Kind::Author | Kind::Stream => {
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
    pub pinned: bool,
    pub toc: Option<String>,
    pub modified_time: Option<i64>,
    pub comments: Option<bool>,
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
            get_stream(&frontmatter)
        } else {
            None
        };

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
            pinned,
            toc,
            modified_time,
            comments,
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
    pinned: Option<bool>,
    toc: Option<String>,
    comments: Option<bool>,
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
            pinned: self.pinned.unwrap_or_default(),
            toc: self.toc,
            modified_time: None,
            comments: self.comments,
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
/// If not found, get the filename without the date
/// If a date is found in the filename, remove it from the slug
/// If a stream is not the default `index`, prepend it to the slug
/// return the slug
pub fn get_slug<'a>(frontmatter: &'a Frontmatter, path: &'a Path) -> String {
    let stream = get_stream(frontmatter).unwrap();
    let mut final_slug: String;

    if let Some(slug) = frontmatter.get("slug") {
        final_slug = slugify(&slug.to_string());
    } else if let Some(title) = frontmatter.get("title") {
        final_slug = slugify(&title.to_string());
    } else {
        final_slug = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_string();
        final_slug = remove_date_from_filename(&final_slug);
    }

    if stream != "index" {
        final_slug = format!("{stream}-{final_slug}");
    }

    final_slug
}

// Remove date prefix from filename `2024-01-01-myfile.md` -> `myfile.md`
// Return filename if no date prefix is found
fn remove_date_from_filename(filename: &str) -> String {
    let date_prefix_re =
        Regex::new(r"^\d{4}-\d{2}-\d{2}([-T]\d{2}([:-]\d{2})?([:-]\d{2})?)?-").unwrap();
    date_prefix_re.replace(filename, "").to_string()
}

/// Capture `stream` from frontmatter
/// If not defined return "index" as default
#[allow(clippy::unnecessary_wraps)]
pub fn get_stream(frontmatter: &Frontmatter) -> Option<String> {
    if let Some(stream) = frontmatter.get("stream") {
        return Some(stream.as_str().unwrap().trim_matches('"').to_string());
    }
    Some("index".to_string())
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

    // Remove empty tags
    tags.iter().filter(|tag| !tag.is_empty()).cloned().collect()
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
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}( \d{2}:\d{2}(:\d{2})?)?").unwrap();
    let input = re.find(input).map_or("", |m| m.as_str());

    input
        .parse::<NaiveDateTime>()
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M"))
        .or_else(|_| {
            NaiveDate::parse_from_str(input, "%Y-%m-%d").map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
}

/// Use regex to extract date from filename `2024-01-01-myfile.md` or `2024-01-01-15-30-myfile.md`
fn extract_date_from_filename(path: &Path) -> Option<NaiveDateTime> {
    if let Some(filename) = path.file_stem().and_then(|stem| stem.to_str()) {
        if let Ok(date) = try_to_parse_date(filename) {
            return Some(date);
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

pub fn slugify(text: &str) -> String {
    let text = text.replace("%20", "-");
    let normalized = text.nfd().collect::<String>().to_lowercase();
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let slug = re.replace_all(&normalized, "-");
    slug.trim_matches('-').to_string()
}

/// Create a new file with the given text as title and slug
pub fn new(input_folder: &Path, text: &str, cli_args: &Arc<Cli>, config_path: &Path) {
    let content_folder = get_content_folder(&Data::from_file(config_path).site, input_folder);
    let mut path = content_folder.clone();
    let slug = slugify(text);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_title_from_frontmatter() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert("title".to_string(), Value::String("Test Title".to_string()));
        let markdown = "# HTML Title";

        let (title, markdown) = get_title(&frontmatter, markdown);
        assert_eq!(title, "Test Title");
        assert!(markdown.contains("HTML Title"));
    }

    #[test]
    fn test_get_title_from_html() {
        let frontmatter = Frontmatter::new();
        let markdown = "# HTML Title";

        let (title, markdown) = get_title(&frontmatter, markdown);
        assert_eq!(title, "HTML Title");
        assert!(!markdown.contains("HTML Title"));
    }

    #[test]
    fn test_get_title_from_html_with_no_title_tag() {
        let frontmatter = Frontmatter::new();
        let markdown = "title here";

        let (title, markdown) = get_title(&frontmatter, markdown);
        assert_eq!(title, "title here");
        assert!(!markdown.contains("title here"));
    }

    #[test]
    fn test_get_title_from_html_with_multiple_lines() {
        let frontmatter = Frontmatter::new();
        let markdown = "
# First Title
Second Title
        ";

        let (title, markdown) = get_title(&frontmatter, markdown);
        assert_eq!(title, "First Title");
        assert!(!markdown.contains("First Title"));
        assert!(markdown.contains("Second Title"));
    }

    #[test]
    fn test_get_description_from_frontmatter() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert(
            "description".to_string(),
            Value::String("Test Description".to_string()),
        );

        let description = get_description(&frontmatter);
        assert_eq!(description, Some("\"Test Description\"".to_string()));
    }

    #[test]
    fn test_get_description_from_empty_frontmatter() {
        let frontmatter = Frontmatter::new();

        let description = get_description(&frontmatter);
        assert_eq!(description, None);
    }

    #[test]
    fn test_get_slug_from_frontmatter() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert("slug".to_string(), Value::String("test-slug".to_string()));
        let path = Path::new("2024-01-01-myfile.md");

        let slug = get_slug(&frontmatter, path);
        assert_eq!(slug, "test-slug");
    }

    #[test]
    fn test_get_slug_from_title() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert("title".to_string(), Value::String("Test Title".to_string()));
        let path = Path::new("2024-01-01-myfile.md");

        let slug = get_slug(&frontmatter, path);
        assert_eq!(slug, "test-title");
    }

    #[test]
    fn test_get_slug_from_filename() {
        let frontmatter = Frontmatter::new();
        let path = Path::new("2024-01-01-myfile.md");

        let slug = get_slug(&frontmatter, path);
        assert_eq!(slug, "myfile");
    }

    #[test]
    fn test_get_slug_from_filename_without_date() {
        let frontmatter = Frontmatter::new();
        let path = Path::new("myfile.md");

        let slug = get_slug(&frontmatter, path);
        assert_eq!(slug, "myfile");
    }

    #[test]
    fn test_get_slug_from_various_filenames() {
        let frontmatter = Frontmatter::new();
        let filenames = vec![
            "my-file.md",
            "2024-01-01-my-file.md",
            "2024-01-01-15-30-my-file.md",
            "2024-01-01-15-30-12-my-file.md",
            "2024-01-01T15:30-my-file.md",
            "2024-01-01T15:30:12-my-file.md",
        ];

        for filename in filenames {
            let path = Path::new(filename);
            let slug = get_slug(&frontmatter, path);
            assert_eq!(slug, "my-file", "Failed for filename: {filename}");
        }
    }

    #[test]
    fn test_get_slug_with_special_characters() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert(
            "title".to_string(),
            Value::String("Test Title with Special Characters!@#".to_string()),
        );
        let path = Path::new("2024-01-01-myfile.md");

        let slug = get_slug(&frontmatter, path);
        assert_eq!(slug, "test-title-with-special-characters");
    }

    #[test]
    fn test_get_tags_from_frontmatter_array() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("tag1".to_string()),
                Value::String("tag2".to_string()),
            ]),
        );

        let tags = get_tags(&frontmatter);
        assert_eq!(tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_get_tags_from_frontmatter_string() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert("tags".to_string(), Value::String("tag1, tag2".to_string()));

        let tags = get_tags(&frontmatter);
        assert_eq!(tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_get_tags_with_no_tags() {
        let frontmatter = Frontmatter::new();

        let tags = get_tags(&frontmatter);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_get_tags_with_empty_str() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert("tags".to_string(), Value::String(String::new()));

        let tags = get_tags(&frontmatter);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_get_date_from_frontmatter() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert(
            "date".to_string(),
            Value::String("2024-01-01 15:40:56".to_string()),
        );
        let path = Path::new("myfile.md");

        let date = get_date(&frontmatter, path).unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(15, 40, 56)
                .unwrap()
        );
    }

    #[test]
    fn test_get_date_from_frontmatter_without_time() {
        let mut frontmatter = Frontmatter::new();
        frontmatter.insert("date".to_string(), Value::String("2024-01-01".to_string()));
        let path = Path::new("myfile.md");

        let date = get_date(&frontmatter, path).unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_get_date_from_filename() {
        let frontmatter = Frontmatter::new();
        let path = Path::new("2024-01-01-myfile.md");

        let date = get_date(&frontmatter, path).unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_get_date_no_date() {
        let frontmatter = Frontmatter::new();
        let path = Path::new("myfile.md");

        let date = get_date(&frontmatter, path);
        assert!(date.is_none());
    }

    #[test]
    fn test_slugify_simple_text() {
        let text = "Simple Text";
        let slug = slugify(text);
        assert_eq!(slug, "simple-text");
    }

    #[test]
    fn test_slugify_with_special_characters() {
        let text = "Text with Special Characters!@#";
        let slug = slugify(text);
        assert_eq!(slug, "text-with-special-characters");
    }

    #[test]
    fn test_slugify_with_accents() {
        let text = "Téxt wíth Áccénts";
        let slug = slugify(text);
        assert_eq!(slug, "te-xt-wi-th-a-cce-nts");
    }

    #[test]
    fn test_slugify_with_multiple_spaces() {
        let text = "Text    with    multiple    spaces";
        let slug = slugify(text);
        assert_eq!(slug, "text-with-multiple-spaces");
    }

    #[test]
    fn test_slugify_with_underscores() {
        let text = "Text_with_underscores";
        let slug = slugify(text);
        assert_eq!(slug, "text-with-underscores");
    }

    #[test]
    fn test_slugify_with_numbers() {
        let text = "Text with numbers 123";
        let slug = slugify(text);
        assert_eq!(slug, "text-with-numbers-123");
    }

    #[test]
    fn test_slugify_empty_string() {
        let text = "";
        let slug = slugify(text);
        assert_eq!(slug, "");
    }

    #[test]
    fn test_check_for_duplicate_slugs_no_duplicates() {
        let post1: Content = ContentBuilder::new()
            .title("Title 1".to_string())
            .slug("slug-1".to_string())
            .build();

        let post2: Content = ContentBuilder::new()
            .title("Title 2".to_string())
            .slug("slug-2".to_string())
            .build();

        let contents = vec![&post1, &post2];
        let result = check_for_duplicate_slugs(&contents);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_for_duplicate_slugs_with_duplicates() {
        let post1: Content = ContentBuilder::new()
            .title("Title 1".to_string())
            .slug("duplicate-slug".to_string())
            .build();

        let post2: Content = ContentBuilder::new()
            .title("Title 2".to_string())
            .slug("duplicate-slug".to_string())
            .build();

        let contents = vec![&post1, &post2];

        let result = check_for_duplicate_slugs(&contents);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "duplicate-slug".to_string());
    }

    #[test]
    fn test_check_for_duplicate_slugs_empty_list() {
        let contents: Vec<&Content> = vec![];

        let result = check_for_duplicate_slugs(&contents);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_date_from_filename_valid_date() {
        let path = Path::new("2024-01-01-myfile.md");
        let date = extract_date_from_filename(path).unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_extract_date_from_filename_invalid_date() {
        let path = Path::new("not-a-date-myfile.md");
        let date = extract_date_from_filename(path);
        assert!(date.is_none());
    }

    #[test]
    fn test_extract_date_from_filename_empty() {
        let path = Path::new("");
        let date = extract_date_from_filename(path);
        assert!(date.is_none());
    }

    #[test]
    fn test_extract_date_from_filename_with_time() {
        let path = Path::new("2024-01-01-15-30-myfile.md");
        let date = extract_date_from_filename(path).unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_extract_date_from_filename_with_multiple_dates() {
        let path = Path::new("2024-01-01-2025-02-02-myfile.md");
        let date = extract_date_from_filename(path).unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_try_to_parse_date() {
        let inputs = vec![
            "2024-01-01",
            "2024-01-01 15:40",
            "2024-01-01-15:40",
            "2024-01-01 15:40:56",
            "2024-01-01-15:40:56",
            "2024-01-01 15:40:56.123Z",
            "2024-01-01T15:40",
            "2024-01-01T15:40:56",
            "2024-01-01T15:40:56.123Z",
            "2024-01-01T15:40:56+0000",
            "2024-01-01T15:40:56.123+0000",
            "2024-01-01T15:40:56.123456+0000",
            "2024-01-01T15:40:56.123456Z",
            "2024-01-01T15:40:56.123456789+0000",
            "2024-01-01T15:40:56.123456789Z",
            "2020-01-19T21:05:12.984Z",
            "2020-01-19T21:05:12+0000",
            "2024-11-22 20:29:53.211984268 +00:00",
        ];

        for input in inputs {
            let date = try_to_parse_date(input);
            assert!(date.is_ok(), "Failed for input: {input}");
        }
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
        let result = Content::from_markdown(path, None, &Marmite::default(), None).unwrap();
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.description, Some("\"Test Description\"".to_string()));
        assert_eq!(result.slug, "test-title");
        assert_eq!(result.tags, vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(result.date.unwrap().to_string(), "2023-01-01 00:00:00");
        assert_eq!(result.html, "<h1><a inert href=\"#test-content\" aria-hidden=\"true\" class=\"anchor\" id=\"test-content\"></a>Test Content</h1>\n<p>This is a test content.</p>\n");
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
        let result = Content::from_markdown(path, None, &Marmite::default(), None);
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
        let result = Content::from_markdown(path, None, &Marmite::default(), None).unwrap();
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
        let result = Content::from_markdown(path, None, &Marmite::default(), None).unwrap();
        assert_eq!(result.slug, "test_get_content_with_empty_file".to_string());
        fs::remove_file(path).unwrap();
    }
}
