use indexmap::IndexMap;
use serde::Serialize;
use tera::{Kwargs, State, TeraResult, Value};
use url::Url;

use crate::content::Content;
use crate::site::Data;
use crate::workspace::CrossSiteData;

fn parse_site_param(kwargs: &Kwargs) -> Option<String> {
    kwargs
        .get::<&str>("site")
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn prefix_slug(slug: &str, output_path: &str) -> String {
    if output_path.is_empty() {
        slug.to_string()
    } else {
        format!("{output_path}/{slug}")
    }
}

fn collect_cross_site_posts(site_param: &str, cross_site_data: &CrossSiteData) -> Vec<Content> {
    let site_names: Vec<&str> = if site_param == "all" {
        cross_site_data.sites.keys().map(String::as_str).collect()
    } else {
        site_param.split(',').map(str::trim).collect()
    };

    let mut posts = Vec::new();
    for name in site_names {
        if let Some(site_data) = cross_site_data.sites.get(name) {
            for post in &site_data.data.posts {
                let mut prefixed = post.clone();
                prefixed.slug = prefix_slug(&post.slug, &site_data.output_path);
                posts.push(prefixed);
            }
        }
    }
    posts
}

fn collect_cross_site_pages(site_param: &str, cross_site_data: &CrossSiteData) -> Vec<Content> {
    let site_names: Vec<&str> = if site_param == "all" {
        cross_site_data.sites.keys().map(String::as_str).collect()
    } else {
        site_param.split(',').map(str::trim).collect()
    };

    let mut pages = Vec::new();
    for name in site_names {
        if let Some(site_data) = cross_site_data.sites.get(name) {
            for page in &site_data.data.pages {
                let mut prefixed = page.clone();
                prefixed.slug = prefix_slug(&page.slug, &site_data.output_path);
                pages.push(prefixed);
            }
        }
    }
    pages
}

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
    pub path_prefix: String,
    pub all_site_prefixes: Vec<String>,
}

impl UrlFor {
    fn has_site_prefix(&self, path: &str) -> bool {
        self.all_site_prefixes
            .iter()
            .filter(|p| !p.is_empty())
            .any(|p| path.starts_with(&format!("/{p}/")))
    }

    pub fn resolve(&self, path: &str, abs: bool) -> String {
        let mut path = path.trim_start_matches("./").to_string();

        let abs_prefixes = ["http", "https", "mailto"];
        if abs_prefixes.iter().any(|&prefix| path.starts_with(prefix)) {
            return path;
        }

        // Ensure the path starts with "/" by adding it if necessary
        if !path.starts_with('/') {
            path = format!("/{path}");
        }

        // Apply workspace path prefix for non-root sites,
        // but skip if the path already belongs to another site
        if !self.path_prefix.is_empty()
            && !path.starts_with(&format!("/{}/", self.path_prefix))
            && !self.has_site_prefix(&path)
        {
            path = format!("/{}{path}", self.path_prefix);
        }

        // Trim trailing slashes from base_url if it's not empty
        let base_url = if self.base_url.is_empty() {
            String::new()
        } else {
            self.base_url.trim_end_matches('/').to_string()
        };

        // Parse the base_url to extract the path part if not empty.
        // When path_prefix is set (workspace non-default site), the base_url
        // already contains the prefix path (e.g. https://example.com/en),
        // so we must NOT also use it as base_path to avoid double-prefixing.
        let base_path = if !self.path_prefix.is_empty() || base_url.is_empty() {
            String::new()
        } else {
            Url::parse(&base_url)
                .map(|parsed_url| parsed_url.path().trim_end_matches('/').to_string())
                .unwrap_or_default()
        };

        // Construct the URL based on the presence of base_url and abs flag
        if abs && !base_url.is_empty() {
            // Absolute URL with base_url.
            // When path_prefix is set, strip the path part from base_url since
            // path_prefix is already included in the resolved path.
            let abs_base = if self.path_prefix.is_empty() {
                base_url.clone()
            } else {
                Url::parse(&base_url).map_or_else(
                    |_| base_url.clone(),
                    |parsed_url| {
                        let host = parsed_url.host_str().unwrap_or("");
                        let scheme = parsed_url.scheme();
                        if let Some(port) = parsed_url.port() {
                            format!("{scheme}://{host}:{port}")
                        } else {
                            format!("{scheme}://{host}")
                        }
                    },
                )
            };
            format!("{}/{}", abs_base, path.trim_start_matches('/'))
        } else if !base_path.is_empty() {
            // Relative URL with base path from base_url
            format!("{base_path}{path}")
        } else {
            // Just the path if no base_url or base_path
            path
        }
    }
}

impl tera::Function<TeraResult<Value>> for UrlFor {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let path: &str = kwargs.must_get("path")?;
        let abs: bool = kwargs.get::<bool>("abs")?.unwrap_or(false);

        Ok(Value::from(self.resolve(path, abs)))
    }
}

/// Tera template function that takes a `kind` argument and returns the grouped content
/// based on the kind. The function is used to group the content by tags or archive.
/// The grouped content is built using the `site_data` field from the `Group` struct.
/// and converted to an `IndexMap` to preserve the order of insertion that is
/// determined by the iter on `GroupedContent`.
pub struct Group {
    pub site_data: Data,
    pub cross_site_data: Option<CrossSiteData>,
}

fn get_grouped_content_from_data<'a>(
    data: &'a Data,
    kind: &str,
) -> Option<&'a crate::content::GroupedContent> {
    match kind {
        "tag" => Some(&data.tag),
        "archive" => Some(&data.archive),
        "author" => Some(&data.author),
        "stream" => Some(&data.stream),
        "series" => Some(&data.series),
        _ => None,
    }
}

fn sort_group_list(group_list: &mut [(String, Vec<Content>)], kind: &str, ord: &str) {
    match kind {
        "archive" => {
            if ord == "asc" {
                group_list.reverse();
            }
        }
        _ => {
            if ord == "asc" {
                group_list.sort_by(|a, b| a.0.cmp(&b.0));
            } else {
                group_list.sort_by_key(|a| std::cmp::Reverse(a.1.len()));
            }
        }
    }
}

fn merge_grouped_content(
    site_param: &str,
    kind: &str,
    cross_site_data: &CrossSiteData,
) -> std::collections::HashMap<String, Vec<Content>> {
    let site_names: Vec<&str> = if site_param == "all" {
        cross_site_data.sites.keys().map(String::as_str).collect()
    } else {
        site_param.split(',').map(str::trim).collect()
    };

    let mut merged: std::collections::HashMap<String, Vec<Content>> =
        std::collections::HashMap::new();
    for name in site_names {
        if let Some(site_data) = cross_site_data.sites.get(name) {
            if let Some(grouped) = get_grouped_content_from_data(&site_data.data, kind) {
                for (key, posts) in grouped.iter() {
                    let entry = merged.entry(key.clone()).or_default();
                    for post in posts {
                        let mut prefixed = post.clone();
                        prefixed.slug = prefix_slug(&post.slug, &site_data.output_path);
                        entry.push(prefixed);
                    }
                }
            }
        }
    }
    merged
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl tera::Function<TeraResult<Value>> for Group {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let kind: &str = kwargs.must_get("kind")?;
        let ord: &str = kwargs.get::<&str>("ord")?.unwrap_or("desc");
        let items: usize = kwargs
            .get::<i64>("items")
            .ok()
            .flatten()
            .map(|n| n as usize)
            .or_else(|| {
                kwargs
                    .get::<&str>("items")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap_or(0);

        let site_param = parse_site_param(&kwargs);

        if let (Some(ref site_param), Some(ref csd)) = (&site_param, &self.cross_site_data) {
            let merged = merge_grouped_content(site_param, kind, csd);
            let mut group_list: Vec<(String, Vec<Content>)> = merged.into_iter().collect();
            sort_group_list(&mut group_list, kind, ord);
            if items > 0 && items < group_list.len() {
                group_list.truncate(items);
            }
            let mut ordered_map = IndexMap::new();
            for (name, posts) in group_list {
                ordered_map.insert(name, posts);
            }
            return Value::try_from_serializable(&ordered_map);
        }

        let grouped_content = match kind {
            "tag" => &self.site_data.tag,
            "archive" => &self.site_data.archive,
            "author" => &self.site_data.author,
            "stream" => &self.site_data.stream,
            "series" => &self.site_data.series,
            _ => return Err(tera::Error::message("Invalid `kind` argument")),
        };

        // Convert to vector for sorting.
        // For tags, filter out backward-compat duplicate keys and recover original names.
        let mut group_list: Vec<(String, Vec<Content>)> = grouped_content
            .iter()
            .filter(|(key, _)| kind != "tag" || crate::slugify::slugify(key) == key.as_str())
            .map(|(name, posts)| {
                if kind == "tag" {
                    let original_name = posts
                        .iter()
                        .find_map(|content| {
                            content
                                .tags
                                .iter()
                                .find(|t| crate::slugify::slugify(t) == name.as_str())
                                .cloned()
                        })
                        .unwrap_or_else(|| name.clone());
                    (original_name, posts.clone())
                } else {
                    (name.clone(), posts.clone())
                }
            })
            .collect();

        sort_group_list(&mut group_list, kind, ord);

        if items > 0 && items < group_list.len() {
            group_list.truncate(items);
        }

        // Convert back to IndexMap to preserve order
        let mut ordered_map = IndexMap::new();
        for (name, posts) in group_list {
            ordered_map.insert(name, posts);
        }

        Value::try_from_serializable(&ordered_map)
    }
}

/// Tera template function that generates source links for content
/// It takes a `content` argument and returns a URL pointing to the source markdown file
/// If `source_repository` is configured, it generates a link to the repository
/// If `publish_md` is true, it generates a link to the local markdown file
pub struct SourceLink {
    pub site_data: Data,
}

impl tera::Function<TeraResult<Value>> for SourceLink {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let content: &Value = kwargs.must_get("content")?;

        // Extract the source_path from the content
        let source_path = content
            .get_from_path("source_path")
            .and_then(Value::as_str)
            .ok_or_else(|| tera::Error::message("Missing `source_path` in content"))?;

        // If source_repository is configured, generate repository link
        if let Some(source_repository) = &self.site_data.site.source_repository {
            let source_path_buf = std::path::Path::new(source_path);
            let file_name = source_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.md");

            let repo_url = format!("{}/{}", source_repository.trim_end_matches('/'), file_name);
            return Ok(Value::from(repo_url));
        }

        // If publish_md is true and no source_repository, generate local link
        if self.site_data.site.publish_md {
            let source_path_buf = std::path::Path::new(source_path);
            let file_name = source_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.md");

            let local_url = format!("./{file_name}");
            return Ok(Value::from(local_url));
        }

        // Return empty string if neither option is enabled
        Ok(Value::from(""))
    }
}

/// Tera template function that returns the display name for a stream or series
/// It takes a `stream` or `series` argument and returns the configured display name
/// If no display name is configured, returns the stream/series name itself
pub struct DisplayName {
    pub site_data: Data,
    pub kind: String,
}

impl DisplayName {
    pub fn resolve(&self, name: &str) -> String {
        let display_name = match self.kind.as_str() {
            "stream" => self
                .site_data
                .site
                .streams
                .get(name)
                .map(|config| &config.display_name)
                .or_else(|| {
                    self.site_data
                        .site
                        .languages
                        .get(name)
                        .map(|config| &config.display_name)
                }),
            "series" => self
                .site_data
                .site
                .series
                .get(name)
                .map(|config| &config.display_name),
            _ => None,
        };

        display_name.cloned().unwrap_or_else(|| name.to_string())
    }
}

impl tera::Function<TeraResult<Value>> for DisplayName {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let name: &str = kwargs.must_get(&self.kind)?;
        Ok(Value::from(self.resolve(name)))
    }
}

/// Tera function to get filtered and sorted posts
/// Args: ord (optional, default="desc"), items (optional, default=0 for all),
///       site (optional, workspace cross-site query)
pub struct GetPosts {
    pub site_data: Data,
    pub cross_site_data: Option<CrossSiteData>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl tera::Function<TeraResult<Value>> for GetPosts {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let ord: &str = kwargs.get::<&str>("ord")?.unwrap_or("desc");
        let items: usize = kwargs
            .get::<i64>("items")
            .ok()
            .flatten()
            .map(|n| n as usize)
            .or_else(|| {
                kwargs
                    .get::<&str>("items")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap_or(0);

        let mut posts = match (parse_site_param(&kwargs), &self.cross_site_data) {
            (Some(site_param), Some(csd)) => collect_cross_site_posts(&site_param, csd),
            _ => self.site_data.posts.clone(),
        };

        posts.sort_by_key(|a| std::cmp::Reverse(a.date));
        if ord == "asc" {
            posts.reverse();
        }

        if items > 0 && items < posts.len() {
            posts.truncate(items);
        }

        Value::try_from_serializable(&posts)
    }
}

/// Tera function to get filtered and sorted pages
/// Args: ord (optional, default="asc"), items (optional, default=0 for all),
///       site (optional, workspace cross-site query)
pub struct GetPages {
    pub site_data: Data,
    pub cross_site_data: Option<CrossSiteData>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl tera::Function<TeraResult<Value>> for GetPages {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let ord: &str = kwargs.get::<&str>("ord")?.unwrap_or("asc");
        let items: usize = kwargs
            .get::<i64>("items")
            .ok()
            .flatten()
            .map(|n| n as usize)
            .or_else(|| {
                kwargs
                    .get::<&str>("items")
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap_or(0);

        let mut pages = match (parse_site_param(&kwargs), &self.cross_site_data) {
            (Some(site_param), Some(csd)) => collect_cross_site_pages(&site_param, csd),
            _ => self.site_data.pages.clone(),
        };

        pages.sort_by(|a, b| a.title.cmp(&b.title));
        if ord == "desc" {
            pages.reverse();
        }

        if items > 0 && items < pages.len() {
            pages.truncate(items);
        }

        Value::try_from_serializable(&pages)
    }
}

fn resolve_slug_in_data(slug: &str, data: &Data) -> Option<SlugData> {
    if let Some(page) = data.pages.iter().find(|p| p.slug == slug) {
        return Some(SlugData {
            image: page
                .banner_image
                .as_ref()
                .unwrap_or(&data.site.banner_image)
                .clone(),
            slug: slug.to_string(),
            title: page.title.clone(),
            text: page.description.as_ref().unwrap_or(&String::new()).clone(),
            content_type: "page".to_string(),
        });
    }
    if let Some(post) = data.posts.iter().find(|p| p.slug == slug) {
        return Some(SlugData {
            image: post
                .banner_image
                .as_ref()
                .unwrap_or(&data.site.banner_image)
                .clone(),
            slug: slug.to_string(),
            title: post.title.clone(),
            text: post
                .date
                .map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string()),
            content_type: "post".to_string(),
        });
    }
    None
}

/// Tera function to get data by slug for card display
/// Takes a slug and resolves which content type it refers to, returning `SlugData`
pub struct GetDataBySlug {
    pub site_data: Data,
    pub cross_site_data: Option<CrossSiteData>,
}

impl tera::Function<TeraResult<Value>> for GetDataBySlug {
    #[allow(clippy::too_many_lines)]
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let raw_slug: &str = kwargs.must_get("slug")?;
        let site_param = parse_site_param(&kwargs);

        if let Some(ref csd) = self.cross_site_data {
            let sep = &csd.separator;
            if let Some(sep_pos) = raw_slug.find(sep.as_str()) {
                let site_name = &raw_slug[..sep_pos];
                let inner_slug = &raw_slug[sep_pos + sep.len()..];
                if let Some(sd) = csd.sites.get(site_name) {
                    let target_data = &sd.data;
                    let slug_data = resolve_slug_in_data(inner_slug, target_data);
                    if let Some(mut sd_result) = slug_data {
                        sd_result.slug = prefix_slug(&sd_result.slug, &sd.output_path);
                        return Value::try_from_serializable(&sd_result);
                    }
                    return Err(tera::Error::message(format!(
                        "Content not found for slug: {raw_slug}"
                    )));
                }
            }

            if let Some(ref site_name) = site_param {
                if let Some(sd) = csd.sites.get(site_name.as_str()) {
                    let target_data = &sd.data;
                    let slug_data = resolve_slug_in_data(raw_slug, target_data);
                    if let Some(mut sd_result) = slug_data {
                        sd_result.slug = prefix_slug(&sd_result.slug, &sd.output_path);
                        return Value::try_from_serializable(&sd_result);
                    }
                    return Err(tera::Error::message(format!(
                        "Content not found for slug: {raw_slug} in site: {site_name}"
                    )));
                }
            }
        }

        let slug = raw_slug;

        // Check what kind of content this slug refers to
        let slug_data = if slug.starts_with("series-") {
            // Series slug: series-{name}
            let series_name = slug
                .strip_prefix("series-")
                .ok_or_else(|| tera::Error::message("Invalid series slug format"))?;
            if let Some(series_contents) = self.site_data.series.map.get(series_name) {
                let display_name_fn = DisplayName {
                    site_data: self.site_data.clone(),
                    kind: "series".to_string(),
                };
                let title = display_name_fn.resolve(series_name);

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
                return Err(tera::Error::message(format!(
                    "Series not found: {series_name}"
                )));
            }
        } else if slug.starts_with("stream-") {
            // Stream slug: stream-{name}
            // This is a special case, because streams are not prefixed with stream-
            let stream_name = slug
                .strip_prefix("stream-")
                .ok_or_else(|| tera::Error::message("Invalid stream slug format"))?;
            if let Some(stream_contents) = self.site_data.stream.map.get(stream_name) {
                let display_name_fn = DisplayName {
                    site_data: self.site_data.clone(),
                    kind: "stream".to_string(),
                };
                let title = display_name_fn.resolve(stream_name);

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
                return Err(tera::Error::message(format!(
                    "Stream not found: {stream_name}"
                )));
            }
        } else if slug.starts_with("tag-") {
            // Tag slug: tag-{name}
            let tag_name = slug
                .strip_prefix("tag-")
                .ok_or_else(|| tera::Error::message("Invalid tag slug format"))?;
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
                return Err(tera::Error::message(format!("Tag not found: {tag_name}")));
            }
        } else if slug.starts_with("author-") {
            // Author slug: author-{name}
            let author_name = slug
                .strip_prefix("author-")
                .ok_or_else(|| tera::Error::message("Invalid author slug format"))?;
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
                return Err(tera::Error::message(format!(
                    "Author not found: {author_name}"
                )));
            }
        } else if slug.starts_with("archive-") {
            // Archive slug: archive-{year}
            let year = slug
                .strip_prefix("archive-")
                .ok_or_else(|| tera::Error::message("Invalid archive slug format"))?;
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
                return Err(tera::Error::message(format!(
                    "Archive year not found: {year}"
                )));
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
                        tera::Error::message(format!("Stream not found: {stream_name}"))
                    })?;
                let display_name_fn = DisplayName {
                    site_data: self.site_data.clone(),
                    kind: "stream".to_string(),
                };
                let title = display_name_fn.resolve(stream_name);
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
                return Err(tera::Error::message(format!(
                    "Content not found for slug: {slug}"
                )));
            }
        };

        Value::try_from_serializable(&slug_data)
    }
}

/// Tera function to get gallery data by path
/// Takes a gallery path and returns the Gallery struct if it exists
pub struct GetGallery {
    pub site_data: Data,
}

impl tera::Function<TeraResult<Value>> for GetGallery {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
        let path: &str = kwargs.must_get("path")?;

        log::info!("GetGallery called with path: {path}");
        log::info!(
            "Available galleries: {:?}",
            self.site_data.galleries.keys().collect::<Vec<_>>()
        );

        // Get the gallery from site_data
        if let Some(gallery) = self.site_data.galleries.get(path) {
            log::info!("Gallery found for path: {path}");
            Ok(Value::try_from_serializable(gallery)?)
        } else {
            log::info!("Gallery not found for path: {path}");
            // Return null if gallery not found
            Ok(Value::none())
        }
    }
}

#[cfg(test)]
#[path = "tests/tera_functions.rs"]
mod tests;
