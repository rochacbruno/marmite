use crate::config::{Author, LanguageConfig, Marmite};
use crate::content::{
    check_for_duplicate_slugs, detect_language_from_path, is_iso_639_1_code, merge_frontmatter,
    Content, ContentBuilder, GroupedContent, Kind, TranslationRef, ISO_639_1_CODES,
};
use crate::embedded::{
    collect_ignore_missing_includes, generate_static, preprocess_template, Templates,
    EMBEDDED_STATIC,
};
use crate::gallery::Gallery;
use crate::highlight::{self, MarmiteHighlighter};
use crate::image_resize;
use crate::parser::fix_wikilinks;
use crate::shortcodes::ShortcodeProcessor;
use crate::tera_functions::{
    DisplayName, GetDataBySlug, GetGallery, GetPages, GetPosts, Group, SourceLink, UrlFor,
};
use crate::{re, server, tera_filter};
use chrono::Datelike;
use core::str;
use fs_extra::dir::{copy as dircopy, CopyOptions};
use glob::glob;
use hotwatch::{Event, EventKind, Hotwatch};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::{fs, process, sync::Arc, sync::Mutex};
use tera::Value;
use tera::{Context, Tera};
use walkdir::WalkDir;

#[derive(Serialize, Clone, Debug, Default)]
pub struct UrlCollection {
    pub posts: Vec<String>,
    pub pages: Vec<String>,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub series: Vec<String>,
    pub streams: Vec<String>,
    pub archives: Vec<String>,
    pub languages: Vec<String>,
    pub feeds: Vec<String>,
    pub pagination: Vec<String>,
    pub file_mappings: Vec<String>,
    pub redirects: Vec<String>,
    pub misc: Vec<String>, // For other generated files
}

#[derive(Serialize, Clone, Debug)]
pub struct Data {
    pub site: Marmite,
    pub posts: Vec<Content>,
    pub pages: Vec<Content>,
    pub tag: GroupedContent,
    pub archive: GroupedContent,
    pub author: GroupedContent,
    pub stream: GroupedContent,
    pub series: GroupedContent,
    pub language: GroupedContent,
    pub latest_timestamp: Option<i64>,
    pub config_path: String,
    pub force_render: bool,
    pub generated_urls: UrlCollection,
    pub galleries: HashMap<String, Gallery>,
}

impl Data {
    pub fn new(config_content: &str, config_path: &Path) -> Self {
        let site: Marmite = match serde_yaml::from_str::<Marmite>(config_content) {
            Ok(site) => site,
            Err(e) => {
                error!("Failed to parse config YAML: {e:?}");
                process::exit(1);
            }
        };

        Data {
            site,
            posts: Vec::new(),
            pages: Vec::new(),
            tag: GroupedContent::new(Kind::Tag),
            archive: GroupedContent::new(Kind::Archive),
            author: GroupedContent::new(Kind::Author),
            stream: GroupedContent::new(Kind::Stream),
            series: GroupedContent::new(Kind::Series),
            language: GroupedContent::new(Kind::Language),
            latest_timestamp: None,
            config_path: config_path.to_string_lossy().to_string(),
            force_render: false,
            generated_urls: UrlCollection::default(),
            galleries: HashMap::new(),
        }
    }

    pub fn from_file(config_path: &Path) -> Self {
        let config_str = fs::read_to_string(config_path).unwrap_or_else(|e| {
            info!(
                "Unable to read '{}', assuming defaults.: {e:?}",
                &config_path.display()
            );
            String::new()
        });

        if config_str.is_empty() {
            info!("Config loaded from: defaults");
        } else {
            info!("Config loaded from: {}", config_path.display());
        }

        Self::new(&config_str, config_path)
    }

    pub fn sort_all(&mut self) {
        self.posts.sort_by_key(|a| std::cmp::Reverse(a.date));
        self.pages.sort_by(|a, b| b.title.cmp(&a.title));
        self.tag.sort_all();
        self.archive.sort_all();
        self.author.sort_all();
        self.stream.sort_all();
        self.series.sort_all();
        self.language.sort_all();
    }

    /// takes content then classifies the content
    /// into posts, pages, tags, authors, archive, stream
    /// and adds the content to the respective fields in self
    pub fn push_content(&mut self, content: Content) {
        if let Some(date) = content.date {
            self.posts.push(content.clone());
            // tags
            for tag in content.tags.clone() {
                let tag_slug = crate::slugify::slugify(&tag);
                // Store under slugified key (primary, used for URLs and new templates)
                self.tag
                    .entry(tag_slug.clone())
                    .or_default()
                    .push(content.clone());

                // BACKWARD COMPATIBILITY: Also store under original tag name
                // This allows old templates using site_data.tag.map[tag] to keep working
                // even when tag contains special characters like "Comunicação"
                if tag != tag_slug {
                    self.tag
                        .entry(tag.clone())
                        .or_default()
                        .push(content.clone());
                }
            }
            // authors
            for username in content.authors.clone() {
                self.author
                    .entry(username)
                    .or_default()
                    .push(content.clone());
            }
            // archive by year
            let year = date.year().to_string();
            self.archive.entry(year).or_default().push(content.clone());
            // stream by name
            if let Some(stream) = &content.stream {
                self.stream
                    .entry(stream.clone())
                    .or_default()
                    .push(content.clone());
            }

            // series by name
            if let Some(series) = &content.series {
                self.series
                    .entry(series.clone())
                    .or_default()
                    .push(content.clone());
            }
        } else {
            self.pages.push(content);
        }
    }

    /// Collect all generated URLs based on the site structure
    #[allow(clippy::too_many_lines)]
    pub fn collect_all_urls(&mut self) {
        // Clear existing URL collection
        self.generated_urls = UrlCollection::default();

        // Add homepage
        self.generated_urls
            .add_url("misc", "index.html".to_string());

        // Add all posts
        for post in &self.posts {
            self.generated_urls
                .add_url("posts", format!("{}.html", post.slug));
        }

        // Add all pages
        for page in &self.pages {
            self.generated_urls
                .add_url("pages", format!("{}.html", page.slug));
        }

        // Add pages listing and pagination
        if !self.pages.is_empty() {
            self.generated_urls
                .add_url("pages", "pages.html".to_string());

            // Add pagination for pages listing
            let content_count = self.pages.len();
            // Always add -1 page (same as base page but with consistent naming)
            self.generated_urls
                .add_url("pagination", "pages-1.html".to_string());

            // Add additional pagination pages if content exceeds pagination limit
            if content_count > self.site.pagination {
                let total_pages = content_count.div_ceil(self.site.pagination);
                for page_num in 2..=total_pages {
                    let pagination_slug = format!("pages-{page_num}.html");
                    self.generated_urls.add_url("pagination", pagination_slug);
                }
            }
        }

        // Add tag pages and pagination
        // Filter to only process slugified keys (avoid duplicates from backward compatibility layer)
        for tag in self
            .tag
            .iter()
            .filter(|(key, _)| crate::slugify::slugify(key) == key.as_str())
        {
            let tag_slug = format!("tag-{}.html", crate::slugify::slugify(tag.0));
            self.generated_urls.add_url("tags", tag_slug);

            // Add pagination for tags
            let content_count = tag.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 = format!("tag-{}-1.html", crate::slugify::slugify(tag.0));
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug =
                            format!("tag-{}-{}.html", crate::slugify::slugify(tag.0), page_num);
                        self.generated_urls.add_url("pagination", pagination_slug);
                    }
                }
            }
        }
        if !self.tag.map.is_empty() {
            self.generated_urls.add_url("tags", "tags.html".to_string());
        }

        // Add author pages and pagination
        for author in self.author.iter() {
            let author_slug = format!("author-{}.html", crate::slugify::slugify(author.0));
            self.generated_urls.add_url("authors", author_slug);

            // Add pagination for authors
            let content_count = author.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 =
                    format!("author-{}-1.html", crate::slugify::slugify(author.0));
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug = format!(
                            "author-{}-{}.html",
                            crate::slugify::slugify(author.0),
                            page_num
                        );
                        self.generated_urls.add_url("pagination", pagination_slug);
                    }
                }
            }
        }
        if !self.author.map.is_empty() {
            self.generated_urls
                .add_url("authors", "authors.html".to_string());
        }

        // Add series pages and pagination
        for series in self.series.iter() {
            let series_slug = format!("series-{}.html", crate::slugify::slugify(series.0));
            self.generated_urls.add_url("series", series_slug);

            // Add pagination for series
            let content_count = series.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 =
                    format!("series-{}-1.html", crate::slugify::slugify(series.0));
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug = format!(
                            "series-{}-{}.html",
                            crate::slugify::slugify(series.0),
                            page_num
                        );
                        self.generated_urls.add_url("pagination", pagination_slug);
                    }
                }
            }
        }
        if !self.series.map.is_empty() {
            self.generated_urls
                .add_url("series", "series.html".to_string());
        }

        // Add stream pages and pagination
        for stream in self.stream.iter() {
            // Skip "index" stream as it's handled separately as the main index
            if stream.0 != "index" {
                let stream_slug = format!("{}.html", crate::slugify::slugify(stream.0));
                self.generated_urls.add_url("streams", stream_slug);
            }

            // Add pagination for streams (skip "index" stream as it's handled separately)
            if stream.0 != "index" {
                let content_count = stream.1.len();
                if content_count > 0 {
                    // Always add -1 page (same as base page but with consistent naming)
                    let pagination_slug_1 = format!("{}-1.html", crate::slugify::slugify(stream.0));
                    self.generated_urls.add_url("pagination", pagination_slug_1);

                    // Add additional pagination pages if content exceeds pagination limit
                    if content_count > self.site.pagination {
                        let total_pages = content_count.div_ceil(self.site.pagination);
                        for page_num in 2..=total_pages {
                            let pagination_slug =
                                format!("{}-{}.html", crate::slugify::slugify(stream.0), page_num);
                            self.generated_urls.add_url("pagination", pagination_slug);
                        }
                    }
                }
            }
        }
        if !self.stream.map.is_empty() {
            self.generated_urls
                .add_url("streams", "streams.html".to_string());
        }

        // Add archive pages and pagination
        for archive in self.archive.iter() {
            let archive_slug = format!("archive-{}.html", archive.0);
            self.generated_urls.add_url("archives", archive_slug);

            // Add pagination for archives
            let content_count = archive.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 = format!("archive-{}-1.html", archive.0);
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug = format!("archive-{}-{}.html", archive.0, page_num);
                        self.generated_urls.add_url("pagination", pagination_slug);
                    }
                }
            }
        }
        if !self.archive.map.is_empty() {
            self.generated_urls
                .add_url("archives", "archive.html".to_string());
        }

        // Add languages group page (always rendered)
        self.generated_urls
            .add_url("languages", "languages.html".to_string());

        // Add main index pagination
        let posts_count = self.posts.len();
        if posts_count > self.site.pagination {
            let total_pages = posts_count.div_ceil(self.site.pagination);
            // Add -1 page (same as base page but with consistent naming)
            self.generated_urls
                .add_url("pagination", "index-1.html".to_string());
            // Add remaining pagination pages
            for page_num in 2..=total_pages {
                let pagination_slug = format!("index-{page_num}.html");
                self.generated_urls.add_url("pagination", pagination_slug);
            }
        }

        // Add RSS feeds (always enabled)
        {
            // Stream feeds (includes index stream which covers main index feed)
            for stream in self.stream.iter() {
                let feed_slug = format!("{}.rss", crate::slugify::slugify(stream.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Series feeds
            for series in self.series.iter() {
                let feed_slug = format!("series-{}.rss", crate::slugify::slugify(series.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Tag feeds
            // Filter to only process slugified keys (avoid duplicates from backward compatibility layer)
            for tag in self
                .tag
                .iter()
                .filter(|(key, _)| crate::slugify::slugify(key) == key.as_str())
            {
                let feed_slug = format!("tag-{}.rss", crate::slugify::slugify(tag.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Author feeds
            for author in self.author.iter() {
                let feed_slug = format!("author-{}.rss", crate::slugify::slugify(author.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Archive feeds
            for archive in self.archive.iter() {
                let feed_slug = format!("archive-{}.rss", archive.0);
                self.generated_urls.add_url("feeds", feed_slug);
            }
        }

        // Add JSON feeds
        if self.site.json_feed {
            // Stream feeds (includes index stream which covers main index feed)
            for stream in self.stream.iter() {
                let feed_slug = format!("{}.json", crate::slugify::slugify(stream.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Series feeds
            for series in self.series.iter() {
                let feed_slug = format!("series-{}.json", crate::slugify::slugify(series.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Tag feeds
            // Filter to only process slugified keys (avoid duplicates from backward compatibility layer)
            for tag in self
                .tag
                .iter()
                .filter(|(key, _)| crate::slugify::slugify(key) == key.as_str())
            {
                let feed_slug = format!("tag-{}.json", crate::slugify::slugify(tag.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Author feeds
            for author in self.author.iter() {
                let feed_slug = format!("author-{}.json", crate::slugify::slugify(author.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Archive feeds
            for archive in self.archive.iter() {
                let feed_slug = format!("archive-{}.json", archive.0);
                self.generated_urls.add_url("feeds", feed_slug);
            }
        }

        // Add file mappings if they exist
        for mapping in &self.site.file_mapping {
            let destination = if mapping.dest.starts_with('/') {
                mapping.dest[1..].to_string()
            } else {
                mapping.dest.clone()
            };
            self.generated_urls.add_url("file_mappings", destination);
        }

        // Add redirect aliases
        for content in self.posts.iter().chain(&self.pages) {
            for alias in &content.aliases {
                self.generated_urls
                    .add_url("redirects", format!("{alias}.html"));
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ContentInfo {
    title: String,
    slug: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    series: Option<String>,
    pinned: bool,
}

impl ContentInfo {
    fn from_content(content: &Content) -> Self {
        Self {
            title: content.title.clone(),
            slug: content.slug.clone(),
            url: format!("/{}.html", content.slug),
            source_path: content
                .source_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            date: content.date.map(|d| d.format("%Y-%m-%d").to_string()),
            tags: content.tags.clone(),
            authors: content.authors.clone(),
            description: content.description.clone(),
            stream: content.stream.clone(),
            series: content.series.clone(),
            pinned: content.pinned,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct BuildInfo {
    marmite_version: String,
    posts: Vec<ContentInfo>,
    pages: Vec<ContentInfo>,
    shortcodes: Vec<String>,
    generated_at: String,
    timestamp: i64,
    elapsed_time: f64,
    config: Marmite,
}

impl BuildInfo {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let build_info: Self = serde_json::from_str(json)?;
        Ok(build_info)
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn build_site_with_config(
    site_config: &Marmite,
    input_folder: &Path,
    output_folder: &Path,
    cli_args: &Arc<crate::cli::Cli>,
    cross_site_data: Option<&crate::workspace::CrossSiteData>,
    path_prefix: &str,
) -> Result<Data, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    let config_str = serde_yaml::to_string(&site_config).unwrap_or_default();
    let config_path = input_folder.join(&cli_args.config);
    let mut site_data = Data::new(&config_str, &config_path);
    let content_folder = get_content_folder(&site_data.site, input_folder);

    let build_info_path = output_folder.join("marmite.json");
    let latest_build_info = get_latest_build_info(&build_info_path)?;
    if let Some(build_info) = &latest_build_info {
        site_data.latest_timestamp = Some(build_info.timestamp);
    }
    if cli_args.force {
        site_data.force_render = true;
    }

    let highlighter = build_code_highlighter(&site_data.site);

    let fragments = collect_content_fragments(&content_folder);
    let folder_defaults = load_folder_frontmatter(&content_folder);
    collect_content(
        &content_folder,
        &mut site_data,
        &fragments,
        highlighter.as_deref(),
        &folder_defaults,
    );

    discover_translations(&mut site_data, &content_folder);
    rebuild_stream_index(&mut site_data);
    build_language_index(&mut site_data);

    let state_path = input_folder.join(".marmite-atproto-state.json");
    if state_path.exists() {
        if let Ok(state_str) = fs::read_to_string(&state_path) {
            if let Ok(state_json) = serde_json::from_str::<serde_json::Value>(&state_str) {
                if let Some(posts_obj) = state_json.get("posts").and_then(|p| p.as_object()) {
                    for post in &mut site_data.posts {
                        if let Some(entry) = posts_obj.get(&post.slug) {
                            if let Some(at_uri) = entry.get("at_uri").and_then(|u| u.as_str()) {
                                post.at_uri = Some(at_uri.to_string());
                            }
                        }
                    }
                    for page in &mut site_data.pages {
                        if let Some(entry) = posts_obj.get(&page.slug) {
                            if let Some(at_uri) = entry.get("at_uri").and_then(|u| u.as_str()) {
                                page.at_uri = Some(at_uri.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let media_path = content_folder.join(&site_data.site.media_path);
    site_data.galleries = crate::gallery::process_galleries(
        &media_path,
        &site_data.site.gallery_path,
        site_data.site.gallery_create_thumbnails,
        site_data.site.gallery_thumb_size,
    );

    site_data.sort_all();
    detect_slug_collision(&site_data);
    collect_back_links(&mut site_data);
    set_next_and_previous_links(&mut site_data);
    site_data.collect_all_urls();

    if site_data.site.check_internal_links {
        let broken = validate_internal_links(&site_data);
        for (source, target) in &broken {
            warn!("Broken internal link in \"{source}.html\": \"{target}.html\" does not exist");
        }
        if !broken.is_empty() {
            warn!("Found {} broken internal link(s)", broken.len());
            if site_data.site.strict_internal_links {
                error!(
                    "Build failed due to broken internal links (strict_internal_links is enabled)"
                );
                process::exit(1);
            }
        }
    }

    let site_path = site_data.site.site_path.clone();
    let output_path = output_folder.join(site_path);
    if let Err(e) = fs::create_dir_all(&output_path) {
        error!("Unable to create output directory: {e:?}");
        process::exit(1);
    }

    let input_folder_arc = Arc::new(input_folder.to_path_buf());
    let output_folder_arc = Arc::new(output_folder.to_path_buf());

    [
        "render_templates",
        "handle_static_artifacts",
        "generate_search_index",
        "copy_markdown_sources",
    ]
    .par_iter()
    .for_each(|step| match *step {
        "render_templates" => {
            let (tera, shortcode_processor) = initialize_tera(
                input_folder_arc.as_path(),
                &site_data,
                cross_site_data,
                path_prefix,
            );
            if let Err(e) = render_templates(
                &content_folder,
                &site_data,
                &tera,
                &output_path,
                input_folder_arc.as_path(),
                &fragments,
                latest_build_info.as_ref(),
                shortcode_processor.as_ref(),
                highlighter.as_deref(),
                cross_site_data,
                false,
            ) {
                error!("Failed to render templates: {e:?}");
                process::exit(1);
            }
        }
        "handle_static_artifacts" => {
            handle_static_artifacts(
                input_folder_arc.as_path(),
                &site_data,
                &output_folder_arc,
                &content_folder,
            );
        }
        "generate_search_index" => {
            if site_data.site.enable_search {
                generate_search_index(&site_data, &output_folder_arc);
            }
        }
        "copy_markdown_sources" if site_data.site.publish_md => {
            copy_markdown_sources(&site_data, &content_folder, &output_path);
        }
        _ => {}
    });

    let (tera, _) = initialize_tera(input_folder, &site_data, cross_site_data, path_prefix);
    generate_sitemap(&site_data, &tera, &output_path);

    if site_data.site.publish_urls_json {
        generate_urls_json(&site_data, &output_path, path_prefix);
    }

    if let Some(atproto) = &site_data.site.atproto {
        if let Some(pub_uri) = &atproto.publication_uri {
            let wk_dir = output_path.join(".well-known");
            if let Err(e) = fs::create_dir_all(&wk_dir) {
                error!("Failed to create .well-known directory: {e:?}");
            } else {
                let wk_file = wk_dir.join("site.standard.publication");
                if let Err(e) = fs::write(&wk_file, pub_uri) {
                    error!("Failed to write site.standard.publication verification file: {e:?}");
                } else {
                    info!("Generated /.well-known/site.standard.publication verification file");
                }
            }
        }
    }

    let end_time = start_time.elapsed().as_secs_f64();
    write_build_info(&output_path, &site_data, input_folder, end_time);
    debug!("Site generated in {end_time:.2}s");
    info!("Site generated at: {}/", output_folder.display());
    Ok(site_data)
}

#[allow(clippy::too_many_lines)]
pub fn generate(
    config_path: &Arc<std::path::PathBuf>,
    input_folder: &Arc<std::path::PathBuf>,
    output_folder: &Arc<std::path::PathBuf>,
    watch: bool,
    serve: bool,
    bind_address: &str,
    cli_args: &Arc<crate::cli::Cli>,
) -> Result<(), Box<dyn std::error::Error>> {
    let moved_input_folder = Arc::clone(input_folder);
    let moved_output_folder = Arc::clone(output_folder);
    let moved_config_path = Arc::clone(config_path);
    let moved_cli_args = Arc::clone(cli_args);

    let live_reload = if watch && serve {
        Some(server::LiveReload::new())
    } else {
        None
    };

    let rebuild = {
        move || -> Result<(), Box<dyn std::error::Error>> {
            let start_time = std::time::Instant::now();
            let site_data = Arc::new(Mutex::new(Data::from_file(
                moved_config_path.clone().as_path(),
            )));
            let content_folder = get_content_folder(
                &site_data
                    .lock()
                    .unwrap_or_else(|e| {
                        error!("Failed to lock site data: {e}");
                        panic!("Cannot proceed without site data lock")
                    })
                    .site,
                moved_input_folder.clone().as_path(),
            );
            let mut site_data = site_data.lock().unwrap_or_else(|e| {
                error!("Failed to lock site data: {e}");
                panic!("Cannot proceed without site data lock")
            });
            if serve {
                site_data
                    .site
                    .markdown_parser
                    .get_or_insert_with(Default::default)
                    .render
                    .sourcepos = true;
            }
            let build_info_path = moved_output_folder.join("marmite.json");
            let latest_build_info = get_latest_build_info(&build_info_path)?;
            if let Some(build_info) = &latest_build_info {
                site_data.latest_timestamp = Some(build_info.timestamp);
            }
            site_data.site.override_from_cli_args(&moved_cli_args);
            if moved_cli_args.force {
                site_data.force_render = true;
            }

            let highlighter = build_code_highlighter(&site_data.site);

            let fragments = collect_content_fragments(&content_folder);
            let folder_defaults = load_folder_frontmatter(&content_folder);
            collect_content(
                &content_folder,
                &mut site_data,
                &fragments,
                highlighter.as_deref(),
                &folder_defaults,
            );

            discover_translations(&mut site_data, &content_folder);

            // Rebuild stream and language indexes after discover_translations may have changed them
            rebuild_stream_index(&mut site_data);
            build_language_index(&mut site_data);

            // Load atproto state to populate at_uri on matching posts/pages
            let state_path = moved_input_folder.join(".marmite-atproto-state.json");
            if state_path.exists() {
                if let Ok(state_str) = fs::read_to_string(&state_path) {
                    if let Ok(state_json) = serde_json::from_str::<serde_json::Value>(&state_str) {
                        if let Some(posts_obj) = state_json.get("posts").and_then(|p| p.as_object())
                        {
                            for post in &mut site_data.posts {
                                if let Some(entry) = posts_obj.get(&post.slug) {
                                    if let Some(at_uri) =
                                        entry.get("at_uri").and_then(|u| u.as_str())
                                    {
                                        post.at_uri = Some(at_uri.to_string());
                                    }
                                }
                            }
                            for page in &mut site_data.pages {
                                if let Some(entry) = posts_obj.get(&page.slug) {
                                    if let Some(at_uri) =
                                        entry.get("at_uri").and_then(|u| u.as_str())
                                    {
                                        page.at_uri = Some(at_uri.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Process galleries
            let media_path = content_folder.join(&site_data.site.media_path);
            site_data.galleries = crate::gallery::process_galleries(
                &media_path,
                &site_data.site.gallery_path,
                site_data.site.gallery_create_thumbnails,
                site_data.site.gallery_thumb_size,
            );

            site_data.sort_all();
            detect_slug_collision(&site_data); // Detect slug collision and warn user
            collect_back_links(&mut site_data);
            set_next_and_previous_links(&mut site_data);
            site_data.collect_all_urls();

            if site_data.site.check_internal_links {
                let broken = validate_internal_links(&site_data);
                for (source, target) in &broken {
                    warn!(
                        "Broken internal link in \"{source}.html\": \"{target}.html\" does not exist"
                    );
                }
                if !broken.is_empty() {
                    warn!("Found {} broken internal link(s)", broken.len());
                    if site_data.site.strict_internal_links {
                        error!("Build failed due to broken internal links (strict_internal_links is enabled)");
                        process::exit(1);
                    }
                }
            }

            let site_path = site_data.site.site_path.clone();
            let output_path = moved_output_folder.join(site_path);
            if let Err(e) = fs::create_dir_all(&output_path) {
                error!("Unable to create output directory: {e:?}");
                process::exit(1);
            }

            [
                "render_templates",
                "handle_static_artifacts",
                "generate_search_index",
                "copy_markdown_sources",
            ]
            .par_iter()
            .for_each(|step| match *step {
                "render_templates" => {
                    let (tera, shortcode_processor) =
                        initialize_tera(&moved_input_folder, &site_data, None, "");
                    if let Err(e) = render_templates(
                        &content_folder,
                        &site_data,
                        &tera,
                        &output_path,
                        &moved_input_folder,
                        &fragments,
                        latest_build_info.as_ref(),
                        shortcode_processor.as_ref(),
                        highlighter.as_deref(),
                        None,
                        serve,
                    ) {
                        error!("Failed to render templates: {e:?}");
                        if !serve {
                            process::exit(1);
                        }
                    }
                }
                "handle_static_artifacts" => {
                    handle_static_artifacts(
                        &moved_input_folder,
                        &site_data,
                        &moved_output_folder,
                        &content_folder,
                    );
                }
                "generate_search_index" => {
                    if site_data.site.enable_search {
                        generate_search_index(&site_data, &moved_output_folder);
                    }
                }
                "copy_markdown_sources" if site_data.site.publish_md => {
                    copy_markdown_sources(&site_data, &content_folder, &output_path);
                }
                _ => {}
            });

            // Generate sitemap after all templates are rendered
            let (tera, _) = initialize_tera(&moved_input_folder, &site_data, None, "");
            generate_sitemap(&site_data, &tera, &output_path);

            // Generate urls.json if enabled
            if site_data.site.publish_urls_json {
                generate_urls_json(&site_data, &output_path, "");
            }

            // Generate standard.site verification file if configured
            if let Some(atproto) = &site_data.site.atproto {
                if let Some(pub_uri) = &atproto.publication_uri {
                    let wk_dir = output_path.join(".well-known");
                    if let Err(e) = fs::create_dir_all(&wk_dir) {
                        error!("Failed to create .well-known directory: {e:?}");
                    } else {
                        let wk_file = wk_dir.join("site.standard.publication");
                        if let Err(e) = fs::write(&wk_file, pub_uri) {
                            error!("Failed to write site.standard.publication verification file: {e:?}");
                        } else {
                            info!("Generated /.well-known/site.standard.publication verification file");
                        }
                    }
                }
            }

            let end_time = start_time.elapsed().as_secs_f64();
            write_build_info(&output_path, &site_data, &moved_input_folder, end_time);
            debug!("Site generated in {end_time:.2}s");
            info!("Site generated at: {}/", moved_output_folder.display());
            Ok(())
        }
    };

    // Initial site generation
    rebuild()?;

    if watch {
        let mut hotwatch = match Hotwatch::new() {
            Ok(hw) => hw,
            Err(e) => {
                error!("Failed to initialize hotwatch: {e}");
                return Ok(());
            }
        };
        let watch_folder = Arc::clone(input_folder).as_path().to_path_buf();
        let out_folder = Arc::clone(output_folder).as_path().to_path_buf();
        // Watch the input folder for changes
        let live_reload_watch = live_reload.clone();
        let watch_result = hotwatch.watch(watch_folder, move |event: Event| match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for ev in &event.paths {
                    if !ev.starts_with(
                        fs::canonicalize(out_folder.clone()).unwrap_or_else(|_| out_folder.clone()),
                    ) {
                        info!("Change detected. Rebuilding site...");
                        if let Err(e) = rebuild() {
                            error!("Failed to rebuild site: {e}");
                        } else if let Some(live_reload) = &live_reload_watch {
                            live_reload.notify_reload();
                        }
                    }
                }
            }
            _ => {}
        });
        if let Err(e) = watch_result {
            error!("Failed to watch the input folder: {e}");
            return Ok(());
        }

        info!("Watching for changes in folder: {}", input_folder.display());

        // Keep the thread alive for watching
        if serve {
            info!("Starting built-in HTTP server...");
            let serve_config = Data::from_file(config_path);
            let ctx = server::ServerContext {
                output_folder: Arc::clone(output_folder),
                input_folder: Arc::clone(input_folder),
                config_path: Arc::clone(config_path),
                enable_toolbar: serve_config.site.enable_toolbar,
                watch_enabled: true,
            };
            server::start(bind_address, &ctx, live_reload.as_ref());
        } else {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    Ok(())
}

fn get_latest_build_info(
    build_info_path: &std::path::PathBuf,
) -> Result<Option<BuildInfo>, std::io::Error> {
    if build_info_path.exists() {
        let build_info_json = fs::read_to_string(build_info_path)?;
        if let Ok(build_info) = BuildInfo::from_json(&build_info_json) {
            return Ok(Some(build_info));
        }
    }
    Ok(None)
}

/// Get the content folder from the config or default to the input folder
pub fn get_content_folder(config: &Marmite, input_folder: &Path) -> std::path::PathBuf {
    Some(input_folder.join(config.content_path.clone()))
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| input_folder.to_path_buf())
}

/// Collect markdown fragments that will merge into the content markdown before processing
/// These are static parts of text that will just be merged to the content
pub(crate) fn collect_content_fragments(content_dir: &Path) -> HashMap<String, String> {
    let markdown_fragments: HashMap<String, String> =
        ["markdown_header", "markdown_footer", "references"]
            .iter()
            .map(|fragment| {
                let fragment_path = content_dir.join(format!("_{fragment}.md"));
                let fragment_content = if fragment_path.exists() {
                    fs::read_to_string(&fragment_path).unwrap_or_else(|e| {
                        error!("Failed to read fragment {fragment}: {e}");
                        String::new()
                    })
                } else {
                    String::new()
                };
                ((*fragment).to_string(), fragment_content)
            })
            .collect();
    markdown_fragments
}

fn parse_frontmatter_yaml(content: &str) -> Option<frontmatter_gen::Frontmatter> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Some(frontmatter_gen::Frontmatter::new());
    }
    frontmatter_gen::parse(trimmed, frontmatter_gen::Format::Yaml).ok()
}

pub(crate) fn load_folder_frontmatter(
    content_dir: &Path,
) -> HashMap<std::path::PathBuf, frontmatter_gen::Frontmatter> {
    let mut folder_defaults: HashMap<std::path::PathBuf, frontmatter_gen::Frontmatter> =
        HashMap::new();

    let mut fm_paths: Vec<std::path::PathBuf> = WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .file_name()
                    .is_some_and(|n| n == "frontmatter.yaml")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    fm_paths.sort_by_key(|p| p.components().count());

    for fm_path in fm_paths {
        let folder = match fm_path.parent() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        match fs::read_to_string(&fm_path) {
            Ok(content) => match parse_frontmatter_yaml(&content) {
                Some(mut fm) => {
                    let mut ancestor = folder.parent();
                    while let Some(dir) = ancestor {
                        if !dir.starts_with(content_dir) {
                            break;
                        }
                        if let Some(parent_fm) = folder_defaults.get(dir) {
                            merge_frontmatter(parent_fm, &mut fm);
                            break;
                        }
                        ancestor = dir.parent();
                    }
                    debug!("Loaded frontmatter.yaml for {}", folder.display());
                    folder_defaults.insert(folder, fm);
                }
                None => {
                    warn!("Failed to parse {}", fm_path.display());
                }
            },
            Err(e) => {
                warn!("Failed to read {}: {e}", fm_path.display());
            }
        }
    }

    folder_defaults
}

/// Collect global fragments of markdown, process them and insert into the global context
/// these are dynamic parts of text that will be processed by Tera
fn collect_global_fragments(
    content_dir: &Path,
    global_context: &mut Context,
    tera: &Tera,
    site_config: &Marmite,
    highlighter: Option<&MarmiteHighlighter>,
) {
    let fragments = [
        "announce", "header", "hero", "sidebar", "footer", "comments", "htmlhead", "htmltail",
    ]
    .par_iter()
    .filter(|fragment| {
        let fragment_path = content_dir.join(format!("_{fragment}.md"));
        fragment_path.exists()
    })
    .map(|fragment| {
        let fragment_path = content_dir.join(format!("_{fragment}.md"));
        let fragment_content = fs::read_to_string(&fragment_path).unwrap_or_else(|e| {
            error!("Failed to read fragment {fragment}: {e}");
            String::new()
        });
        // append references
        let references_path = content_dir.join("_references.md");
        let fragment_content =
            crate::parser::append_references(&fragment_content, &references_path);
        let rendered_fragment = tera
            .clone()
            .render_str(&fragment_content, global_context, false)
            .unwrap_or_else(|e| {
                error!("Failed to render fragment {fragment}: {e}");
                fragment_content.clone()
            });
        let default_parser_options = crate::config::ParserOptions::default();
        let parser_options = site_config
            .markdown_parser
            .as_ref()
            .unwrap_or(&default_parser_options);
        let fragment_content =
            crate::parser::get_html_with_options(&rendered_fragment, parser_options, highlighter);
        // global_context.insert((*fragment).to_string(), &fragment_content);
        debug!("{} fragment {}", fragment, &fragment_content);
        ((*fragment).to_string(), fragment_content)
    })
    .collect::<Vec<_>>();
    for (name, content) in fragments {
        global_context.insert(name, &content);
    }
}

#[allow(clippy::used_underscore_items)]
fn collect_back_links(site_data: &mut Data) {
    let other_contents = site_data
        .posts
        .clone()
        .iter()
        .chain(&site_data.pages.clone())
        .map(std::borrow::ToOwned::to_owned)
        .collect::<Vec<Content>>();

    _collect_back_links(&mut site_data.posts, &other_contents);
    _collect_back_links(&mut site_data.pages, &other_contents);
}

#[allow(clippy::needless_range_loop)]
fn _collect_back_links(contents: &mut [Content], other_contents: &[Content]) {
    for content in contents.iter_mut() {
        content.back_links.clear();
    }
    for i in 0..contents.len() {
        let slug = contents[i].slug.clone();
        for other_content in other_contents {
            if let Some(ref links_to) = other_content.links_to {
                // if content and other_content has the same slug skip
                if links_to.contains(&slug) && slug != other_content.slug {
                    contents[i].back_links.push(other_content.clone());
                }
            }
        }
    }
}

fn validate_internal_links(site_data: &Data) -> Vec<(String, String)> {
    let mut valid_slugs: std::collections::HashSet<String> = std::collections::HashSet::new();

    let all_urls = site_data.generated_urls.get_all_urls();
    for url in &all_urls {
        valid_slugs.insert(url.trim_end_matches(".html").to_string());
    }
    for url in &site_data.generated_urls.redirects {
        valid_slugs.insert(url.trim_end_matches(".html").to_string());
    }

    // Build a set of slugified titles so wikilinks that will be resolved
    // by fix_wikilinks at render time are not reported as broken.
    // Include both normal slugification and a variant with "&" -> "amp"
    // because comrak encodes "&" as "&amp;" in HTML which then slugifies
    // to "amp" in the auto-generated wikilink href.
    let mut title_slugs: std::collections::HashSet<String> = std::collections::HashSet::new();
    for content in site_data.posts.iter().chain(&site_data.pages) {
        title_slugs.insert(crate::slugify::slugify(&content.title));
        title_slugs.insert(crate::slugify::slugify(content.title.replace('&', "amp")));
    }

    let mut broken = Vec::new();
    for content in site_data.posts.iter().chain(&site_data.pages) {
        if let Some(ref links) = content.links_to {
            for link in links {
                let slug = link.split('#').next().unwrap_or(link);
                if !valid_slugs.contains(slug) && !title_slugs.contains(slug) {
                    broken.push((content.slug.clone(), slug.to_string()));
                }
            }
        }
    }
    broken
}

#[allow(clippy::cast_possible_wrap)]
fn set_next_and_previous_links(site_data: &mut Data) {
    // First, handle series navigation (takes precedence over stream navigation)
    let mut series_posts: HashMap<String, Vec<Content>> = HashMap::new();
    for post in &site_data.posts {
        if let Some(series_name) = &post.series {
            series_posts
                .entry(series_name.clone())
                .or_default()
                .push(post.clone());
        }
    }

    // Sort series posts chronologically (oldest to newest)
    for posts in series_posts.values_mut() {
        posts.sort_by_key(|a| a.date);
    }

    // Set next/previous for posts in series
    for posts in series_posts.values() {
        for i in 0..posts.len() {
            let current_slug = &posts[i].slug;

            let previous = if i > 0 {
                Some(Box::new(posts[i - 1].clone()))
            } else {
                None
            };

            let next = if i < posts.len() - 1 {
                Some(Box::new(posts[i + 1].clone()))
            } else {
                None
            };

            if let Some(content) = site_data.posts.iter_mut().find(|c| c.slug == *current_slug) {
                content.previous = previous;
                content.next = next;
            }
        }
    }

    // Then handle stream navigation for posts NOT in a series
    let mut stream_posts: HashMap<String, Vec<Content>> = HashMap::new();
    for post in &site_data.posts {
        // Only include posts that are NOT in a series
        if post.series.is_none() {
            if let Some(stream_name) = &post.stream {
                stream_posts
                    .entry(stream_name.clone())
                    .or_default()
                    .push(post.clone());
            }
        }
    }

    for (_, posts) in stream_posts {
        for i in 0..posts.len() {
            let current_slug = &posts[i].slug;

            let previous = if i < posts.len() - 1 {
                Some(Box::new(posts[i + 1].clone()))
            } else {
                None
            };

            let next = if i > 0 {
                Some(Box::new(posts[i - 1].clone()))
            } else {
                None
            };

            if let Some(content) = site_data.posts.iter_mut().find(|c| c.slug == *current_slug) {
                content.previous = previous;
                content.next = next;
            }
        }
    }
}

#[allow(clippy::cast_possible_wrap, clippy::too_many_lines)]
pub(crate) fn collect_content(
    content_dir: &std::path::PathBuf,
    site_data: &mut Data,
    fragments: &HashMap<String, String>,
    highlighter: Option<&MarmiteHighlighter>,
    folder_defaults: &HashMap<std::path::PathBuf, frontmatter_gen::Frontmatter>,
) {
    let contents = WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .into_par_iter()
        .filter(|e| {
            let file_name = e
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(
                    e.path()
                        .to_str()
                        .unwrap_or_else(|| panic!("Could not get file name {e:?}")),
                );
            let file_extension = e.path().extension().and_then(|ext| ext.to_str());
            if !(e.path().is_file() && file_extension == Some("md") && !file_name.starts_with('_'))
            {
                return false;
            }

            true
        })
        .map(|entry| {
            let file_metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(e) => {
                    error!(
                        "Failed to get file metadata for {}: {}",
                        entry.path().display(),
                        e
                    );
                    return Err(format!("Failed to get file metadata: {e}"));
                }
            };
            let modified_time = if let Ok(modified_time) = file_metadata.modified() {
                Some(
                    modified_time
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_else(|e| {
                            error!("Failed to get duration since UNIX_EPOCH: {e}");
                            std::time::Duration::ZERO
                        })
                        .as_secs() as i64,
                )
            } else {
                None
            };
            let defaults = {
                let mut dir = entry.path().parent();
                loop {
                    match dir {
                        Some(d) if d.starts_with(content_dir.as_path()) => {
                            if let Some(fm) = folder_defaults.get(d) {
                                break Some(fm);
                            }
                            dir = d.parent();
                        }
                        _ => break None,
                    }
                }
            };
            Content::from_markdown(
                entry.path(),
                Some(fragments),
                &site_data.site,
                modified_time,
                highlighter,
                defaults,
                Some(content_dir),
            )
        })
        .collect::<Vec<_>>();
    for content in contents {
        match content {
            Ok(mut content) => {
                // Language-from-path detection is deferred to discover_translations
                // Pass 3, where it only applies inside validated translation groups.

                if let Some(ref lang) = content.language {
                    if content.stream.as_deref() == Some("index")
                        && *lang != site_data.site.language
                    {
                        content.stream = Some(lang.clone());
                        let prefix = format!("{lang}-");
                        if !content.slug.starts_with(&prefix) {
                            content.slug = format!("{lang}-{}", content.slug);
                        }
                    }
                }

                if content.language.is_none() {
                    if let Some(ref stream) = content.stream {
                        if is_iso_639_1_code(stream) {
                            content.language = Some(stream.clone());
                        } else if stream == "index" {
                            content.language = Some(site_data.site.language.clone());
                        }
                    }
                }

                site_data.push_content(content);
            }
            Err(e) => {
                error!("Failed to process content: {e:?}");
            }
        }
    }

    // Auto-populate languages from observed content
    let mut observed_languages: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for content in site_data.posts.iter().chain(&site_data.pages) {
        if let Some(ref lang) = content.language {
            if *lang != site_data.site.language {
                observed_languages.insert(lang.clone());
            }
        }
    }
    for lang in observed_languages {
        site_data
            .site
            .languages
            .entry(lang.clone())
            .or_insert_with(|| crate::config::LanguageConfig { display_name: lang });
    }
}

#[allow(clippy::too_many_lines)]
fn rebuild_stream_index(site_data: &mut Data) {
    let posts: Vec<_> = site_data.posts.clone();
    site_data.stream = GroupedContent::new(Kind::Stream);
    for post in &posts {
        if let Some(stream) = &post.stream {
            site_data
                .stream
                .entry(stream.clone())
                .or_default()
                .push(post.clone());
        }
    }
    site_data.stream.sort_all();
}

pub fn build_language_index(site_data: &mut Data) {
    let default_language = site_data.site.language.clone();
    let posts: Vec<_> = site_data.posts.clone();
    site_data.language = GroupedContent::new(Kind::Language);
    for post in &posts {
        let lang = post
            .language
            .as_deref()
            .unwrap_or(&default_language)
            .to_string();
        site_data
            .language
            .entry(lang)
            .or_default()
            .push(post.clone());
    }
    site_data.language.sort_all();
}

fn discover_translations(site_data: &mut Data, content_dir: &Path) {
    if site_data.site.language.is_empty() {
        site_data.site.language = "en".to_string();
    }
    let default_language = site_data.site.language.clone();
    let languages = site_data.site.languages.clone();

    process_translates_frontmatter(site_data);
    infer_content_languages(site_data, &default_language);
    build_and_link_subfolder_translations(site_data, content_dir, &default_language, &languages);
    resolve_frontmatter_translations(site_data, &default_language, &languages);
}

fn process_translates_frontmatter(site_data: &mut Data) {
    let mut translates_refs: Vec<(usize, bool, String)> = Vec::new();
    for (i, post) in site_data.posts.iter().enumerate() {
        if let Some(ref target_slug) = post.translates {
            translates_refs.push((i, true, target_slug.clone()));
        }
    }
    for (i, page) in site_data.pages.iter().enumerate() {
        if let Some(ref target_slug) = page.translates {
            translates_refs.push((i, false, target_slug.clone()));
        }
    }
    for (idx, is_post, target_slug) in &translates_refs {
        let content = if *is_post {
            &mut site_data.posts[*idx]
        } else {
            &mut site_data.pages[*idx]
        };
        if !content
            .translations
            .iter()
            .any(|existing| existing.slug == *target_slug)
        {
            content.translations.push(TranslationRef {
                slug: target_slug.clone(),
                lang: String::new(),
                name: String::new(),
                title: String::new(),
            });
        }
    }
}

fn infer_content_languages(site_data: &mut Data, default_language: &str) {
    for post in &mut site_data.posts {
        if post.language.is_none() {
            if let Some(ref stream) = post.stream {
                if is_iso_639_1_code(stream) {
                    post.language = Some(stream.clone());
                } else if stream == "index" {
                    post.language = Some(default_language.to_string());
                }
            }
        }
    }

    for page in &mut site_data.pages {
        if page.language.is_none() {
            if let Some(ref source_path) = page.source_path {
                if let Some(filename) = source_path.file_stem().and_then(|s| s.to_str()) {
                    for &lang_code in ISO_639_1_CODES {
                        let prefix = format!("{lang_code}-");
                        if filename.starts_with(&prefix) {
                            page.language = Some(lang_code.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn build_subfolder_groups(
    site_data: &Data,
    content_dir: &Path,
) -> HashMap<String, Vec<(usize, bool)>> {
    let mut subfolder_groups: HashMap<String, Vec<(usize, bool)>> = HashMap::new();

    for (i, post) in site_data.posts.iter().enumerate() {
        if let Some(ref source_path) = post.source_path {
            if let Ok(relative) = source_path.strip_prefix(content_dir) {
                if let Some(parent) = relative.parent() {
                    let parent_str = parent.to_string_lossy();
                    if !parent_str.is_empty() {
                        subfolder_groups
                            .entry(parent_str.to_string())
                            .or_default()
                            .push((i, true));
                    }
                }
            }
        }
    }

    for (i, page) in site_data.pages.iter().enumerate() {
        if let Some(ref source_path) = page.source_path {
            if let Ok(relative) = source_path.strip_prefix(content_dir) {
                if let Some(parent) = relative.parent() {
                    let parent_str = parent.to_string_lossy();
                    if !parent_str.is_empty() {
                        subfolder_groups
                            .entry(parent_str.to_string())
                            .or_default()
                            .push((i, false));
                    }
                }
            }
        }
    }

    for (i, post) in site_data.posts.iter().enumerate() {
        if let Some(ref source_path) = post.source_path {
            if let Ok(relative) = source_path.strip_prefix(content_dir) {
                let components: Vec<_> = relative.components().collect();
                if components.len() == 1 && subfolder_groups.contains_key(&post.slug) {
                    let group = subfolder_groups.get_mut(&post.slug).unwrap();
                    if !group.iter().any(|&(idx, is_post)| is_post && idx == i) {
                        group.push((i, true));
                    }
                }
            }
        }
    }

    for (i, page) in site_data.pages.iter().enumerate() {
        if let Some(ref source_path) = page.source_path {
            if let Ok(relative) = source_path.strip_prefix(content_dir) {
                let components: Vec<_> = relative.components().collect();
                if components.len() == 1 && subfolder_groups.contains_key(&page.slug) {
                    let group = subfolder_groups.get_mut(&page.slug).unwrap();
                    if !group.iter().any(|&(idx, is_post)| !is_post && idx == i) {
                        group.push((i, false));
                    }
                }
            }
        }
    }

    subfolder_groups
}

fn is_translation_group(group: &[(usize, bool)], posts: &[Content], pages: &[Content]) -> bool {
    let has_lang_prefixed_file = group.iter().any(|&(idx, is_post)| {
        let content = if is_post { &posts[idx] } else { &pages[idx] };
        if let Some(ref source_path) = content.source_path {
            if let Some(stem) = source_path.file_stem().and_then(|s| s.to_str()) {
                return crate::content::ISO_639_1_CODES
                    .iter()
                    .any(|code| stem.starts_with(&format!("{code}-")));
            }
        }
        false
    });
    if !has_lang_prefixed_file {
        return false;
    }

    let non_prefixed_count = group
        .iter()
        .filter(|&&(idx, is_post)| {
            let content = if is_post { &posts[idx] } else { &pages[idx] };
            content
                .source_path
                .as_ref()
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
                .is_some_and(|stem| {
                    !crate::content::ISO_639_1_CODES
                        .iter()
                        .any(|code| stem.starts_with(&format!("{code}-")))
                })
        })
        .count();
    non_prefixed_count == 1
}

fn build_and_link_subfolder_translations(
    site_data: &mut Data,
    content_dir: &Path,
    default_language: &str,
    languages: &HashMap<String, LanguageConfig>,
) {
    let subfolder_groups = build_subfolder_groups(site_data, content_dir);
    let mut translation_links: Vec<(usize, bool, Vec<TranslationRef>)> = Vec::new();

    for group in subfolder_groups.values() {
        if group.len() < 2 {
            continue;
        }

        if !is_translation_group(group, &site_data.posts, &site_data.pages) {
            continue;
        }

        for &(idx, is_post) in group {
            let content = if is_post {
                &mut site_data.posts[idx]
            } else {
                &mut site_data.pages[idx]
            };
            if let Some(lang) = content
                .source_path
                .as_deref()
                .and_then(|p| detect_language_from_path(p, content_dir))
            {
                if content.stream.is_some() {
                    content.stream = Some(lang.clone());
                }
                content.language = Some(lang.clone());
                let prefix = format!("{lang}-");
                if !content.slug.starts_with(&prefix) {
                    content.slug = format!("{lang}-{}", content.slug);
                }
            }
            if content.language.is_none() {
                content.language = Some(default_language.to_string());
            }
        }

        let members: Vec<(usize, bool, String, String, String)> = group
            .iter()
            .map(|&(idx, is_post)| {
                let content = if is_post {
                    &site_data.posts[idx]
                } else {
                    &site_data.pages[idx]
                };
                let lang = content
                    .language
                    .clone()
                    .unwrap_or_else(|| default_language.to_string());
                (
                    idx,
                    is_post,
                    lang,
                    content.slug.clone(),
                    content.title.clone(),
                )
            })
            .collect();

        for (idx, is_post, lang, member_slug, _title) in &members {
            let mut refs: Vec<TranslationRef> = Vec::new();
            for (_, _, other_lang, other_slug, other_title) in &members {
                if other_lang == lang && other_slug == member_slug {
                    continue;
                }
                let lang_name = languages
                    .get(other_lang.as_str())
                    .map_or_else(|| other_lang.clone(), |c| c.display_name.clone());
                refs.push(TranslationRef {
                    lang: other_lang.clone(),
                    name: lang_name,
                    slug: other_slug.clone(),
                    title: other_title.clone(),
                });
            }
            if !refs.is_empty() {
                translation_links.push((*idx, *is_post, refs));
            }
        }
    }

    for (idx, is_post, refs) in translation_links {
        let content = if is_post {
            &mut site_data.posts[idx]
        } else {
            &mut site_data.pages[idx]
        };
        for tr in refs {
            if !content
                .translations
                .iter()
                .any(|existing| existing.slug == tr.slug)
            {
                content.translations.push(tr);
            }
        }
    }
}

struct TranslationContentInfo {
    title: String,
    lang: String,
    idx: usize,
    is_post: bool,
}

fn build_translation_slug_index(
    site_data: &Data,
    default_language: &str,
) -> HashMap<String, TranslationContentInfo> {
    let mut slug_index: HashMap<String, TranslationContentInfo> = HashMap::new();
    for (i, post) in site_data.posts.iter().enumerate() {
        let lang = post
            .language
            .clone()
            .or_else(|| post.stream.clone().filter(|s| is_iso_639_1_code(s)))
            .unwrap_or_else(|| default_language.to_string());
        slug_index.insert(
            post.slug.clone(),
            TranslationContentInfo {
                title: post.title.clone(),
                lang,
                idx: i,
                is_post: true,
            },
        );
    }
    for (i, page) in site_data.pages.iter().enumerate() {
        let lang = page
            .language
            .clone()
            .unwrap_or_else(|| default_language.to_string());
        slug_index.insert(
            page.slug.clone(),
            TranslationContentInfo {
                title: page.title.clone(),
                lang,
                idx: i,
                is_post: false,
            },
        );
    }
    slug_index
}

fn apply_translation_resolutions(
    site_data: &mut Data,
    updates: Vec<(usize, bool, String, String, String, String)>,
    bidirectional_adds: Vec<(usize, bool, TranslationRef)>,
) {
    for (idx, is_post, tr_slug, lang, name, title) in updates {
        let content = if is_post {
            &mut site_data.posts[idx]
        } else {
            &mut site_data.pages[idx]
        };
        if let Some(tr) = content.translations.iter_mut().find(|t| t.slug == tr_slug) {
            if tr.lang.is_empty() {
                tr.lang = lang;
            }
            if tr.name.is_empty() {
                tr.name = name;
            }
            if tr.title.is_empty() {
                tr.title = title;
            }
        }
    }

    for (idx, is_post, tr) in bidirectional_adds {
        let content = if is_post {
            &mut site_data.posts[idx]
        } else {
            &mut site_data.pages[idx]
        };
        if !content
            .translations
            .iter()
            .any(|existing| existing.slug == tr.slug)
        {
            content.translations.push(tr);
        }
    }
}

fn resolve_frontmatter_translations(
    site_data: &mut Data,
    default_language: &str,
    languages: &HashMap<String, LanguageConfig>,
) {
    let slug_index = build_translation_slug_index(site_data, default_language);

    let mut updates: Vec<(usize, bool, String, String, String, String)> = Vec::new();
    let mut bidirectional_adds: Vec<(usize, bool, TranslationRef)> = Vec::new();

    let all_contents: Vec<(usize, bool, &Content)> = site_data
        .posts
        .iter()
        .enumerate()
        .map(|(i, c)| (i, true, c))
        .chain(
            site_data
                .pages
                .iter()
                .enumerate()
                .map(|(i, c)| (i, false, c)),
        )
        .collect();

    for &(src_idx, src_is_post, content) in &all_contents {
        for tr in &content.translations {
            if tr.lang.is_empty() || tr.title.is_empty() {
                if let Some(target_info) = slug_index.get(&tr.slug) {
                    let resolved_lang = if tr.lang.is_empty() {
                        target_info.lang.clone()
                    } else {
                        tr.lang.clone()
                    };
                    let resolved_name = languages
                        .get(resolved_lang.as_str())
                        .map_or_else(|| resolved_lang.clone(), |c| c.display_name.clone());
                    let resolved_title = if tr.title.is_empty() {
                        target_info.title.clone()
                    } else {
                        tr.title.clone()
                    };

                    updates.push((
                        src_idx,
                        src_is_post,
                        tr.slug.clone(),
                        resolved_lang,
                        resolved_name,
                        resolved_title,
                    ));

                    let src_lang = content
                        .language
                        .clone()
                        .unwrap_or_else(|| default_language.to_string());
                    let src_lang_name = languages
                        .get(src_lang.as_str())
                        .map_or_else(|| src_lang.clone(), |c| c.display_name.clone());
                    bidirectional_adds.push((
                        target_info.idx,
                        target_info.is_post,
                        TranslationRef {
                            lang: src_lang,
                            name: src_lang_name,
                            slug: content.slug.clone(),
                            title: content.title.clone(),
                        },
                    ));
                } else {
                    warn!(
                        "Translation reference '{}' in '{}' not found",
                        tr.slug, content.slug
                    );
                }
            }
        }
    }

    apply_translation_resolutions(site_data, updates, bidirectional_adds);
}

fn detect_slug_collision(site_data: &Data) {
    if let Err(duplicate) = check_for_duplicate_slugs(
        &site_data
            .posts
            .iter()
            .chain(&site_data.pages)
            .collect::<Vec<_>>(),
    ) {
        error!(
            "Duplicate slug found: '{duplicate}' \
            - try setting `title` or `slug` as a unique text, \
            or leave both empty so filename will be assumed. \
            - The latest content rendered will overwrite the previous one."
        );
    }
}

#[allow(clippy::too_many_lines)]
fn initialize_tera(
    input_folder: &Path,
    site_data: &Data,
    cross_site_data: Option<&crate::workspace::CrossSiteData>,
    path_prefix: &str,
) -> (Tera, Option<ShortcodeProcessor>) {
    let mut tera = Tera::default();
    tera.autoescape_on(Vec::<&str>::new());
    let all_site_prefixes = cross_site_data
        .map(|csd| {
            csd.sites
                .values()
                .map(|sd| sd.output_path.clone())
                .collect()
        })
        .unwrap_or_default();
    tera.register_function(
        "url_for",
        UrlFor {
            base_url: site_data.site.url.clone(),
            path_prefix: path_prefix.to_string(),
            all_site_prefixes,
        },
    );
    let csd = cross_site_data.cloned();
    tera.register_function(
        "group",
        Group {
            site_data: site_data.clone(),
            cross_site_data: csd.clone(),
        },
    );
    tera.register_function(
        "source_link",
        SourceLink {
            site_data: site_data.clone(),
        },
    );
    tera.register_function(
        "stream_display_name",
        DisplayName {
            site_data: site_data.clone(),
            kind: "stream".to_string(),
        },
    );
    tera.register_function(
        "series_display_name",
        DisplayName {
            site_data: site_data.clone(),
            kind: "series".to_string(),
        },
    );
    tera.register_function(
        "language_display_name",
        DisplayName {
            site_data: site_data.clone(),
            kind: "language".to_string(),
        },
    );
    tera.register_function(
        "get_posts",
        GetPosts {
            site_data: site_data.clone(),
            cross_site_data: csd.clone(),
        },
    );
    tera.register_function(
        "get_pages",
        GetPages {
            site_data: site_data.clone(),
            cross_site_data: csd.clone(),
        },
    );
    tera.register_function(
        "get_data_by_slug",
        GetDataBySlug {
            site_data: site_data.clone(),
            cross_site_data: csd,
        },
    );
    tera.register_function(
        "get_gallery",
        GetGallery {
            site_data: site_data.clone(),
        },
    );
    tera.register_filter(
        "default_date_format",
        tera_filter::DefaultDateFormat {
            date_format: site_data.site.default_date_format.clone(),
        },
    );
    tera.register_filter("remove_draft", tera_filter::RemoveDraft);
    tera.register_filter("slugify", tera_filter::Slugify);
    tera.register_filter("striptags", tera_filter::striptags);
    tera.register_filter("trim_start_matches", tera_filter::trim_start_matches);
    tera.register_filter("slice", tera_filter::slice);
    tera.register_filter("date", tera_filter::date);

    let templates_path = site_data.site.get_templates_path(input_folder);
    let mandatory_templates = ["base.html", "list.html", "group.html", "content.html"];

    // Phase 1: Collect all template content (user templates override embedded defaults)
    let mut all_templates: Vec<(String, String)> = Vec::new();

    // Load user templates from the templates directory
    for entry in WalkDir::new(&templates_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        let template_path = entry.path();
        let template_name = template_path
            .strip_prefix(&templates_path)
            .unwrap_or(template_path)
            .to_str()
            .unwrap_or("")
            .to_string();
        let template_name = template_name.replace('\\', "/");
        let template_name = template_name.trim_start_matches('/').to_string();
        let template_content = fs::read_to_string(template_path).unwrap_or_else(|e| {
            error!("Failed to read template {template_name}: {e}");
            String::new()
        });
        all_templates.push((template_name, template_content));
    }

    // Add embedded templates as defaults for any not provided by the user
    let user_names: Vec<String> = all_templates.iter().map(|(n, _)| n.clone()).collect();
    for name in Templates::iter() {
        if user_names.iter().any(|n| n == name.as_ref()) {
            continue;
        }
        if let Some(template) = Templates::get(name.as_ref()) {
            if let Ok(template_str) = std::str::from_utf8(template.data.as_ref()) {
                all_templates.push((name.to_string(), template_str.to_string()));
            }
        }
    }

    // Verify mandatory templates exist
    for tpl_name in &mandatory_templates {
        if !all_templates.iter().any(|(n, _)| n == *tpl_name) {
            error!("Failed to load template: {tpl_name}");
            process::exit(1);
        }
    }

    // Phase 2: Collect all optional includes and register empty templates for missing ones
    let mut optional_includes: Vec<String> = Vec::new();
    let all_names: Vec<&str> = all_templates.iter().map(|(n, _)| n.as_str()).collect();
    for (_, content) in &all_templates {
        optional_includes.extend(collect_ignore_missing_includes(content));
    }
    for name in &optional_includes {
        if !all_names.contains(&name.as_str()) {
            debug!("Registering empty template for optional include: {name}");
            if let Err(e) = tera.add_raw_template(name, "") {
                error!("Failed to register empty template '{name}': {e}");
            }
        }
    }

    // Phase 3: Load all templates in one batch with preprocessing applied
    let processed_templates: Vec<(String, String)> = all_templates
        .iter()
        .map(|(name, content)| (name.clone(), preprocess_template(content)))
        .collect();
    let template_refs: Vec<(&str, &str)> = processed_templates
        .iter()
        .map(|(n, c)| (n.as_str(), c.as_str()))
        .collect();
    if let Err(e) = tera.add_raw_templates(template_refs) {
        error!("Failed to load templates: {e}");
    }

    // Initialize shortcode processor if enabled
    let shortcode_processor = if site_data.site.enable_shortcodes {
        let mut processor = ShortcodeProcessor::new(site_data.site.shortcode_pattern.as_deref());
        if let Err(e) = processor.collect_shortcodes(input_folder) {
            error!("Failed to collect shortcodes: {e}");
        }
        Some(processor)
    } else {
        None
    };

    debug!("{:#?}", &tera);
    (tera, shortcode_processor)
}

#[allow(clippy::too_many_arguments)]
fn render_templates(
    content_dir: &Path,
    site_data: &Data,
    tera: &Tera,
    output_dir: &Path,
    input_folder: &Path,
    fragments: &HashMap<String, String>,
    latest_build_info: Option<&BuildInfo>,
    shortcode_processor: Option<&ShortcodeProcessor>,
    highlighter: Option<&MarmiteHighlighter>,
    cross_site_data: Option<&crate::workspace::CrossSiteData>,
    generate_metadata: bool,
) -> Result<(), String> {
    // Build the context of variables that are global on every template
    let mut global_context = Context::new();
    global_context.insert("markdown_fragments", &fragments);
    let site_data = site_data.clone();

    global_context.insert("site_data", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);
    global_context.insert("language", &site_data.site.language);
    global_context.insert("languages", &site_data.site.languages);
    debug!("Global Context site: {:?}", &site_data.site);
    debug!("Site data galleries count: {}", site_data.galleries.len());
    collect_global_fragments(
        content_dir,
        &mut global_context,
        tera,
        &site_data.site,
        highlighter,
    );

    handle_stream_pages(&site_data, &global_context, tera, output_dir)?;
    handle_series_pages(&site_data, &global_context, tera, output_dir)?;
    // If site_data.stream.map does not contain the index stream
    // we will render empty index.html from list.html template
    if !site_data.stream.map.contains_key("index") {
        handle_default_empty_site(&global_context, tera, output_dir)?;
    }

    handle_group_pages(&global_context, &site_data, tera, output_dir)?;

    // Pages are treated as a list of content, no stream separation is needed
    // pages are usually just static pages that user will link in the menu.
    handle_list_page(
        &global_context,
        &site_data.site.pages_title,
        &site_data.pages,
        &site_data,
        tera,
        output_dir,
        "pages",
    )?;

    // Check and guarantees that page 404 was generated even if _404.md is removed
    handle_404(content_dir, &global_context, tera, output_dir, highlighter)?;

    // Render individual content-slug.html from content.html template
    // content is rendered as last step so it gives the user the ability to
    // override some prebuilt pages like tags.html, authors.html, etc.
    handle_content_pages(
        &site_data,
        &global_context,
        tera,
        output_dir,
        content_dir,
        input_folder,
        latest_build_info,
        shortcode_processor,
        highlighter,
        cross_site_data,
        generate_metadata,
    )?;

    handle_redirect_aliases(&site_data, output_dir)?;

    if generate_metadata {
        write_template_context_files(output_dir);
    }

    Ok(())
}

fn handle_group_pages(
    global_context: &Context,
    site_data: &Data,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    [
        "tags",
        "archives",
        "authors",
        "streams",
        "series",
        "languages",
    ]
    .par_iter()
    .map(|step| -> Result<(), String> {
        match *step {
            "tags" => {
                handle_tag_pages(output_dir, site_data, global_context, tera)?;
            }
            "archives" => {
                handle_archive_pages(output_dir, site_data, global_context, tera)?;
            }
            "authors" => {
                handle_author_pages(output_dir, site_data, global_context, tera)?;
            }
            "streams" => {
                handle_stream_list_page(output_dir, site_data, global_context, tera)?;
            }
            "series" => {
                handle_series_list_page(output_dir, site_data, global_context, tera)?;
            }
            "languages" => {
                handle_language_list_page(output_dir, site_data, global_context, tera)?;
            }
            _ => {}
        }
        Ok(())
    })
    .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
    .unwrap_or(Ok(()))
}

/// Assuming every item on `site_data.posts` is a Content and has a stream field
/// we can use this to render a {stream}.html page from list.html template
/// by default posts will have a `index` stream.
/// for (stream, `stream_contents`) in
fn handle_stream_pages(
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    site_data
        .stream
        .iter()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(stream, stream_contents)| -> Result<(), String> {
            let stream_slug = crate::slugify::slugify(stream);
            let title = if *stream == "index" {
                String::new()
            } else {
                site_data
                    .site
                    .streams_content_title
                    .replace("$stream", stream)
            };

            // if there is any content on the stream with pinned set to true
            // sort the content by pinned first and then by date
            let mut sorted_stream_contents = stream_contents.clone();
            sorted_stream_contents.sort_by(|a, b| {
                if a.pinned && !b.pinned {
                    std::cmp::Ordering::Less
                } else if !a.pinned && b.pinned {
                    std::cmp::Ordering::Greater
                } else {
                    b.date.cmp(&a.date)
                }
            });

            handle_list_page(
                global_context,
                &title,
                &sorted_stream_contents,
                site_data,
                tera,
                output_dir,
                &stream_slug,
            )?;

            // Skip generating feeds for draft stream
            if *stream != "draft" {
                // Render {stream}.rss for each stream
                crate::feed::generate_rss(
                    stream_contents,
                    output_dir,
                    &stream_slug,
                    &site_data.site,
                )?;

                if site_data.site.json_feed {
                    crate::feed::generate_json(
                        stream_contents,
                        output_dir,
                        &stream_slug,
                        &site_data.site,
                    )?;
                }
            }
            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))
}

/// Handle individual series pages
/// Generate series-{series}.html pages from list.html template
/// Series content is sorted chronologically (oldest to newest)
fn handle_series_pages(
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    site_data
        .series
        .iter()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(series, series_contents)| -> Result<(), String> {
            let series_slug = format!("series-{}", crate::slugify::slugify(series));
            let title = site_data
                .site
                .series_content_title
                .replace("$series", series);

            // Series content is already sorted chronologically (oldest to newest) by GroupedContent::sort_all
            let sorted_series_contents = series_contents.clone();

            handle_list_page(
                global_context,
                &title,
                &sorted_series_contents,
                site_data,
                tera,
                output_dir,
                &series_slug,
            )?;

            // Generate RSS feed for series
            crate::feed::generate_rss(series_contents, output_dir, &series_slug, &site_data.site)?;

            if site_data.site.json_feed {
                crate::feed::generate_json(
                    series_contents,
                    output_dir,
                    &series_slug,
                    &site_data.site,
                )?;
            }

            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))
}

fn handle_default_empty_site(
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    let mut index_context = global_context.clone();
    index_context.insert("title", &"Welcome to Marmite");
    let empty_content_list: Vec<Content> = Vec::new();
    index_context.insert("content_list", &empty_content_list);
    index_context.insert("total_pages", &1);
    index_context.insert("per_page", &1);
    index_context.insert("total_content", &1);
    index_context.insert("current_page", "index.html");
    index_context.insert("current_page_number", &1);
    render_html(
        "custom_index.html,list.html",
        "index.html",
        tera,
        &index_context,
        output_dir,
    )?;
    Ok(())
}

fn handle_stream_list_page(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    let mut stream_list_context = global_context.clone();
    stream_list_context.insert("title", &site_data.site.streams_title);
    stream_list_context.insert("current_page", "streams.html");
    stream_list_context.insert("kind", "stream");
    render_html(
        "group.html",
        "streams.html",
        tera,
        &stream_list_context,
        output_dir,
    )?;
    Ok(())
}

fn handle_series_list_page(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    let mut series_list_context = global_context.clone();
    series_list_context.insert("title", &site_data.site.series_title);
    series_list_context.insert("current_page", "series.html");
    series_list_context.insert("kind", "series");
    render_html(
        "group.html",
        "series.html",
        tera,
        &series_list_context,
        output_dir,
    )?;
    Ok(())
}

fn handle_language_list_page(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    let mut lang_list_context = global_context.clone();
    lang_list_context.insert("title", &site_data.site.languages_title);
    lang_list_context.insert("current_page", "languages.html");
    lang_list_context.insert("kind", "language");
    render_html(
        "group.html",
        "languages.html",
        tera,
        &lang_list_context,
        output_dir,
    )?;
    Ok(())
}

fn handle_author_pages(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    site_data
        .author
        .iter()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(username, _)| -> Result<(), String> {
            let default_author = Author {
                name: (*username).clone(),
                bio: None,
                avatar: Some("static/avatar-placeholder.png".to_string()),
                links: None,
            };
            let mut author_context = global_context.clone();
            let author = if let Some(author) = site_data.site.authors.get(*username) {
                author
            } else {
                &default_author
            };
            author_context.insert("author", &author);

            let author_slug = crate::slugify::slugify(username);
            let mut author_posts = site_data
                .posts
                .iter()
                .filter(|post| {
                    post.authors.contains(username) && post.stream.as_deref() != Some("draft")
                })
                .cloned()
                .collect::<Vec<Content>>();

            author_posts.sort_by(|a, b| {
                if a.pinned && !b.pinned {
                    std::cmp::Ordering::Less
                } else if !a.pinned && b.pinned {
                    std::cmp::Ordering::Greater
                } else {
                    b.date.cmp(&a.date)
                }
            });

            let filename = format!("author-{}", &author_slug);
            handle_list_page(
                &author_context,
                &author.name,
                &author_posts,
                site_data,
                tera,
                output_dir,
                &filename,
            )?;

            // Render author-{name}.rss for each stream
            crate::feed::generate_rss(
                &author_posts,
                output_dir,
                &filename.clone(),
                &site_data.site,
            )?;

            if site_data.site.json_feed {
                crate::feed::generate_json(&author_posts, output_dir, &filename, &site_data.site)?;
            }

            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))?;

    // Render authors.html group page
    let mut authors_list_context = global_context.clone();
    authors_list_context.insert("title", &site_data.site.authors_title);
    authors_list_context.insert("current_page", "authors.html");
    authors_list_context.insert("kind", "author");
    render_html(
        "group.html",
        "authors.html",
        tera,
        &authors_list_context,
        output_dir,
    )?;

    Ok(())
}

fn handle_file_mappings(
    input_folder: &Path,
    output_folder: &Path,
    file_mappings: &[crate::config::FileMapping],
) {
    for mapping in file_mappings {
        let source_path = if Path::new(&mapping.source).is_absolute() {
            std::path::PathBuf::from(&mapping.source)
        } else {
            input_folder.join(&mapping.source)
        };

        let dest_path = output_folder.join(&mapping.dest);

        // Ensure destination directory exists
        if let Some(parent) = dest_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!(
                    "Failed to create destination directory {}: {e:?}",
                    parent.display()
                );
                continue;
            }
        }

        // Check if source is a single file
        if source_path.is_file() {
            match fs::copy(&source_path, &dest_path) {
                Ok(_) => info!(
                    "Copied file mapping: {} -> {}",
                    source_path.display(),
                    dest_path.display()
                ),
                Err(e) => error!("Failed to copy file {}: {e:?}", source_path.display()),
            }
        }
        // Check if source is a directory
        else if source_path.is_dir() {
            // Create the destination directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&dest_path) {
                error!(
                    "Failed to create destination directory {}: {e:?}",
                    dest_path.display()
                );
                continue;
            }

            let mut options = CopyOptions::new();
            options.overwrite = true;
            options.content_only = false;

            match dircopy(&source_path, &dest_path, &options) {
                Ok(size) => {
                    info!(
                        "Copied directory mapping: {} -> {} ({} bytes)",
                        source_path.display(),
                        dest_path.display(),
                        size
                    );
                }
                Err(e) => {
                    error!("Failed to copy directory {}: {e:?}", source_path.display());
                }
            }
        }
        // Otherwise treat as glob pattern
        else {
            let pattern = source_path.to_str().unwrap_or("");
            match glob(pattern) {
                Ok(paths) => {
                    for path_result in paths {
                        match path_result {
                            Ok(path) => {
                                if path.is_file() {
                                    let file_name = path.file_name().unwrap_or_default();
                                    let final_dest = if dest_path.extension().is_some() {
                                        // If dest has extension, it's a file
                                        dest_path.clone()
                                    } else {
                                        // Otherwise it's a directory
                                        dest_path.join(file_name)
                                    };

                                    // Ensure parent directory exists
                                    if let Some(parent) = final_dest.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }

                                    match fs::copy(&path, &final_dest) {
                                        Ok(_) => debug!(
                                            "Copied glob match: {} -> {}",
                                            path.display(),
                                            final_dest.display()
                                        ),
                                        Err(e) => error!(
                                            "Failed to copy glob file {}: {e:?}",
                                            path.display()
                                        ),
                                    }
                                }
                            }
                            Err(e) => error!("Glob error: {e:?}"),
                        }
                    }
                    info!(
                        "Processed glob mapping: {} -> {}",
                        pattern,
                        dest_path.display()
                    );
                }
                Err(e) => error!("Invalid glob pattern {pattern}: {e:?}"),
            }
        }
    }
}

fn build_code_highlighter(site: &Marmite) -> Option<Arc<MarmiteHighlighter>> {
    let cfg = site.code_highlight.clone().unwrap_or_default();
    if !cfg.enabled {
        return None;
    }
    match highlight::build(&cfg) {
        Ok(hl) => Some(hl),
        Err(e) => {
            error!("code_highlight misconfigured: {e}");
            None
        }
    }
}

fn write_code_highlight_css(site: &Marmite, output_folder: &Arc<std::path::PathBuf>) {
    let cfg = site.code_highlight.clone().unwrap_or_default();
    if !cfg.enabled {
        // NB: No CSS is written if syntax highlighting is disabled!
        return;
    }
    match highlight::generate_css(&cfg) {
        Ok(css) => {
            let static_dir = output_folder.join(site.static_path.clone());
            if let Err(e) = fs::create_dir_all(&static_dir) {
                error!("Failed to create static dir for arborium.css: {e:?}");
                return;
            }
            let path = static_dir.join("arborium.css");
            if let Err(e) = fs::write(&path, css) {
                error!("Failed to write {}: {e:?}", path.display());
            } else {
                info!("Generated {}", path.display());
            }
        }
        Err(e) => error!("Failed to generate code-highlight CSS: {e}"),
    }
}

const CORE_STATIC_FILES: &[&str] = &[
    "marmite.css",
    "marmite.js",
    "search.js",
    "pico.min.css",
    "AtkinsonHyperlegibleNext-Regular.woff2",
];

fn is_core_static_file(name: &str) -> bool {
    CORE_STATIC_FILES.contains(&name) || name.starts_with("colorschemes/")
}

fn check_static_drift(user_static_dir: &Path) {
    for (name, embedded_data) in EMBEDDED_STATIC.iter() {
        if !is_core_static_file(name) {
            continue;
        }
        let user_file = user_static_dir.join(name);
        if user_file.exists() {
            if let Ok(user_data) = fs::read(&user_file) {
                if user_data != *embedded_data {
                    warn!(
                        "Static file '{}' differs from the embedded version. \
                         The embedded version may contain updates or fixes. \
                         To use the embedded version, remove '{}' from your static folder.",
                        name,
                        user_file.display()
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn handle_static_artifacts(
    input_folder: &Path,
    site_data: &Data,
    output_folder: &Arc<std::path::PathBuf>,
    content_dir: &std::path::Path,
) {
    let static_source = site_data.site.get_static_path(input_folder);
    let has_theme = site_data.site.theme.is_some();

    if has_theme && static_source.is_dir() {
        // Theme provides its own complete static files
        let mut options = CopyOptions::new();
        options.overwrite = true;

        if let Err(e) = dircopy(&static_source, &**output_folder, &options) {
            error!("Failed to copy static directory: {e:?}");
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &static_source.display(),
            &output_folder.display()
        );
    } else {
        // No theme (or theme without static dir) - use embedded as base
        let output_static = output_folder.join(site_data.site.static_path.clone());
        generate_static(&output_static);

        // Copy user's own static files on top if they exist
        let user_static = input_folder.join(&site_data.site.static_path);
        if user_static.is_dir() {
            check_static_drift(&user_static);

            let mut options = CopyOptions::new();
            options.overwrite = true;

            if let Err(e) = dircopy(&user_static, &**output_folder, &options) {
                error!("Failed to copy user static directory: {e:?}");
                process::exit(1);
            }

            info!(
                "Copied '{}' on top of embedded static to '{}/'",
                &user_static.display(),
                &output_folder.display()
            );
        }
    }

    // Copy extra static folders if present
    if let Some(extra_data) = site_data.site.extra.clone() {
        if let Some(static_folders_value) = extra_data.get("static_folders") {
            if let Ok(static_folders) =
                serde_yaml::from_value::<Vec<std::path::PathBuf>>(static_folders_value.clone())
            {
                for static_folder in static_folders {
                    let source_folder = input_folder.join(&static_folder);
                    if source_folder.is_dir() {
                        let mut options = CopyOptions::new();
                        options.overwrite = true;

                        if let Err(e) = dircopy(&source_folder, &**output_folder, &options) {
                            error!("Failed to copy extra static folders directory: {e:?}");
                            process::exit(1);
                        }

                        info!(
                            "Copied extra static folders '{}' to '{}/'",
                            source_folder.display(),
                            output_folder.display()
                        );
                    }
                }
            }
        }
    }

    // Copy content/media folder if present
    let media_source = content_dir.join(site_data.site.media_path.clone());
    if media_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = dircopy(&media_source, &**output_folder, &options) {
            error!("Failed to copy media directory: {e:?}");
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &media_source.display(),
            &output_folder.display()
        );
    }

    // Copy media from content subfolders at any depth.
    // content/{any/path}/media/ -> output/media/{any/path}/
    // These take precedence over global media since they're copied after.
    let media_folder_name = &site_data.site.media_path;
    for entry in WalkDir::new(content_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if dir_name != media_folder_name.as_str() {
            continue;
        }
        if path.parent() == Some(content_dir) {
            continue;
        }
        // Map content/{any/path}/media/ -> output/media/{parent-name}/
        // Uses the immediate parent name (matches the slug) to stay
        // consistent with @/ references and banner discovery.
        let Some(parent) = path.parent() else {
            continue;
        };
        let Some(subfolder_name) = parent.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let dest = output_folder.join(media_folder_name).join(subfolder_name);
        if let Err(e) = fs::create_dir_all(&dest) {
            error!("Failed to create media subfolder directory: {e:?}");
            continue;
        }
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.content_only = true;
        if let Err(e) = dircopy(path, &dest, &options) {
            error!("Failed to copy subfolder media: {e:?}");
            continue;
        }
        debug!(
            "Copied content subfolder media '{}' to '{}'",
            path.display(),
            dest.display()
        );
    }

    // Process image resizing if configured in extra (and not skipped)
    if media_source.is_dir() {
        if site_data.site.skip_image_resize {
            debug!("Image resizing skipped (--skip-image-resize flag)");
        } else if let Some(extra) = &site_data.site.extra {
            if extra.get("banner_image_width").is_some() || extra.get("max_image_width").is_some() {
                let output_media_path = output_folder.join(&site_data.site.media_path);
                let banner_paths = image_resize::collect_banner_paths_from_content(
                    &site_data.posts,
                    &site_data.pages,
                );
                image_resize::process_media_images(
                    &output_media_path,
                    &site_data.site,
                    &banner_paths,
                );
            }
        }
    }

    // Handle file mappings
    if !site_data.site.file_mapping.is_empty() {
        handle_file_mappings(input_folder, output_folder, &site_data.site.file_mapping);
    }

    // we want to check if the file exists in `input_folder` and `content_dir`
    // if not then we want to check if exists in `output_folder/static` (came from embedded)
    // the first we find we want to copy to the `output_folder/{destiny_path}`
    let custom_files = [
        // name, destination
        ("custom.css", site_data.site.static_path.clone()),
        ("custom.js", site_data.site.static_path.clone()),
        ("favicon.ico", site_data.site.static_path.clone()),
        ("robots.txt", String::new()),
    ];
    let output_static_destiny = output_folder.join(site_data.site.static_path.clone());
    let possible_sources = [input_folder, content_dir, output_static_destiny.as_path()];
    let mut copied_custom_files = Vec::new();
    for possible_source in &possible_sources {
        for custom_file in &custom_files {
            let source_file = possible_source.join(custom_file.0);
            if copied_custom_files.contains(&custom_file.0.to_string()) {
                continue;
            }
            if source_file.exists() {
                let destiny_path = output_folder
                    .join(custom_file.1.clone())
                    .join(custom_file.0);
                match fs::copy(&source_file, &destiny_path) {
                    Ok(_) => {
                        copied_custom_files.push(custom_file.0.to_string());
                        info!(
                            "Copied {} to {}",
                            source_file.display(),
                            &destiny_path.display()
                        );
                    }
                    Err(e) => error!("Failed to copy {}: {e:?}", source_file.display()),
                }
            }
        }
    }

    // Copy favicon.ico to the output root so browsers can find it at /favicon.ico
    // (browsers request this path automatically in addition to the <link> tag in <head>)
    let favicon_at_static = output_folder
        .join(site_data.site.static_path.clone())
        .join("favicon.ico");
    let favicon_at_root = output_folder.join("favicon.ico");
    if favicon_at_static.exists() && !favicon_at_root.exists() {
        match fs::copy(&favicon_at_static, &favicon_at_root) {
            Ok(_) => info!("Copied favicon.ico to site root"),
            Err(e) => error!("Failed to copy favicon.ico to site root: {e:?}"),
        }
    }

    // Generate code highlighting CSS based on the site settings
    write_code_highlight_css(&site_data.site, output_folder);
}

fn generate_search_index(site_data: &Data, output_folder: &Arc<std::path::PathBuf>) {
    let remove_html_tags = |html: &str| -> String {
        // Remove HTML tags, Liquid tags, and Jinja tags
        let re = Regex::new(re::MATCH_HTML_OR_TEMPLATE_TAGS)
            .map_err(|e| format!("Failed to create regex: {e}"))
            .unwrap_or_else(|e| {
                error!("Regex compilation failed: {e}");
                // Return a simple regex that won't crash but may not work perfectly
                Regex::new(re::MATCH_HTML_TAGS).expect("Basic regex should compile")
            });
        re.replace_all(html, "")
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };

    let convert_items_to_json = |item: &Content| {
        serde_json::json!({
            "title": item.title,
            "description": item.description,
            "tags": item.tags,
            "slug": item.slug,
            "html": remove_html_tags(&item.html),
        })
    };

    // Merge posts and pages into a single list, filtering out draft content
    let all_content_json = site_data
        .posts
        .iter()
        .filter(|content| {
            content
                .stream
                .as_ref()
                .is_none_or(|stream| stream != "draft")
        })
        .map(convert_items_to_json)
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            site_data
                .pages
                .iter()
                .filter(|content| {
                    content
                        .stream
                        .as_ref()
                        .is_none_or(|stream| stream != "draft")
                })
                .map(convert_items_to_json)
                .collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();

    let search_json_path = output_folder
        .join(site_data.site.static_path.clone())
        .join("search_index.json");
    if let Err(e) = fs::write(
        search_json_path,
        serde_json::to_string(&all_content_json).unwrap_or_else(|e| {
            error!("Failed to serialize search index: {e}");
            "[]".to_string()
        }),
    ) {
        error!("Failed to write search_index.json: {e:?}");
    } else {
        info!("Generated search_index.json");
    }
}

fn copy_markdown_sources(site_data: &Data, content_folder: &Path, output_path: &Path) {
    site_data
        .posts
        .iter()
        .chain(&site_data.pages)
        .for_each(|content| {
            if let Some(source_path) = &content.source_path {
                let relative_path = source_path
                    .strip_prefix(content_folder)
                    .unwrap_or(source_path);
                let dest_path = output_path.join(relative_path);

                if let Some(parent) = dest_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        error!("Failed to create directory for markdown source: {e:?}");
                        return;
                    }
                }

                if let Err(e) = fs::copy(source_path, &dest_path) {
                    error!(
                        "Failed to copy markdown source {}: {e:?}",
                        source_path.display()
                    );
                } else {
                    info!("Copied markdown source to {}", dest_path.display());
                }
            }
        });
}

#[allow(clippy::too_many_lines)]
fn write_template_context_files(output_dir: &Path) {
    let global_vars: Vec<&str> = vec![
        "site",
        "site.name",
        "site.tagline",
        "site.url",
        "site.footer",
        "site.language",
        "site.pagination",
        "site.extra",
        "menu",
        "language",
        "languages",
        "hero",
        "sidebar",
        "announce",
        "header",
        "footer",
        "comments",
        "htmlhead",
        "htmltail",
        "markdown_fragments",
        "site_data",
    ];
    let functions: Vec<&str> = vec![
        "url_for",
        "group",
        "get_posts",
        "get_pages",
        "get_data_by_slug",
        "source_link",
        "stream_display_name",
        "series_display_name",
        "language_display_name",
        "get_gallery",
    ];
    let filters: Vec<&str> = vec![
        "default_date_format",
        "remove_draft",
        "slugify",
        "striptags",
        "trim_start_matches",
        "slice",
        "date",
    ];

    let templates = vec![
        (
            "content",
            vec![
                "title",
                "content",
                "content.title",
                "content.slug",
                "content.html",
                "content.date",
                "content.tags",
                "content.authors",
                "content.description",
                "content.stream",
                "content.series",
                "content.toc",
                "content.banner_image",
                "content.card_image",
                "content.next",
                "content.previous",
                "content.back_links",
                "content.extra",
                "content.comments",
                "content.language",
                "content.translations",
                "content.pinned",
                "current_page",
            ],
        ),
        (
            "list",
            vec![
                "title",
                "content_list",
                "per_page",
                "total_pages",
                "total_content",
                "current_page",
                "current_page_number",
                "previous_page",
                "next_page",
                "author",
            ],
        ),
        ("group", vec!["title", "current_page", "kind"]),
        ("base", vec![]),
        (
            "pagination",
            vec![
                "current_page_number",
                "total_pages",
                "previous_page",
                "next_page",
                "current_page",
            ],
        ),
        ("sitemap", vec!["sitemap_urls"]),
    ];

    for (name, specific_vars) in &templates {
        let mut all_vars: Vec<&str> = global_vars.clone();
        all_vars.extend(specific_vars);
        let context = serde_json::json!({
            "template": format!("{name}.html"),
            "variables": all_vars,
            "functions": functions,
            "filters": filters,
        });
        let path = output_dir.join(format!("template.{name}.context.json"));
        if let Err(e) = fs::write(
            &path,
            serde_json::to_string_pretty(&context).unwrap_or_default(),
        ) {
            error!("Failed to write template context for {name}: {e}");
        }
    }
}

fn write_build_info(output_path: &Path, site_data: &Data, input_folder: &Path, end_time: f64) {
    let shortcodes = if site_data.site.enable_shortcodes {
        let mut processor = ShortcodeProcessor::new(site_data.site.shortcode_pattern.as_deref());
        let _ = processor.collect_shortcodes(input_folder);
        processor
            .list_shortcodes_with_descriptions()
            .into_iter()
            .map(|(name, _)| name.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let build_info = BuildInfo {
        marmite_version: env!("CARGO_PKG_VERSION").to_string(),
        posts: site_data
            .posts
            .iter()
            .map(ContentInfo::from_content)
            .collect(),
        pages: site_data
            .pages
            .iter()
            .map(ContentInfo::from_content)
            .collect(),
        shortcodes,
        generated_at: chrono::Local::now().to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        elapsed_time: end_time,
        config: site_data.site.clone(),
    };

    let build_info_path = output_path.join("marmite.json");
    if let Err(e) = fs::write(
        &build_info_path,
        serde_json::to_string_pretty(&build_info).unwrap_or_else(|e| {
            error!("Failed to serialize build info: {e}");
            "{}".to_string()
        }),
    ) {
        error!("Failed to write marmite.json: {e:?}");
    } else {
        info!("Generated build info at marmite.json");
    }
}

fn generate_sitemap(site_data: &Data, tera: &Tera, output_path: &Path) {
    if !site_data.site.build_sitemap {
        return;
    }

    // Create UrlFor function instance
    let url_for = UrlFor {
        base_url: site_data.site.url.clone(),
        ..Default::default()
    };

    // Helper to generate URL using url_for
    let generate_url = |path: &str| -> String {
        let abs = !site_data.site.url.is_empty();
        url_for.resolve(path, abs)
    };

    // Get all URLs from the shared collection and apply URL generation
    let all_raw_urls = site_data.generated_urls.get_all_urls();
    let sitemap_urls: Vec<String> = all_raw_urls
        .iter()
        .map(|url| {
            // Remove leading slash if present for consistent path handling
            let path = url.strip_prefix('/').unwrap_or(url);
            generate_url(path)
        })
        .collect();

    // Render sitemap
    let mut context = Context::new();
    context.insert("sitemap_urls", &sitemap_urls);

    match tera.render("sitemap.xml", &context) {
        Ok(rendered) => {
            let sitemap_path = output_path.join("sitemap.xml");
            if let Err(e) = fs::write(&sitemap_path, rendered) {
                error!("Failed to write sitemap.xml: {e:?}");
            } else {
                info!("Generated sitemap.xml with {} URLs", sitemap_urls.len());
            }
        }
        Err(e) => {
            error!("Failed to render sitemap.xml: {e:?}");
        }
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn create_urls_json(site_data: &Data, path_prefix: &str) -> serde_json::Value {
    let url_for = UrlFor {
        base_url: site_data.site.url.clone(),
        path_prefix: path_prefix.to_string(),
        ..Default::default()
    };

    // Determine if we should use absolute URLs
    let use_abs = !site_data.site.url.is_empty();

    // Helper to generate URL using url_for
    let generate_url = |path: &str| -> String { url_for.resolve(path, use_abs) };

    // Convert URL collection to full URLs with proper formatting
    let mut output = serde_json::Map::new();

    // Add posts
    let posts: Vec<String> = site_data
        .generated_urls
        .posts
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "posts".to_string(),
        serde_json::Value::Array(
            posts
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add pages
    let pages: Vec<String> = site_data
        .generated_urls
        .pages
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "pages".to_string(),
        serde_json::Value::Array(
            pages
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add tags
    let tags: Vec<String> = site_data
        .generated_urls
        .tags
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "tags".to_string(),
        serde_json::Value::Array(
            tags.iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add authors
    let authors: Vec<String> = site_data
        .generated_urls
        .authors
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "authors".to_string(),
        serde_json::Value::Array(
            authors
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add series
    let series: Vec<String> = site_data
        .generated_urls
        .series
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "series".to_string(),
        serde_json::Value::Array(
            series
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add streams
    let streams: Vec<String> = site_data
        .generated_urls
        .streams
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "streams".to_string(),
        serde_json::Value::Array(
            streams
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add archives
    let archives: Vec<String> = site_data
        .generated_urls
        .archives
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "archives".to_string(),
        serde_json::Value::Array(
            archives
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add languages
    let languages: Vec<String> = site_data
        .generated_urls
        .languages
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "languages".to_string(),
        serde_json::Value::Array(
            languages
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add feeds
    let feeds: Vec<String> = site_data
        .generated_urls
        .feeds
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "feeds".to_string(),
        serde_json::Value::Array(
            feeds
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add pagination
    let pagination: Vec<String> = site_data
        .generated_urls
        .pagination
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "pagination".to_string(),
        serde_json::Value::Array(
            pagination
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add file_mappings
    let file_mappings: Vec<String> = site_data
        .generated_urls
        .file_mappings
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "file_mappings".to_string(),
        serde_json::Value::Array(
            file_mappings
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add redirects
    let redirects: Vec<String> = site_data
        .generated_urls
        .redirects
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "redirects".to_string(),
        serde_json::Value::Array(
            redirects
                .iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add misc
    let misc: Vec<String> = site_data
        .generated_urls
        .misc
        .iter()
        .map(|url| generate_url(url.trim_start_matches('/')))
        .collect();
    output.insert(
        "misc".to_string(),
        serde_json::Value::Array(
            misc.iter()
                .map(|url| serde_json::Value::String(url.clone()))
                .collect(),
        ),
    );

    // Add summary
    let mut summary = serde_json::Map::new();
    summary.insert(
        "posts".to_string(),
        serde_json::Value::Number(serde_json::Number::from(posts.len())),
    );
    summary.insert(
        "pages".to_string(),
        serde_json::Value::Number(serde_json::Number::from(pages.len())),
    );
    summary.insert(
        "tags".to_string(),
        serde_json::Value::Number(serde_json::Number::from(tags.len())),
    );
    summary.insert(
        "authors".to_string(),
        serde_json::Value::Number(serde_json::Number::from(authors.len())),
    );
    summary.insert(
        "series".to_string(),
        serde_json::Value::Number(serde_json::Number::from(series.len())),
    );
    summary.insert(
        "streams".to_string(),
        serde_json::Value::Number(serde_json::Number::from(streams.len())),
    );
    summary.insert(
        "archives".to_string(),
        serde_json::Value::Number(serde_json::Number::from(archives.len())),
    );
    summary.insert(
        "languages".to_string(),
        serde_json::Value::Number(serde_json::Number::from(languages.len())),
    );
    summary.insert(
        "feeds".to_string(),
        serde_json::Value::Number(serde_json::Number::from(feeds.len())),
    );
    summary.insert(
        "pagination".to_string(),
        serde_json::Value::Number(serde_json::Number::from(pagination.len())),
    );
    summary.insert(
        "file_mappings".to_string(),
        serde_json::Value::Number(serde_json::Number::from(file_mappings.len())),
    );
    summary.insert(
        "redirects".to_string(),
        serde_json::Value::Number(serde_json::Number::from(redirects.len())),
    );
    summary.insert(
        "misc".to_string(),
        serde_json::Value::Number(serde_json::Number::from(misc.len())),
    );
    summary.insert(
        "total".to_string(),
        serde_json::Value::Number(serde_json::Number::from(
            site_data.generated_urls.total_count(),
        )),
    );

    // Add meta information
    let mut meta = serde_json::Map::new();
    meta.insert(
        "url".to_string(),
        serde_json::Value::String(site_data.site.url.clone()),
    );
    meta.insert(
        "absolute_urls".to_string(),
        serde_json::Value::Bool(use_abs),
    );

    summary.insert("meta".to_string(), serde_json::Value::Object(meta));
    output.insert("summary".to_string(), serde_json::Value::Object(summary));

    serde_json::Value::Object(output)
}

fn generate_urls_json(site_data: &Data, output_path: &Path, path_prefix: &str) {
    if !site_data.site.publish_urls_json {
        return;
    }

    let json = create_urls_json(site_data, path_prefix);

    // Write JSON to file
    let urls_file = output_path.join("urls.json");
    match serde_json::to_string_pretty(&json) {
        Ok(json_string) => {
            if let Err(e) = fs::write(&urls_file, json_string) {
                error!("Failed to write urls.json: {e:?}");
            } else {
                info!("Generated urls.json");
            }
        }
        Err(e) => {
            error!("Failed to serialize URLs to JSON: {e}");
        }
    }
}

fn handle_list_page(
    global_context: &Context,
    title: &str,
    all_content: &[Content],
    site_data: &Data,
    tera: &Tera,
    output_dir: &Path,
    output_filename: &str,
) -> Result<(), String> {
    let per_page = &site_data.site.pagination;
    let total_content = all_content.len();
    let mut context = global_context.clone();
    context.insert("title", title);
    context.insert("per_page", &per_page);
    context.insert("current_page", &format!("{output_filename}.html"));

    // If all_content is empty, ensure we still generate an empty page
    if total_content == 0 {
        let empty_content_list: Vec<Content> = Vec::new();
        context.insert("content_list", &empty_content_list);
        context.insert("total_pages", &1);
        context.insert("total_content", &1);
        context.insert("current_page_number", &1);
        render_html(
            &format!("custom_{output_filename},list.html"),
            &format!("{output_filename}.html"),
            tera,
            &context,
            output_dir,
        )?;
        return Ok(());
    }

    let total_pages = total_content.div_ceil(*per_page);
    context.insert("total_content", &total_content);
    context.insert("total_pages", &total_pages);
    (0..total_pages)
        .into_par_iter()
        .map(|page_num| -> Result<(), String> {
            // Slice the content list for this page
            let page_content = &all_content
                [page_num * per_page..(page_num * per_page + per_page).min(total_content)];

            let mut context = context.clone();
            // Set up context for pagination
            context.insert("content_list", page_content);

            let current_page_number = page_num + 1;
            let filename = format!("{output_filename}-{current_page_number}.html");

            if title.is_empty() {
                context.insert("title", &format!("Page - {current_page_number}"));
            } else {
                context.insert("title", &format!("{title} - {current_page_number}"));
            }

            context.insert("current_page", &filename);
            context.insert("current_page_number", &current_page_number);

            if page_num > 0 {
                let prev_page = format!("{output_filename}-{page_num}.html");
                context.insert("previous_page", &prev_page);
            }

            if page_num < total_pages - 1 {
                let next_page = format!("{output_filename}-{}.html", page_num + 2);
                context.insert("next_page", &next_page);
            }

            debug!(
                "List Context for {}: {:#?} - Pagination({:#?})",
                filename,
                page_content
                    .iter()
                    .map(|p| format!("title:{}, slug:{}", p.title, p.slug))
                    .collect::<Vec<_>>(),
                [
                    "total_pages",
                    "per_page",
                    "total_content",
                    "current_page",
                    "current_page_number",
                    "previous_page",
                    "next_page"
                ]
                .iter()
                .map(|name| format!("{}:{}", name, context.get(name).unwrap_or(&Value::none())))
                .collect::<Vec<_>>()
            );

            // Render the HTML file for this page
            let templates = format!("custom_{output_filename},list.html");
            render_html(&templates, &filename, tera, &context, output_dir)?;
            // If there isn't an item in site_data.pages with the same slug as output_filename
            // we will render a {output_filename}.html with the same content as {output_filename}-1.html
            // this will generate a duplicate page for each stream, but will allow the user to
            // have a custom first page for each stream by dropping a {stream}.md file on the content folder
            if current_page_number == 1 {
                let page_exists = site_data
                    .pages
                    .iter()
                    .any(|page| page.slug == output_filename);
                if !page_exists {
                    context.insert("current_page", &format!("{output_filename}.html"));
                    context.insert("title", title);
                    render_html(
                        &templates,
                        &format!("{output_filename}.html"),
                        tera,
                        &context,
                        output_dir,
                    )?;
                }
            }
            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_redirect_html(target_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Redirecting...</title>
<meta http-equiv="refresh" content="0; url={target_url}">
<link rel="canonical" href="{target_url}">
</head>
<body>
<p>This page has moved to <a href="{target_url}">{target_url}</a>.</p>
<script>window.location.href = "{target_url}";</script>
</body>
</html>
"#
    )
}

fn handle_redirect_aliases(site_data: &Data, output_dir: &Path) -> Result<(), String> {
    let url_for = UrlFor {
        base_url: site_data.site.url.clone(),
        ..Default::default()
    };

    let all_slugs: std::collections::HashSet<String> = site_data
        .posts
        .iter()
        .chain(&site_data.pages)
        .map(|c| c.slug.clone())
        .collect();

    let mut seen_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for content in site_data.posts.iter().chain(&site_data.pages) {
        for alias in &content.aliases {
            if all_slugs.contains(alias) {
                warn!(
                    "Redirect alias \"{}\" in \"{}\" conflicts with an existing content slug, skipping",
                    alias, content.slug
                );
                continue;
            }

            if let Some(other_slug) = seen_aliases.get(alias) {
                warn!(
                    "Duplicate redirect alias \"{}\" defined in \"{}\" and \"{}\", skipping duplicate",
                    alias, other_slug, content.slug
                );
                continue;
            }

            seen_aliases.insert(alias.clone(), content.slug.clone());

            let target_url = url_for.resolve(&format!("{}.html", content.slug), false);
            let redirect_html = generate_redirect_html(&target_url);
            let output_file = output_dir.join(format!("{alias}.html"));
            fs::write(&output_file, redirect_html)
                .map_err(|e| format!("Failed to write redirect alias {alias}.html: {e}"))?;
            info!("Generated redirect: {alias}.html -> {}.html", content.slug);
        }
    }

    Ok(())
}

fn build_content_metadata(content: &Content, content_dir: &Path) -> serde_json::Value {
    let source_path = content
        .source_path
        .as_ref()
        .and_then(|p| p.strip_prefix(content_dir).ok())
        .map(|p| p.display().to_string());

    let last_updated = content.modified_time.map(|ts| {
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default()
    });

    serde_json::json!({
        "frontmatter": {
            "title": content.title,
            "description": content.description,
            "slug": content.slug,
            "date": content.date.map(|d| d.to_string()),
            "tags": content.tags,
            "authors": content.authors,
            "stream": content.stream,
            "series": content.series,
            "pinned": content.pinned,
            "language": content.language,
            "translates": content.translates,
            "translations": content.translations,
            "card_image": content.card_image,
            "banner_image": content.banner_image,
            "aliases": content.aliases,
            "comments": content.comments,
            "extra": content.extra,
        },
        "source_path": source_path,
        "last_updated": last_updated,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_content_pages(
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
    content_dir: &Path,
    input_folder: &Path,
    latest_build_info: Option<&BuildInfo>,
    shortcode_processor: Option<&ShortcodeProcessor>,
    highlighter: Option<&MarmiteHighlighter>,
    cross_site_data: Option<&crate::workspace::CrossSiteData>,
    generate_metadata: bool,
) -> Result<(), String> {
    let last_build = site_data.latest_timestamp.unwrap_or(0);
    let force_render = should_force_render(
        input_folder,
        site_data,
        last_build,
        content_dir,
        latest_build_info,
    );

    site_data
        .posts
        .iter()
        .chain(&site_data.pages)
        .collect::<Vec<_>>()
        .par_iter()
        .filter(|content| {
            // render only if force_render or content is newer than the latest timestamp
            force_render || content.modified_time.unwrap_or(i64::MAX) > last_build
        })
        .map(|content| -> Result<(), String> {
            let mut content_context = global_context.clone();
            content_context.insert("title", &content.title);
            content_context.insert("content", &content);
            content_context.insert("current_page", &format!("{}.html", &content.slug));
            if let Some(ref lang) = content.language {
                content_context.insert("language", lang);
            }
            debug!(
                "{} context: {:?}",
                &content.slug,
                format!(
                    "title: {},date: {:?},tags: {:?}",
                    &content.title, &content.date, &content.tags
                )
            );

            if content.comments == Some(false) {
                content_context.remove("comments");
            }

            let result = render_html_with_shortcodes(
                "content.html",
                &format!("{}.html", &content.slug),
                tera,
                &content_context,
                output_dir,
                &PostProcessors {
                    shortcode_processor,
                    site_data: Some(site_data),
                    highlighter,
                    cross_site_data,
                },
            );

            if generate_metadata && result.is_ok() {
                let metadata = build_content_metadata(content, content_dir);
                let metadata_path = output_dir.join(format!("{}.metadata.json", &content.slug));
                if let Err(e) = fs::write(
                    &metadata_path,
                    serde_json::to_string_pretty(&metadata).unwrap_or_default(),
                ) {
                    error!("Failed to write metadata for {}: {e}", content.slug);
                }
            }

            result
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))
}

#[allow(clippy::cast_possible_wrap)]
fn should_force_render(
    input_folder: &Path,
    site_data: &Data,
    last_build: i64,
    content_dir: &Path,
    latest_build_info: Option<&BuildInfo>,
) -> bool {
    if site_data.force_render {
        return true;
    }

    let templates_path = site_data.site.get_templates_path(input_folder);
    let templates_modified = WalkDir::new(&templates_path)
        .into_iter()
        .filter_map(Result::ok)
        .any(|entry| {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified_time) = metadata.modified() {
                    return modified_time
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_else(|e| {
                            error!("Failed to get duration since UNIX_EPOCH: {e}");
                            std::time::Duration::ZERO
                        })
                        .as_secs() as i64
                        > last_build;
                }
            }
            true
        });

    let fragments_modified = WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let file_name = e
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(
                    e.path()
                        .to_str()
                        .unwrap_or_else(|| panic!("Could not get file name {e:?}")),
                );
            let file_extension = e.path().extension().and_then(|ext| ext.to_str());
            e.path().is_file() && file_extension == Some("md") && file_name.starts_with('_')
        })
        .any(|entry| {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified_time) = metadata.modified() {
                    return modified_time
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_else(|e| {
                            error!("Failed to get duration since UNIX_EPOCH: {e}");
                            std::time::Duration::ZERO
                        })
                        .as_secs() as i64
                        > last_build;
                }
            }
            true
        });

    let config_modified = latest_build_info
        .as_ref()
        .is_none_or(|info| info.config != site_data.site);

    templates_modified || fragments_modified || config_modified
}

#[allow(clippy::similar_names)]
fn handle_404(
    content_dir: &Path,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
    highlighter: Option<&MarmiteHighlighter>,
) -> Result<(), String> {
    let input_404_path = content_dir.join("_404.md");
    let mut context = global_context.clone();
    let mut content = ContentBuilder::default()
        .html("Page not found :/".to_string())
        .title("Page not found".to_string())
        .slug("404".to_string())
        .build();
    if input_404_path.exists() {
        let custom_content = Content::from_markdown(
            &input_404_path,
            None,
            &Marmite::default(),
            None,
            highlighter,
            None,
            None,
        )?;
        content.html.clone_from(&custom_content.html);
        content.title.clone_from(&custom_content.title);
    }
    context.insert("title", &content.title);
    context.insert("content", &content);
    context.insert("current_page", "404.html");
    render_html("content.html", "404.html", tera, &context, output_dir)?;
    Ok(())
}

fn handle_tag_pages(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    site_data
        .tag
        .iter()
        // IMPORTANT: Filter to only process slugified keys (for backward compatibility,
        // we store under both original and slugified keys, but only generate pages for slugified ones)
        .filter(|(key, _)| crate::slugify::slugify(key) == **key)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(tag_slug, tagged_contents)| -> Result<(), String> {
            // tag_slug is the slugified version from the HashMap key
            // we need to find the original tag name from the content
            // We look in the unfiltered content first to ensure we find the original tag
            let original_tag = tagged_contents
                .iter()
                .find_map(|content| {
                    content
                        .tags
                        .iter()
                        .find(|t| crate::slugify::slugify(t) == tag_slug.as_str())
                        .cloned()
                })
                .unwrap_or_else(|| (*tag_slug).clone());

            debug!("Tag slug: '{tag_slug}' -> Original tag: '{original_tag}'");

            let filename = format!("tag-{tag_slug}");
            // Filter out draft content
            let filtered_contents: Vec<Content> = tagged_contents
                .iter()
                .filter(|content| content.stream.as_deref() != Some("draft"))
                .cloned()
                .collect();
            handle_list_page(
                global_context,
                &site_data
                    .site
                    .tags_content_title
                    .replace("$tag", &original_tag),
                &filtered_contents,
                site_data,
                tera,
                output_dir,
                &filename,
            )?;
            // Render tag-{tag}.rss for each stream
            crate::feed::generate_rss(
                &filtered_contents,
                output_dir,
                &filename.clone(),
                &site_data.site,
            )?;

            if site_data.site.json_feed {
                crate::feed::generate_json(
                    &filtered_contents,
                    output_dir,
                    &filename,
                    &site_data.site,
                )?;
            }
            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))?;

    // Render tags.html group page
    let mut tag_list_context = global_context.clone();
    tag_list_context.insert("title", &site_data.site.tags_title);
    tag_list_context.insert("current_page", "tags.html");
    tag_list_context.insert("kind", "tag");
    render_html(
        "group.html",
        "tags.html",
        tera,
        &tag_list_context,
        output_dir,
    )?;
    Ok(())
}

fn handle_archive_pages(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    site_data
        .archive
        .iter()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(year, archive_contents)| -> Result<(), String> {
            let filename = format!("archive-{year}");
            // Filter out draft content
            let filtered_contents: Vec<Content> = archive_contents
                .iter()
                .filter(|content| content.stream.as_deref() != Some("draft"))
                .cloned()
                .collect();
            handle_list_page(
                global_context,
                &site_data.site.archives_content_title.replace("$year", year),
                &filtered_contents,
                site_data,
                tera,
                output_dir,
                &filename,
            )?;
            // Render archive-{year}.rss for each stream
            crate::feed::generate_rss(
                &filtered_contents,
                output_dir,
                &filename.clone(),
                &site_data.site,
            )?;

            if site_data.site.json_feed {
                crate::feed::generate_json(
                    &filtered_contents,
                    output_dir,
                    &filename,
                    &site_data.site,
                )?;
            }
            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))?;

    // Render archive.html group page
    let mut archive_context = global_context.clone();
    archive_context.insert("title", &site_data.site.archives_title);
    archive_context.insert("current_page", "archive.html");
    archive_context.insert("kind", "archive");
    render_html(
        "group.html",
        "archive.html",
        tera,
        &archive_context,
        output_dir,
    )?;

    Ok(())
}

impl UrlCollection {
    pub fn add_url(&mut self, category: &str, url: String) {
        match category {
            "posts" => self.posts.push(url),
            "pages" => self.pages.push(url),
            "tags" => self.tags.push(url),
            "authors" => self.authors.push(url),
            "series" => self.series.push(url),
            "streams" => self.streams.push(url),
            "archives" => self.archives.push(url),
            "languages" => self.languages.push(url),
            "feeds" => self.feeds.push(url),
            "pagination" => self.pagination.push(url),
            "file_mappings" => self.file_mappings.push(url),
            "redirects" => self.redirects.push(url),
            _ => self.misc.push(url),
        }
    }

    pub fn get_all_urls(&self) -> Vec<String> {
        let mut all_urls = Vec::new();
        all_urls.extend(self.posts.iter().cloned());
        all_urls.extend(self.pages.iter().cloned());
        all_urls.extend(self.tags.iter().cloned());
        all_urls.extend(self.authors.iter().cloned());
        all_urls.extend(self.series.iter().cloned());
        all_urls.extend(self.streams.iter().cloned());
        all_urls.extend(self.archives.iter().cloned());
        all_urls.extend(self.languages.iter().cloned());
        all_urls.extend(self.feeds.iter().cloned());
        all_urls.extend(self.pagination.iter().cloned());
        all_urls.extend(self.file_mappings.iter().cloned());
        all_urls.extend(self.misc.iter().cloned());
        // Redirects are intentionally excluded from get_all_urls
        // so they don't appear in the sitemap (redirect pages should not be indexed)
        all_urls
    }

    pub fn total_count(&self) -> usize {
        self.posts.len()
            + self.pages.len()
            + self.tags.len()
            + self.authors.len()
            + self.series.len()
            + self.streams.len()
            + self.archives.len()
            + self.languages.len()
            + self.feeds.len()
            + self.pagination.len()
            + self.file_mappings.len()
            + self.redirects.len()
            + self.misc.len()
    }
}

#[derive(Default)]
struct PostProcessors<'a> {
    shortcode_processor: Option<&'a ShortcodeProcessor>,
    site_data: Option<&'a Data>,
    highlighter: Option<&'a MarmiteHighlighter>,
    cross_site_data: Option<&'a crate::workspace::CrossSiteData>,
}

fn render_html(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
) -> Result<(), String> {
    render_html_with_shortcodes(
        template,
        filename,
        tera,
        context,
        output_dir,
        &PostProcessors::default(),
    )
}

fn render_html_with_shortcodes(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
    processors: &PostProcessors<'_>,
) -> Result<(), String> {
    let templates = template.split(',').collect::<Vec<_>>();
    let template = templates
        .iter()
        .find(|t| tera.get_template_names().any(|n| n == **t))
        .unwrap_or(&templates[0]);
    let mut rendered = tera.render(template, context).map_err(|e| {
        error!("Error rendering template `{template}` -> {filename}: {e:#?}");
        e.to_string()
    })?;

    if let Some(processor) = processors.shortcode_processor {
        debug!("Processing shortcodes for {filename}");
        rendered = processor.process_shortcodes(&rendered, context, tera, processors.highlighter);
    } else {
        debug!("No shortcode processor available for {filename}");
    }

    if let Some(data) = processors.site_data {
        debug!("Processing wikilinks for {filename}");
        rendered = fix_wikilinks(&rendered, data);
    }

    if let Some(cross_site) = processors.cross_site_data {
        debug!("Resolving cross-site references for {filename}");
        rendered = crate::workspace::resolve_cross_site_refs(&rendered, cross_site);
    }

    let output_file = output_dir.join(filename);
    fs::write(&output_file, rendered).map_err(|e| e.to_string())?;
    info!("Generated {}", &output_file.display());
    Ok(())
}

/// Initialize a new site in the input folder
#[allow(clippy::too_many_lines)]
pub fn initialize(input_folder: &Arc<std::path::PathBuf>, cli_args: &Arc<crate::cli::Cli>) {
    let input_folder = input_folder.as_path();
    let content_folder = input_folder.join("content");
    let media_folder = content_folder.join("media");

    if let Err(e) = fs::create_dir_all(input_folder) {
        error!("Failed to create input folder: {e:?}");
        process::exit(1);
    }
    let has_visible_files = input_folder
        .read_dir()
        .map_err(|e| {
            error!("Failed to read input folder: {e}");
            process::exit(1);
        })
        .unwrap_or_else(|()| std::process::exit(1))
        .any(|entry| {
            entry
                .ok()
                .and_then(|e| e.file_name().to_str().map(|n| !n.starts_with('.')))
                .unwrap_or(false)
        });
    if has_visible_files {
        error!("Input folder is not empty: {}", input_folder.display());
        process::exit(1);
    }
    crate::config::generate(input_folder, cli_args);
    if let Err(e) = fs::create_dir(&content_folder) {
        error!("Failed to create 'content' folder: {e:?}");
        process::exit(1);
    }
    if let Err(e) = fs::create_dir(&media_folder) {
        error!("Failed to create 'content/media' folder: {e:?}");
        process::exit(1);
    }
    let pages_folder = content_folder.join("pages");
    if let Err(e) = fs::create_dir(&pages_folder) {
        error!("Failed to create 'content/pages' folder: {e:?}");
        process::exit(1);
    }
    let posts_folder = content_folder.join("posts");
    if let Err(e) = fs::create_dir(&posts_folder) {
        error!("Failed to create 'content/posts' folder: {e:?}");
        process::exit(1);
    }
    // create input_folder/custom.css with `/* Custom CSS */` content
    if let Err(e) = fs::write(input_folder.join("custom.css"), "/* Custom CSS */") {
        error!("Failed to create 'custom.css' file: {e:?}");
        process::exit(1);
    }
    // create input_folder/custom.js with `// Custom JS` content
    if let Err(e) = fs::write(input_folder.join("custom.js"), "// Custom JS") {
        error!("Failed to create 'custom.js' file: {e:?}");
        process::exit(1);
    }
    // create content/_404.md with `# Not Found` content
    if let Err(e) = fs::write(content_folder.join("_404.md"), "# Not Found") {
        error!("Failed to create 'content/_404.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_references.md with `[marmite]: https://github.com/rochacbruno/marmite` content
    if let Err(e) = fs::write(
        content_folder.join("_references.md"),
        "[github]: https://github.com/rochacbruno/marmite",
    ) {
        error!("Failed to create 'content/_references.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_markdown_header.md with `<!-- Content Injected to every content markdown header -->` content
    if let Err(e) = fs::write(
        content_folder.join("_markdown_header.md"),
        "<!-- Content Injected to every content markdown header -->",
    ) {
        error!("Failed to create 'content/_markdown_header.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_markdown_footer.md with `<!-- Content Injected to every content markdown footer -->` content
    if let Err(e) = fs::write(
        content_folder.join("_markdown_footer.md"),
        "<!-- Content Injected to every content markdown footer -->",
    ) {
        error!("Failed to create 'content/_markdown_footer.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_announce.md with `Give us a &star; on [github]` content
    if let Err(e) = fs::write(
        content_folder.join("_announce.md"),
        "Give us a &star; on [github]",
    ) {
        error!("Failed to create 'content/_announce.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_sidebar.md with `<!-- Sidebar content -->` content
    let side_bar_content = "
    {% set groups = ['tag', 'archive', 'author'] %}\n\
    {% for group in groups %}\n\
    \n\
    ##### {{group}}s\n\
    \n\
    {% for name, items in group(kind=group) -%}\n\
    - [{{name}}]({{group}}-{{name | slugify}}.html)\n\
    {% endfor %}\n\
    \n\
    {% endfor %}\n\
    \n\
    #### Streams\n\
    \n\
    {% for name, items in group(kind='stream') -%}\n\
    \n\
    - [{{name}}]({{name | slugify}}.html)\n\
    \n\
    {% endfor %}\n\n\
    ";
    if let Err(e) = fs::write(content_folder.join("_sidebar.example.md"), side_bar_content) {
        error!("Failed to create 'content/_sidebar.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_comments.md with `<!-- Comments -->` content
    if let Err(e) = fs::write(
        content_folder.join("_comments.md"),
        "##### Comments\n\
        **edit `content/_comments.md` to adjust for your own site/repo**\n\n\
        **remove** the file to disable comments\n\
        \n\
        <script src='https://utteranc.es/client.js'\n\
        repo='rochacbruno/issue-bin'\n\
        issue-term='pathname'\n\
        theme='preferred-color-scheme'\n\
        crossorigin='anonymous'\n\
        async>\n\
        </script>\n\
        ",
    ) {
        error!("Failed to create 'content/_comments.md' file: {e:?}");
        process::exit(1);
    }
    // create content/_hero.md with `<!-- Hero content -->` content
    if let Err(e) = fs::write(
        content_folder.join("_hero.md"),
        "##### Welcome to Marmite\n\
        \n\
        Marmite is a static site generator written in Rust.\n\
        edit `content/_hero.md` to change this content.\n\
        remove the file to disable the hero section.\n\
        ",
    ) {
        error!("Failed to create 'content/_hero.md' file: {e:?}");
        process::exit(1);
    }
    // create content/pages/about.md with `# About` content
    if let Err(e) = fs::write(
        pages_folder.join("about.md"),
        "# About\n\
        \n\
        Hi, edit `content/pages/about.md` to change this content.\n\
        \n\
        Pages are content without a date. They do not appear in feeds or the index.\n\
        Add them to the menu in `marmite.yaml` to make them accessible.\n\
        \n\
        To see all your pages take a look at [[pages]]\n\n\
        ",
    ) {
        error!("Failed to create 'content/pages/about.md' file: {e:?}");
        process::exit(1);
    }
    // create content/posts/welcome.md with date in frontmatter
    let now = chrono::Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    if let Err(e) = fs::write(
        posts_folder.join("welcome.md"),
        format!(
            "---\n\
            date: {now_str}\n\
            ---\n\
            # Welcome to Marmite\n\
            \n\
            This is your first post!\n\
            \n\
            ## Edit this content\n\n\
            Edit `content/posts/welcome.md` to change this post.\n\n\
            ## Add more content\n\n\
            Create new markdown files in `content/posts/` for posts \
            or `content/pages/` for pages.\n\n\
            Use `marmite --new \"Post Title\"` to create a new post, \
            add `-d posts` to place it in the posts folder.\n\n\
            ## Customize your site\n\n\
            Edit `marmite.yaml` to change site settings.\n\n\
            Edit the files starting with `_` in the `content` folder to change the layout.\n\n\
            ## Deploy your site\n\n\
            Read more on [marmite documentation](https://marmite.blog).\n\n\
            "
        ),
    ) {
        error!("Failed to create 'content/posts/welcome.md' file: {e:?}");
        process::exit(1);
    }
    info!("Site initialized in {}", input_folder.display());
}

pub fn initialize_project(input_folder: &std::path::Path) -> Result<(), String> {
    let content_folder = input_folder.join("content");
    let media_folder = content_folder.join("media");
    let pages_folder = content_folder.join("pages");
    let posts_folder = content_folder.join("posts");

    fs::create_dir_all(input_folder).map_err(|e| format!("Failed to create input folder: {e}"))?;

    // Determine the output folder name to ignore it when checking emptiness.
    // Read from config if it exists, otherwise use the default ("site").
    let config_path = input_folder.join("marmite.yaml");
    let output_name = if config_path.exists() {
        let data = Data::from_file(&config_path);
        let sp = &data.site.site_path;
        if sp.is_empty() {
            "site".to_string()
        } else {
            sp.clone()
        }
    } else {
        "site".to_string()
    };

    let has_visible_files = input_folder
        .read_dir()
        .map_err(|e| format!("Failed to read input folder: {e}"))?
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|n| !n.starts_with('.') && n != output_name)
        });

    if has_visible_files {
        return Err(format!(
            "Input folder is not empty: {}",
            input_folder.display()
        ));
    }

    crate::config::generate_default_config(input_folder)
        .map_err(|e| format!("Failed to generate config: {e}"))?;

    for dir in [&content_folder, &media_folder, &pages_folder, &posts_folder] {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {e}", dir.display()))?;
    }

    let files: &[(&str, &str)] = &[
        ("custom.css", "/* Custom CSS */"),
        ("custom.js", "// Custom JS"),
    ];
    for (name, content) in files {
        fs::write(input_folder.join(name), content)
            .map_err(|e| format!("Failed to create {name}: {e}"))?;
    }

    let content_files: &[(&str, &str)] = &[
        ("_404.md", "# Not Found"),
        (
            "_references.md",
            "[github]: https://github.com/rochacbruno/marmite",
        ),
        ("_hero.md", "##### Welcome to Marmite\n\nMarmite is a static site generator written in Rust.\nEdit `content/_hero.md` to change this content.\nRemove the file to disable the hero section.\n"),
        ("_announce.md", "Give us a &star; on [github]"),
    ];
    for (name, content) in content_files {
        fs::write(content_folder.join(name), content)
            .map_err(|e| format!("Failed to create content/{name}: {e}"))?;
    }

    fs::write(
        pages_folder.join("about.md"),
        "# About\n\nEdit `content/pages/about.md` to change this content.\n\nPages are content without a date.\nAdd them to the menu in `marmite.yaml` to make them accessible.\n",
    )
    .map_err(|e| format!("Failed to create about.md: {e}"))?;

    let now = chrono::Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    fs::write(
        posts_folder.join("welcome.md"),
        format!(
            "---\ndate: {now_str}\n---\n# Welcome to Marmite\n\nThis is your first post!\n\nEdit `content/posts/welcome.md` to change this post.\nCreate new markdown files in `content/posts/` for posts or `content/pages/` for pages.\n"
        ),
    )
    .map_err(|e| format!("Failed to create welcome.md: {e}"))?;

    Ok(())
}

/// Show all site URLs in JSON format
#[allow(clippy::too_many_lines)]
pub fn show_urls(
    config_path: &Arc<std::path::PathBuf>,
    input_folder: &Arc<std::path::PathBuf>,
    args: &Arc<crate::cli::Cli>,
) {
    // Load site data from config
    let mut site_data = Data::from_file(config_path.as_path());
    let content_folder = get_content_folder(&site_data.site, input_folder.as_path());

    // Override site config with CLI arguments
    site_data.site.override_from_cli_args(args);

    // Collect content fragments and process content
    let fragments = collect_content_fragments(&content_folder);
    let folder_defaults = load_folder_frontmatter(&content_folder);
    collect_content(
        &content_folder,
        &mut site_data,
        &fragments,
        None,
        &folder_defaults,
    );
    site_data.sort_all();

    // Collect all URLs including pagination, feeds, and file mappings
    site_data.collect_all_urls();

    // Generate JSON using the shared function
    let json = create_urls_json(&site_data, "");

    // Output JSON
    match serde_json::to_string_pretty(&json) {
        Ok(json_string) => println!("{json_string}"),
        Err(e) => error!("Failed to serialize URLs to JSON: {e}"),
    }
}

#[cfg(test)]
#[path = "tests/site.rs"]
mod tests;
