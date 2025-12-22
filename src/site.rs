use crate::config::{Author, Marmite};
use crate::content::{check_for_duplicate_slugs, Content, ContentBuilder, GroupedContent, Kind};
use crate::embedded::{generate_static, Templates, EMBEDDED_TERA};
use crate::gallery::Gallery;
use crate::image_resize;
use crate::parser::fix_wikilinks;
use crate::shortcodes::ShortcodeProcessor;
use crate::tera_functions::{
    DisplayName, GetDataBySlug, GetGallery, GetPosts, Group, SourceLink, UrlFor,
};
use crate::{re, server, tera_filter};
use chrono::Datelike;
use core::str;
use fs_extra::dir::{copy as dircopy, CopyOptions};
use glob::glob;
use hotwatch::{Event, EventKind, Hotwatch};
use log::{debug, error, info};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::vec;
use std::{fs, process, sync::Arc, sync::Mutex};
use tera::{Context, Tera};
use tera::{Function, Value};
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
    pub feeds: Vec<String>,
    pub pagination: Vec<String>,
    pub file_mappings: Vec<String>,
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
        self.posts.sort_by(|a, b| b.date.cmp(&a.date));
        self.pages.sort_by(|a, b| b.title.cmp(&a.title));
        self.tag.sort_all();
        self.archive.sort_all();
        self.author.sort_all();
        self.stream.sort_all();
        self.series.sort_all();
    }

    /// takes content then classifies the content
    /// into posts, pages, tags, authors, archive, stream
    /// and adds the content to the respective fields in self
    pub fn push_content(&mut self, content: Content) {
        if let Some(date) = content.date {
            self.posts.push(content.clone());
            // tags
            for tag in content.tags.clone() {
                let tag_slug = slug::slugify(&tag);
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
                    .entry(stream.to_string())
                    .or_default()
                    .push(content.clone());
            }

            // series by name
            if let Some(series) = &content.series {
                self.series
                    .entry(series.to_string())
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
            .filter(|(key, _)| slug::slugify(key) == key.as_str())
        {
            let tag_slug = format!("tag-{}.html", slug::slugify(tag.0));
            self.generated_urls.add_url("tags", tag_slug);

            // Add pagination for tags
            let content_count = tag.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 = format!("tag-{}-1.html", slug::slugify(tag.0));
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug =
                            format!("tag-{}-{}.html", slug::slugify(tag.0), page_num);
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
            let author_slug = format!("author-{}.html", slug::slugify(author.0));
            self.generated_urls.add_url("authors", author_slug);

            // Add pagination for authors
            let content_count = author.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 = format!("author-{}-1.html", slug::slugify(author.0));
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug =
                            format!("author-{}-{}.html", slug::slugify(author.0), page_num);
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
            let series_slug = format!("series-{}.html", slug::slugify(series.0));
            self.generated_urls.add_url("series", series_slug);

            // Add pagination for series
            let content_count = series.1.len();
            if content_count > 0 {
                // Always add -1 page (same as base page but with consistent naming)
                let pagination_slug_1 = format!("series-{}-1.html", slug::slugify(series.0));
                self.generated_urls.add_url("pagination", pagination_slug_1);

                // Add additional pagination pages if content exceeds pagination limit
                if content_count > self.site.pagination {
                    let total_pages = content_count.div_ceil(self.site.pagination);
                    for page_num in 2..=total_pages {
                        let pagination_slug =
                            format!("series-{}-{}.html", slug::slugify(series.0), page_num);
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
                let stream_slug = format!("{}.html", slug::slugify(stream.0));
                self.generated_urls.add_url("streams", stream_slug);
            }

            // Add pagination for streams (skip "index" stream as it's handled separately)
            if stream.0 != "index" {
                let content_count = stream.1.len();
                if content_count > 0 {
                    // Always add -1 page (same as base page but with consistent naming)
                    let pagination_slug_1 = format!("{}-1.html", slug::slugify(stream.0));
                    self.generated_urls.add_url("pagination", pagination_slug_1);

                    // Add additional pagination pages if content exceeds pagination limit
                    if content_count > self.site.pagination {
                        let total_pages = content_count.div_ceil(self.site.pagination);
                        for page_num in 2..=total_pages {
                            let pagination_slug =
                                format!("{}-{}.html", slug::slugify(stream.0), page_num);
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
                let feed_slug = format!("{}.rss", slug::slugify(stream.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Series feeds
            for series in self.series.iter() {
                let feed_slug = format!("series-{}.rss", slug::slugify(series.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Tag feeds
            // Filter to only process slugified keys (avoid duplicates from backward compatibility layer)
            for tag in self
                .tag
                .iter()
                .filter(|(key, _)| slug::slugify(key) == key.as_str())
            {
                let feed_slug = format!("tag-{}.rss", slug::slugify(tag.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Author feeds
            for author in self.author.iter() {
                let feed_slug = format!("author-{}.rss", slug::slugify(author.0));
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
                let feed_slug = format!("{}.json", slug::slugify(stream.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Series feeds
            for series in self.series.iter() {
                let feed_slug = format!("series-{}.json", slug::slugify(series.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Tag feeds
            // Filter to only process slugified keys (avoid duplicates from backward compatibility layer)
            for tag in self
                .tag
                .iter()
                .filter(|(key, _)| slug::slugify(key) == key.as_str())
            {
                let feed_slug = format!("tag-{}.json", slug::slugify(tag.0));
                self.generated_urls.add_url("feeds", feed_slug);
            }

            // Author feeds
            for author in self.author.iter() {
                let feed_slug = format!("author-{}.json", slug::slugify(author.0));
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
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct BuildInfo {
    marmite_version: String,
    posts: usize,
    pages: usize,
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
            let build_info_path = moved_output_folder.join("marmite.json");
            let latest_build_info = get_latest_build_info(&build_info_path)?;
            if let Some(build_info) = &latest_build_info {
                site_data.latest_timestamp = Some(build_info.timestamp);
            }
            site_data.site.override_from_cli_args(&moved_cli_args);
            if moved_cli_args.force {
                site_data.force_render = true;
            }

            let fragments = collect_content_fragments(&content_folder);
            collect_content(&content_folder, &mut site_data, &fragments);

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
                        initialize_tera(&moved_input_folder, &site_data);
                    if let Err(e) = render_templates(
                        &content_folder,
                        &site_data,
                        &tera,
                        &output_path,
                        &moved_input_folder,
                        &fragments,
                        latest_build_info.as_ref(),
                        shortcode_processor.as_ref(),
                    ) {
                        error!("Failed to render templates: {e:?}");
                        process::exit(1);
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
                "copy_markdown_sources" => {
                    if site_data.site.publish_md {
                        copy_markdown_sources(&site_data, &content_folder, &output_path);
                    }
                }
                _ => {}
            });

            // Generate sitemap after all templates are rendered
            let (tera, _) = initialize_tera(&moved_input_folder, &site_data);
            generate_sitemap(&site_data, &tera, &output_path);

            // Generate urls.json if enabled
            if site_data.site.publish_urls_json {
                generate_urls_json(&site_data, &output_path);
            }

            let end_time = start_time.elapsed().as_secs_f64();
            write_build_info(&output_path, &site_data, end_time);
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
            server::start(
                bind_address,
                &Arc::clone(output_folder),
                live_reload.as_ref(),
            );
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
fn collect_content_fragments(content_dir: &Path) -> HashMap<String, String> {
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

/// Collect global fragments of markdown, process them and insert into the global context
/// these are dynamic parts of text that will be processed by Tera
fn collect_global_fragments(
    content_dir: &Path,
    global_context: &mut Context,
    tera: &Tera,
    site_config: &Marmite,
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
            .render_str(&fragment_content, global_context)
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
            crate::parser::get_html_with_options(&rendered_fragment, parser_options);
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
fn collect_back_links(site_data: &mut std::sync::MutexGuard<'_, Data>) {
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

#[allow(clippy::cast_possible_wrap)]
fn set_next_and_previous_links(site_data: &mut std::sync::MutexGuard<'_, Data>) {
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
        posts.sort_by(|a, b| a.date.cmp(&b.date));
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

#[allow(clippy::cast_possible_wrap)]
fn collect_content(
    content_dir: &std::path::PathBuf,
    site_data: &mut Data,
    fragments: &HashMap<String, String>,
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
            e.path().is_file() && file_extension == Some("md") && !file_name.starts_with('_')
        })
        .map(|entry| {
            // let modified_time = entry.metadata().unwrap().modified().unwrap();
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
                // get the timestamp for modified time
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
            Content::from_markdown(
                entry.path(),
                Some(fragments),
                &site_data.site,
                modified_time,
            )
        })
        .collect::<Vec<_>>();
    for content in contents {
        match content {
            Ok(content) => {
                site_data.push_content(content);
            }
            Err(e) => {
                error!("Failed to process content: {e:?}");
            }
        }
    }
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
fn initialize_tera(input_folder: &Path, site_data: &Data) -> (Tera, Option<ShortcodeProcessor>) {
    let mut tera = Tera::default();
    tera.autoescape_on(vec![]);
    tera.register_function(
        "url_for",
        UrlFor {
            base_url: site_data.site.url.to_string(),
        },
    );
    tera.register_function(
        "group",
        Group {
            site_data: site_data.clone(),
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
        "get_posts",
        GetPosts {
            site_data: site_data.clone(),
        },
    );
    tera.register_function(
        "get_data_by_slug",
        GetDataBySlug {
            site_data: site_data.clone(),
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
            date_format: site_data.site.default_date_format.to_string(),
        },
    );
    tera.register_filter("remove_draft", tera_filter::RemoveDraft);

    let templates_path = site_data.site.get_templates_path(input_folder);
    let mandatory_templates = ["base.html", "list.html", "group.html", "content.html"];
    // Required because Tera needs base templates to be loaded before extending them
    for template_name in &mandatory_templates {
        let template_path = templates_path.join(template_name);
        if template_path.exists() {
            let template_content = fs::read_to_string(&template_path).unwrap_or_else(|e| {
                error!("Failed to read template {template_name}: {e}");
                String::new()
            });
            if let Err(e) = tera.add_raw_template(template_name, &template_content) {
                error!("Failed to load template {template_name}: {e}");
            }
        } else {
            Templates::get(template_name).map_or_else(
                || {
                    error!("Failed to load template: {template_name}");
                    process::exit(1);
                },
                |template| {
                    let template_str =
                        std::str::from_utf8(template.data.as_ref()).unwrap_or_else(|e| {
                            error!("Failed to parse template {template_name}: {e}");
                            ""
                        });
                    if let Err(e) = tera.add_raw_template(template_name, template_str) {
                        error!("Failed to load template {template_name}: {e}");
                    }
                },
            );
        }
    }

    // For every template file in the templates folder including subfolders
    // we will load it into the tera instance one by one
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
        // windows compatibility
        let template_name = template_name.replace('\\', "/");
        let template_name = template_name.trim_start_matches('/');
        let template_content = fs::read_to_string(template_path).unwrap_or_else(|e| {
            error!("Failed to read template {template_name}: {e}");
            String::new()
        });
        if let Err(e) = tera.add_raw_template(template_name, &template_content) {
            error!("Failed to load template {template_name}: {e}");
        }
    }

    // Now extend the remaining templates from the embedded::Templates struct
    if let Err(e) = tera.extend(&EMBEDDED_TERA) {
        error!("Failed to extend with embedded templates: {e}");
    }

    // Initialize shortcode processor if enabled
    let shortcode_processor = if site_data.site.enable_shortcodes {
        let mut processor = ShortcodeProcessor::new(site_data.site.shortcode_pattern.as_deref());
        if let Err(e) = processor.collect_shortcodes(input_folder) {
            error!("Failed to collect shortcodes: {e}");
        }
        // Add shortcodes to Tera
        if let Err(e) = processor.add_shortcodes_to_tera(&mut tera) {
            error!("Failed to add shortcodes to Tera: {e}");
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
) -> Result<(), String> {
    // Build the context of variables that are global on every template
    let mut global_context = Context::new();
    global_context.insert("markdown_fragments", &fragments);
    let site_data = site_data.clone();

    global_context.insert("site_data", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);
    global_context.insert("language", &site_data.site.language);
    debug!("Global Context site: {:?}", &site_data.site);
    debug!("Site data galleries count: {}", site_data.galleries.len());
    collect_global_fragments(content_dir, &mut global_context, tera, &site_data.site);

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
    handle_404(content_dir, &global_context, tera, output_dir)?;

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
    )?;

    Ok(())
}

fn handle_group_pages(
    global_context: &Context,
    site_data: &Data,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    ["tags", "archives", "authors", "streams", "series"]
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
            let stream_slug = slug::slugify(stream);
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
            let series_slug = format!("series-{}", slug::slugify(series));
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
                name: (*username).to_string(),
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

            let author_slug = slug::slugify(username);
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

#[allow(clippy::too_many_lines)]
fn handle_static_artifacts(
    input_folder: &Path,
    site_data: &Data,
    output_folder: &Arc<std::path::PathBuf>,
    content_dir: &std::path::Path,
) {
    let static_source = site_data.site.get_static_path(input_folder);
    if static_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

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
        generate_static(&output_folder.join(site_data.site.static_path.clone()));
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

        // Process image resizing if configured in extra (and not skipped)
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

fn write_build_info(
    output_path: &Path,
    site_data: &std::sync::MutexGuard<'_, Data>,
    end_time: f64,
) {
    let build_info = BuildInfo {
        marmite_version: env!("CARGO_PKG_VERSION").to_string(),
        posts: site_data.posts.len(),
        pages: site_data.pages.len(),
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
    };

    // Helper to generate URL using url_for
    let generate_url = |path: &str| -> String {
        let mut args = HashMap::new();
        args.insert("path".to_string(), Value::String(path.to_string()));

        if site_data.site.url.is_empty() {
            // Use relative URLs
            match url_for.call(&args) {
                Ok(Value::String(url)) => url,
                _ => format!("/{path}"),
            }
        } else {
            // Use absolute URLs
            args.insert("abs".to_string(), Value::Bool(true));
            match url_for.call(&args) {
                Ok(Value::String(url)) => url,
                _ => format!("{}/{}", site_data.site.url.trim_end_matches('/'), path),
            }
        }
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
fn create_urls_json(site_data: &Data) -> serde_json::Value {
    // Create UrlFor function instance
    let url_for = UrlFor {
        base_url: site_data.site.url.clone(),
    };

    // Determine if we should use absolute URLs
    let use_abs = !site_data.site.url.is_empty();

    // Helper to generate URL using url_for
    let generate_url = |path: &str| -> String {
        let mut args = HashMap::new();
        args.insert("path".to_string(), Value::String(path.to_string()));

        if use_abs {
            args.insert("abs".to_string(), Value::Bool(true));
        }

        match url_for.call(&args) {
            Ok(Value::String(url)) => url,
            _ => {
                if use_abs {
                    format!("{}/{}", site_data.site.url.trim_end_matches('/'), path)
                } else {
                    format!("/{path}")
                }
            }
        }
    };

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

fn generate_urls_json(site_data: &Data, output_path: &Path) {
    if !site_data.site.publish_urls_json {
        return;
    }

    let json = create_urls_json(site_data);

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
                .map(|name| format!(
                    "{}:{}",
                    name,
                    context.get(name).unwrap_or(&tera::Value::Null)
                ))
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
fn handle_content_pages(
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
    content_dir: &Path,
    input_folder: &Path,
    latest_build_info: Option<&BuildInfo>,
    shortcode_processor: Option<&ShortcodeProcessor>,
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

            render_html_with_shortcodes(
                "content.html",
                &format!("{}.html", &content.slug),
                tera,
                &content_context,
                output_dir,
                shortcode_processor,
                Some(site_data),
            )
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
) -> Result<(), String> {
    let input_404_path = content_dir.join("_404.md");
    let mut context = global_context.clone();
    let mut content = ContentBuilder::default()
        .html("Page not found :/".to_string())
        .title("Page not found".to_string())
        .slug("404".to_string())
        .build();
    if input_404_path.exists() {
        let custom_content =
            Content::from_markdown(&input_404_path, None, &Marmite::default(), None)?;
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
        .filter(|(key, _)| slug::slugify(key) == **key)
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
                        .find(|t| slug::slugify(t) == tag_slug.as_str())
                        .cloned()
                })
                .unwrap_or_else(|| (*tag_slug).to_string());

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
            "feeds" => self.feeds.push(url),
            "pagination" => self.pagination.push(url),
            "file_mappings" => self.file_mappings.push(url),
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
        all_urls.extend(self.feeds.iter().cloned());
        all_urls.extend(self.pagination.iter().cloned());
        all_urls.extend(self.file_mappings.iter().cloned());
        all_urls.extend(self.misc.iter().cloned());
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
            + self.feeds.len()
            + self.pagination.len()
            + self.file_mappings.len()
            + self.misc.len()
    }
}

fn render_html(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
) -> Result<(), String> {
    render_html_with_shortcodes(template, filename, tera, context, output_dir, None, None)
}

fn render_html_with_shortcodes(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
    shortcode_processor: Option<&ShortcodeProcessor>,
    site_data: Option<&Data>,
) -> Result<(), String> {
    let templates = template.split(',').collect::<Vec<_>>();
    let template = templates
        .iter()
        .find(|t| tera.get_template(t).is_ok())
        .unwrap_or(&templates[0]);
    let mut rendered = tera.render(template, context).map_err(|e| {
        error!("Error rendering template `{template}` -> {filename}: {e:#?}");
        e.to_string()
    })?;

    // Process shortcodes if processor is available
    if let Some(processor) = shortcode_processor {
        debug!("Processing shortcodes for {filename}");
        rendered = processor.process_shortcodes(&rendered, context, tera);
    } else {
        debug!("No shortcode processor available for {filename}");
    }

    // Process wikilinks if site data is available
    if let Some(data) = site_data {
        debug!("Processing wikilinks for {filename}");
        rendered = fix_wikilinks(&rendered, data);
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
    if input_folder
        .read_dir()
        .map_err(|e| {
            error!("Failed to read input folder: {e}");
            process::exit(1);
        })
        .unwrap_or_else(|()| std::process::exit(1))
        .next()
        .is_some()
    {
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
    {% set groups = ['tag', 'archive', 'author', 'stream'] %}\n\
    {% for group in groups %}\n\
    \n\
    ##### {{group}}s\n\
    \n\
    {% for name, items in group(kind=group) -%}\n\
    - [{{name}}]({{group}}-{{name | slugify}}.html)\n\
    {% endfor %}\n\
    \n\
    {% endfor %}
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
    // create content/about.md with `# About` content
    if let Err(e) = fs::write(
        content_folder.join("about.md"),
        "# About\n\
        \n\
        Hi, edit `about.md` to change this content.
        ",
    ) {
        error!("Failed to create 'content/about.md' file: {e:?}");
        process::exit(1);
    }
    // create content/{now}-welcome.md with `# Welcome to Marmite` content
    let now = chrono::Local::now();
    let now = now.format("%Y-%m-%d").to_string();
    if let Err(e) = fs::write(
        content_folder.join(format!("{now}-welcome.md")),
        "# Welcome to Marmite\n\
        \n\
        This is your first post!\n\
        \n\
        ## Edit this content\n\n\
        edit on `content/{date}-welcome.md`\n\n\
        ## Add more content\n\n\
        create new markdown files in the `content` folder\n\n\
        use `marmite --new` to create new content\n\n\
        ## Customize your site\n\n\
        edit `marmite.yaml` to change site settings\n\n\
        edit the files starting with `_` in the `content` folder to change the layout\n\n\
        or edit the templates to create a custom layout\n\n\
        ## Deploy your site\n\n\
        read more on [marmite documentation](https://marmite.blog)\n\n\
        ",
    ) {
        error!("Failed to create 'content/{now}-welcome.md' file: {e:?}");
        process::exit(1);
    }
    info!("Site initialized in {}", input_folder.display());
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
    collect_content(&content_folder, &mut site_data, &fragments);
    site_data.sort_all();

    // Collect all URLs including pagination, feeds, and file mappings
    site_data.collect_all_urls();

    // Generate JSON using the shared function
    let json = create_urls_json(&site_data);

    // Output JSON
    match serde_json::to_string_pretty(&json) {
        Ok(json_string) => println!("{json_string}"),
        Err(e) => error!("Failed to serialize URLs to JSON: {e}"),
    }
}

#[cfg(test)]
#[path = "tests/site.rs"]
mod tests;
