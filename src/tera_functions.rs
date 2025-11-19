use indexmap::IndexMap;
use serde::Serialize;
use std::collections::HashMap;
use tera::{to_value, Function, Result as TeraResult, Value};
use url::Url;

use crate::content::Content;
use crate::site::Data;

#[derive(Serialize)]
pub struct SlugData {
    pub image: String,
    pub slug: String,
    pub title: String,
    pub text: String,
    pub content_type: String,
}

#[derive(Default)]
pub struct UrlFor {
    pub base_url: String,
}

impl Function for UrlFor {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        // Extract the "path" argument
        let mut path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `path` argument"))?
            .trim_start_matches("./")
            .to_string();

        let abs_prefixes = ["http", "https", "mailto"];
        if abs_prefixes.iter().any(|&prefix| path.starts_with(prefix)) {
            return to_value(path).map_err(tera::Error::from);
        }

        // Ensure the path starts with "/" by adding it if necessary
        if !path.starts_with('/') {
            path = format!("/{path}");
        }

        // Trim trailing slashes from base_url if it's not empty
        let base_url = if self.base_url.is_empty() {
            String::new()
        } else {
            self.base_url.trim_end_matches('/').to_string()
        };

        // Parse the base_url to extract the path part if not empty
        let base_path = if base_url.is_empty() {
            String::new()
        } else {
            Url::parse(&base_url)
                .map(|parsed_url| parsed_url.path().trim_end_matches('/').to_string())
                .unwrap_or_default()
        };

        // Check if the "abs" argument is provided and set to true
        let abs = args.get("abs").and_then(Value::as_bool).unwrap_or(false);

        // Construct the URL based on the presence of base_url and abs flag
        let url = if abs && !base_url.is_empty() {
            // Absolute URL with base_url
            format!("{}/{}", base_url, path.trim_start_matches('/'))
        } else if !base_path.is_empty() {
            // Relative URL with base path from base_url
            format!("{base_path}{path}")
        } else {
            // Just the path if no base_url or base_path
            path
        };

        // Return the URL as a Tera Value
        to_value(url).map_err(tera::Error::from)
    }
}

/// Tera template function that takes a `kind` argument and returns the grouped content
/// based on the kind. The function is used to group the content by tags or archive.
/// The grouped content is built using the `site_data` field from the `Group` struct.
/// and converted to an `IndexMap` to preserve the order of insertion that is
/// determined by the iter on `GroupedContent`.
pub struct Group {
    pub site_data: Data,
}

#[allow(clippy::cast_possible_truncation)]
impl Function for Group {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let kind = args
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `kind` argument"))?;

        let ord = args.get("ord").and_then(Value::as_str).unwrap_or("desc");

        let items = args
            .get("items")
            .and_then(|v| match v {
                Value::Number(n) => n.as_u64(),
                Value::String(s) => s.parse::<u64>().ok(),
                _ => None,
            })
            .unwrap_or(0) as usize;

        let grouped_content = match kind {
            "tag" => &self.site_data.tag,
            "archive" => &self.site_data.archive,
            "author" => &self.site_data.author,
            "stream" => &self.site_data.stream,
            "series" => &self.site_data.series,
            _ => return Err(tera::Error::msg("Invalid `kind` argument")),
        };

        // Convert to vector for sorting
        let mut group_list: Vec<(String, Vec<Content>)> = grouped_content
            .iter()
            .map(|(name, posts)| (name.clone(), posts.clone()))
            .collect();

        // Sort based on kind and ord parameter
        match kind {
            "archive" => {
                // Archive is already sorted by year, just reverse if needed
                if ord == "asc" {
                    group_list.reverse();
                }
            }
            _ => {
                // For tag, author, stream, series - sort by post count (desc) or alphabetically by name (asc)
                if ord == "asc" {
                    group_list.sort_by(|a, b| a.0.cmp(&b.0));
                } else {
                    group_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
                }
            }
        }

        // Limit items if specified
        if items > 0 && items < group_list.len() {
            group_list.truncate(items);
        }

        // Convert back to IndexMap to preserve order
        let mut ordered_map = IndexMap::new();
        for (name, posts) in group_list {
            ordered_map.insert(name, posts);
        }

        let json_value = serde_json::to_value(&ordered_map)
            .map_err(|e| tera::Error::msg(format!("Failed to convert to JSON: {e}")))?;

        to_value(json_value).map_err(tera::Error::from)
    }
}

/// Tera template function that generates source links for content
/// It takes a `content` argument and returns a URL pointing to the source markdown file
/// If `source_repository` is configured, it generates a link to the repository
/// If `publish_md` is true, it generates a link to the local markdown file
pub struct SourceLink {
    pub site_data: Data,
}

impl Function for SourceLink {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let content = args
            .get("content")
            .ok_or_else(|| tera::Error::msg("Missing `content` argument"))?;

        // Extract the source_path from the content
        let source_path = content
            .get("source_path")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `source_path` in content"))?;

        // If source_repository is configured, generate repository link
        if let Some(source_repository) = &self.site_data.site.source_repository {
            let source_path_buf = std::path::Path::new(source_path);
            let file_name = source_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.md");

            let repo_url = format!("{}/{}", source_repository.trim_end_matches('/'), file_name);
            return to_value(repo_url).map_err(tera::Error::from);
        }

        // If publish_md is true and no source_repository, generate local link
        if self.site_data.site.publish_md {
            let source_path_buf = std::path::Path::new(source_path);
            let file_name = source_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.md");

            let local_url = format!("./{file_name}");
            return to_value(local_url).map_err(tera::Error::from);
        }

        // Return empty string if neither option is enabled
        to_value("").map_err(tera::Error::from)
    }
}

/// Tera template function that returns the display name for a stream or series
/// It takes a `stream` or `series` argument and returns the configured display name
/// If no display name is configured, returns the stream/series name itself
pub struct DisplayName {
    pub site_data: Data,
    pub kind: String,
}

impl Function for DisplayName {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let name = args
            .get(&self.kind)
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg(format!("Missing `{}` argument", self.kind)))?;

        // Check if there's a configured display name based on the kind
        let display_name = match self.kind.as_str() {
            "stream" => self
                .site_data
                .site
                .streams
                .get(name)
                .map(|config| &config.display_name),
            "series" => self
                .site_data
                .site
                .series
                .get(name)
                .map(|config| &config.display_name),
            _ => None,
        };

        if let Some(display_name) = display_name {
            to_value(display_name).map_err(tera::Error::from)
        } else {
            // Return the name itself if no display name is configured
            to_value(name).map_err(tera::Error::from)
        }
    }
}

/// Tera function to get filtered and sorted posts
/// Args: ord (optional, default="desc"), items (optional, default=0 for all)
pub struct GetPosts {
    pub site_data: Data,
}

#[allow(clippy::cast_possible_truncation)]
impl Function for GetPosts {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let ord = args.get("ord").and_then(Value::as_str).unwrap_or("desc");

        let items = args
            .get("items")
            .and_then(|v| match v {
                Value::Number(n) => n.as_u64(),
                Value::String(s) => s.parse::<u64>().ok(),
                _ => None,
            })
            .unwrap_or(0) as usize;

        let mut posts = self.site_data.posts.clone();

        // Sort posts
        if ord == "asc" {
            posts.reverse();
        }

        // Limit items if specified
        if items > 0 && items < posts.len() {
            posts.truncate(items);
        }

        to_value(posts).map_err(tera::Error::from)
    }
}

/// Tera function to get data by slug for card display
/// Takes a slug and resolves which content type it refers to, returning `SlugData`
pub struct GetDataBySlug {
    pub site_data: Data,
}

impl Function for GetDataBySlug {
    #[allow(clippy::too_many_lines)]
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let slug = args
            .get("slug")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `slug` argument"))?;

        // Check what kind of content this slug refers to
        let slug_data = if slug.starts_with("series-") {
            // Series slug: series-{name}
            let series_name = slug
                .strip_prefix("series-")
                .ok_or_else(|| tera::Error::msg("Invalid series slug format"))?;
            if let Some(series_contents) = self.site_data.series.map.get(series_name) {
                let display_name_fn = DisplayName {
                    site_data: self.site_data.clone(),
                    kind: "series".to_string(),
                };
                let mut args = std::collections::HashMap::new();
                args.insert(
                    "series".to_string(),
                    tera::Value::String(series_name.to_string()),
                );
                let title = display_name_fn
                    .call(&args)
                    .unwrap_or_else(|_| tera::Value::String(series_name.to_string()))
                    .as_str()
                    .unwrap_or(series_name)
                    .to_string();

                let description = self
                    .site_data
                    .site
                    .series
                    .get(series_name)
                    .and_then(|config| config.description.as_ref())
                    .cloned()
                    .unwrap_or_else(|| format!("{} posts", series_contents.len()));

                let image = series_contents
                    .first()
                    .and_then(|content| content.banner_image.as_ref())
                    .unwrap_or(&self.site_data.site.banner_image)
                    .clone();

                SlugData {
                    image,
                    slug: slug.to_string(),
                    title,
                    text: description,
                    content_type: "series".to_string(),
                }
            } else {
                return Err(tera::Error::msg(format!("Series not found: {series_name}")));
            }
        } else if slug.starts_with("stream-") {
            // Stream slug: stream-{name}
            // This is a special case, because streams are not prefixed with stream-
            let stream_name = slug
                .strip_prefix("stream-")
                .ok_or_else(|| tera::Error::msg("Invalid stream slug format"))?;
            if let Some(stream_contents) = self.site_data.stream.map.get(stream_name) {
                let display_name_fn = DisplayName {
                    site_data: self.site_data.clone(),
                    kind: "stream".to_string(),
                };
                let mut args = std::collections::HashMap::new();
                args.insert(
                    "stream".to_string(),
                    tera::Value::String(stream_name.to_string()),
                );
                let title = display_name_fn
                    .call(&args)
                    .unwrap_or_else(|_| tera::Value::String(stream_name.to_string()))
                    .as_str()
                    .unwrap_or(stream_name)
                    .to_string();

                let description = format!("{} posts", stream_contents.len());

                let image = stream_contents
                    .first()
                    .and_then(|content| content.banner_image.as_ref())
                    .unwrap_or(&self.site_data.site.banner_image)
                    .clone();

                SlugData {
                    image,
                    slug: stream_name.to_string(),
                    title,
                    text: description,
                    content_type: "stream".to_string(),
                }
            } else {
                return Err(tera::Error::msg(format!("Stream not found: {stream_name}")));
            }
        } else if slug.starts_with("tag-") {
            // Tag slug: tag-{name}
            let tag_name = slug
                .strip_prefix("tag-")
                .ok_or_else(|| tera::Error::msg("Invalid tag slug format"))?;
            if let Some(tag_contents) = self.site_data.tag.map.get(tag_name) {
                let image = tag_contents
                    .first()
                    .and_then(|content| content.banner_image.as_ref())
                    .unwrap_or(&self.site_data.site.banner_image)
                    .clone();

                SlugData {
                    image,
                    slug: slug.to_string(),
                    title: tag_name.to_string(),
                    text: format!("{} posts", tag_contents.len()),
                    content_type: "tag".to_string(),
                }
            } else {
                return Err(tera::Error::msg(format!("Tag not found: {tag_name}")));
            }
        } else if slug.starts_with("author-") {
            // Author slug: author-{name}
            let author_name = slug
                .strip_prefix("author-")
                .ok_or_else(|| tera::Error::msg("Invalid author slug format"))?;
            if let Some(author_contents) = self.site_data.author.map.get(author_name) {
                let author_info = self.site_data.site.authors.get(author_name);
                let title = author_info
                    .map_or(&author_name.to_string(), |a| &a.name)
                    .clone();

                let image = author_info
                    .and_then(|a| a.avatar.as_ref())
                    .unwrap_or(&self.site_data.site.banner_image)
                    .clone();

                SlugData {
                    image,
                    slug: slug.to_string(),
                    title,
                    text: format!("{} posts", author_contents.len()),
                    content_type: "author".to_string(),
                }
            } else {
                return Err(tera::Error::msg(format!("Author not found: {author_name}")));
            }
        } else if slug.starts_with("archive-") {
            // Archive slug: archive-{year}
            let year = slug
                .strip_prefix("archive-")
                .ok_or_else(|| tera::Error::msg("Invalid archive slug format"))?;
            if let Some(archive_contents) = self.site_data.archive.map.get(year) {
                let image = archive_contents
                    .first()
                    .and_then(|content| content.banner_image.as_ref())
                    .unwrap_or(&self.site_data.site.banner_image)
                    .clone();

                SlugData {
                    image,
                    slug: slug.to_string(),
                    title: format!("Posts from {year}"),
                    text: format!("{} posts", archive_contents.len()),
                    content_type: "archive".to_string(),
                }
            } else {
                return Err(tera::Error::msg(format!("Archive year not found: {year}")));
            }
        } else {
            // Check if it's a page
            if let Some(page) = self.site_data.pages.iter().find(|p| p.slug == slug) {
                SlugData {
                    image: page
                        .banner_image
                        .as_ref()
                        .unwrap_or(&self.site_data.site.banner_image)
                        .clone(),
                    slug: slug.to_string(),
                    title: page.title.clone(),
                    text: page.description.as_ref().unwrap_or(&String::new()).clone(),
                    content_type: "page".to_string(),
                }
            } else if let Some(post) = self.site_data.posts.iter().find(|p| p.slug == slug) {
                // Check if it's a post
                SlugData {
                    image: post
                        .banner_image
                        .as_ref()
                        .unwrap_or(&self.site_data.site.banner_image)
                        .clone(),
                    slug: slug.to_string(),
                    title: post.title.clone(),
                    text: post
                        .date
                        .map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string()),
                    content_type: "post".to_string(),
                }
            } else if self.site_data.stream.map.contains_key(slug) {
                // Check if it's a Stream (streams does not start with stream-, those are just bare slugs)
                // to identify we must look to the site data streams map
                let stream_name = slug;
                let stream_contents =
                    self.site_data.stream.map.get(stream_name).ok_or_else(|| {
                        tera::Error::msg(format!("Stream not found: {stream_name}"))
                    })?;
                let display_name_fn = DisplayName {
                    site_data: self.site_data.clone(),
                    kind: "stream".to_string(),
                };
                let mut args = std::collections::HashMap::new();
                args.insert(
                    "stream".to_string(),
                    tera::Value::String(stream_name.to_string()),
                );
                let title = display_name_fn
                    .call(&args)
                    .unwrap_or_else(|_| tera::Value::String(stream_name.to_string()))
                    .as_str()
                    .unwrap_or(stream_name)
                    .to_string();
                SlugData {
                    image: stream_contents
                        .first()
                        .and_then(|content| content.banner_image.as_ref())
                        .unwrap_or(&self.site_data.site.banner_image)
                        .clone(),
                    slug: slug.to_string(),
                    title,
                    text: format!("{} posts", stream_contents.len()),
                    content_type: "stream".to_string(),
                }
            } else {
                return Err(tera::Error::msg(format!(
                    "Content not found for slug: {slug}"
                )));
            }
        };

        to_value(slug_data).map_err(tera::Error::from)
    }
}

/// Tera function to get gallery data by path
/// Takes a gallery path and returns the Gallery struct if it exists
pub struct GetGallery {
    pub site_data: Data,
}

impl Function for GetGallery {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::msg("Missing `path` argument"))?;

        log::info!("GetGallery called with path: {path}");
        log::info!(
            "Available galleries: {:?}",
            self.site_data.galleries.keys().collect::<Vec<_>>()
        );

        // Get the gallery from site_data
        if let Some(gallery) = self.site_data.galleries.get(path) {
            log::info!("Gallery found for path: {path}");
            to_value(gallery).map_err(tera::Error::from)
        } else {
            log::info!("Gallery not found for path: {path}");
            // Return null if gallery not found
            Ok(Value::Null)
        }
    }
}

#[cfg(test)]
#[path = "tests/tera_functions.rs"]
mod tests;
