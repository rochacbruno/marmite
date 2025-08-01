use crate::config::{Author, Marmite};
use crate::content::{
    check_for_duplicate_slugs, slugify, Content, ContentBuilder, GroupedContent, Kind,
};
use crate::embedded::{generate_static, Templates, EMBEDDED_TERA};
use crate::shortcodes::ShortcodeProcessor;
use crate::tera_functions::{DisplayName, Group, SourceLink, UrlFor};
use crate::{server, tera_filter};
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
use walkdir::WalkDir;

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
                self.tag.entry(tag).or_default().push(content.clone());
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
) {
    let moved_input_folder = Arc::clone(input_folder);
    let moved_output_folder = Arc::clone(output_folder);
    let moved_config_path = Arc::clone(config_path);
    let moved_cli_args = Arc::clone(cli_args);

    let rebuild = {
        move || {
            let start_time = std::time::Instant::now();
            let site_data = Arc::new(Mutex::new(Data::from_file(
                moved_config_path.clone().as_path(),
            )));
            let content_folder = get_content_folder(
                &site_data.lock().unwrap().site,
                moved_input_folder.clone().as_path(),
            );
            let mut site_data = site_data.lock().unwrap();
            let build_info_path = moved_output_folder.join("marmite.json");
            let latest_build_info = get_latest_build_info(&build_info_path);
            if let Some(build_info) = &latest_build_info {
                site_data.latest_timestamp = Some(build_info.timestamp);
            }
            site_data.site.override_from_cli_args(&moved_cli_args);
            if moved_cli_args.force {
                site_data.force_render = true;
            }

            let fragments = collect_content_fragments(&content_folder);
            collect_content(&content_folder, &mut site_data, &fragments);
            site_data.sort_all();
            detect_slug_collision(&site_data); // Detect slug collision and warn user
            collect_back_links(&mut site_data);
            set_next_and_previous_links(&mut site_data);

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

            let end_time = start_time.elapsed().as_secs_f64();
            write_build_info(&output_path, &site_data, end_time);
            debug!("Site generated in {end_time:.2}s");
            info!("Site generated at: {}/", moved_output_folder.display());
        }
    };

    // Initial site generation
    rebuild();

    if watch {
        let mut hotwatch = Hotwatch::new().expect("Failed to initialize hotwatch!");
        let watch_folder = Arc::clone(input_folder).as_path().to_path_buf();
        let out_folder = Arc::clone(output_folder).as_path().to_path_buf();
        // Watch the input folder for changes
        hotwatch
            .watch(watch_folder, move |event: Event| match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    for ev in &event.paths {
                        if !ev.starts_with(fs::canonicalize(out_folder.clone()).unwrap()) {
                            info!("Change detected. Rebuilding site...");
                            rebuild();
                        }
                    }
                }
                _ => {}
            })
            .expect("Failed to watch the input folder!");

        info!("Watching for changes in folder: {}", input_folder.display());

        // Keep the thread alive for watching
        if serve {
            info!("Starting built-in HTTP server...");
            server::start(bind_address, &Arc::clone(output_folder));
        } else {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

fn get_latest_build_info(build_info_path: &std::path::PathBuf) -> Option<BuildInfo> {
    let mut latest_build_info: Option<BuildInfo> = None;
    if build_info_path.exists() {
        let build_info_json = fs::read_to_string(build_info_path).unwrap();
        if let Ok(build_info) = BuildInfo::from_json(&build_info_json) {
            latest_build_info = Some(build_info);
        }
    }
    latest_build_info
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
                    fs::read_to_string(fragment_path).unwrap()
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
        let fragment_content = fs::read_to_string(fragment_path).unwrap();
        // append references
        let references_path = content_dir.join("_references.md");
        let fragment_content =
            crate::parser::append_references(&fragment_content, &references_path);
        let rendered_fragment = tera
            .clone()
            .render_str(&fragment_content, global_context)
            .unwrap();
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
            let file_metadata = entry.metadata().expect("Failed to get file metadata");
            let modified_time = if let Ok(modified_time) = file_metadata.modified() {
                // get the timestamp for modified time
                Some(
                    modified_time
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
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
            let template_content = fs::read_to_string(template_path).unwrap();
            tera.add_raw_template(template_name, &template_content)
                .expect("Failed to load template");
        } else {
            Templates::get(template_name).map_or_else(
                || {
                    error!("Failed to load template: {template_name}");
                    process::exit(1);
                },
                |template| {
                    let template_str = std::str::from_utf8(template.data.as_ref()).unwrap();
                    tera.add_raw_template(template_name, template_str)
                        .expect("Failed to load template");
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
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        // windows compatibility
        let template_name = template_name.replace('\\', "/");
        let template_name = template_name.trim_start_matches('/');
        let template_content = fs::read_to_string(template_path).unwrap();
        tera.add_raw_template(template_name, &template_content)
            .expect("Failed to load template");
    }

    // Now extend the remaining templates from the embedded::Templates struct
    tera.extend(&EMBEDDED_TERA).unwrap();

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
            let stream_slug = slugify(stream);
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
            let series_slug = format!("series-{}", slugify(series));
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

            let author_slug = slugify(username);
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
        let re = Regex::new(r"<[^>]*>|(\{\{[^>]*\}\})|(\{%[^>]*%\})").unwrap();
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
        serde_json::to_string(&all_content_json).unwrap(),
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
        serde_json::to_string_pretty(&build_info).unwrap(),
    ) {
        error!("Failed to write marmite.json: {e:?}");
    } else {
        info!("Generated build info at marmite.json");
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
                        .unwrap()
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
                        .unwrap()
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
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(tag, tagged_contents)| -> Result<(), String> {
            let tag_slug = slugify(tag);
            let filename = format!("tag-{}", &tag_slug);
            // Filter out draft content
            let filtered_contents: Vec<Content> = tagged_contents
                .iter()
                .filter(|content| content.stream.as_deref() != Some("draft"))
                .cloned()
                .collect();
            handle_list_page(
                global_context,
                &site_data.site.tags_content_title.replace("$tag", tag),
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

fn render_html(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
) -> Result<(), String> {
    render_html_with_shortcodes(template, filename, tera, context, output_dir, None)
}

fn render_html_with_shortcodes(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
    shortcode_processor: Option<&ShortcodeProcessor>,
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
        rendered = processor.process_shortcodes(&rendered, context, tera);
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
    if input_folder.read_dir().unwrap().next().is_some() {
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
        read more on [marmite documentation](https://rochacbruno.github.io/marmite)\n\n\
        ",
    ) {
        error!("Failed to create 'content/{now}-welcome.md' file: {e:?}");
        process::exit(1);
    }
    info!("Site initialized in {}", input_folder.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentBuilder;
    use chrono::NaiveDate;
    use std::sync::Mutex;

    #[test]
    fn test_next_and_previous_links_are_stream_specific() {
        let mut site_data = Data::new("", Path::new("marmite.yaml"));
        let post1 = ContentBuilder::new()
            .title("Post 1".to_string())
            .slug("post-1".to_string())
            .stream("blog".to_string())
            .date(
                NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
            .build();
        let post2 = ContentBuilder::new()
            .title("Post 2".to_string())
            .slug("post-2".to_string())
            .stream("blog".to_string())
            .date(
                NaiveDate::from_ymd_opt(2024, 1, 2)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
            .build();
        let post3 = ContentBuilder::new()
            .title("Post 3".to_string())
            .slug("post-3".to_string())
            .stream("news".to_string())
            .date(
                NaiveDate::from_ymd_opt(2024, 1, 3)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
            .build();

        site_data.posts.push(post1.clone());
        site_data.posts.push(post2.clone());
        site_data.posts.push(post3.clone());
        site_data.sort_all();

        let mutex = Mutex::new(site_data);
        let mut site_data_guard = mutex.lock().unwrap();

        set_next_and_previous_links(&mut site_data_guard);

        let blog_post1 = site_data_guard
            .posts
            .iter()
            .find(|p| p.slug == "post-1")
            .unwrap();
        assert!(blog_post1.next.is_some());
        assert_eq!(blog_post1.next.as_ref().unwrap().slug, "post-2");
        assert!(blog_post1.previous.is_none());

        let blog_post2 = site_data_guard
            .posts
            .iter()
            .find(|p| p.slug == "post-2")
            .unwrap();
        assert!(blog_post2.previous.is_some());
        assert_eq!(blog_post2.previous.as_ref().unwrap().slug, "post-1");
        assert!(blog_post2.next.is_none());

        let news_post = site_data_guard
            .posts
            .iter()
            .find(|p| p.slug == "post-3")
            .unwrap();
        assert!(news_post.next.is_none());
        assert!(news_post.previous.is_none());
    }

    #[test]
    fn test_file_mapping() {
        use crate::config::FileMapping;
        use tempfile::TempDir;

        // Create temp directories
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create test files in input directory
        let test_file = input_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let ai_dir = input_dir.path().join("ai");
        fs::create_dir(&ai_dir).unwrap();
        let llms_file = ai_dir.join("llms.txt");
        fs::write(&llms_file, "llms content").unwrap();

        // Create file mappings
        let mappings = vec![
            FileMapping {
                source: "test.txt".to_string(),
                dest: "output_test.txt".to_string(),
            },
            FileMapping {
                source: "ai/llms.txt".to_string(),
                dest: "llms.txt".to_string(),
            },
        ];

        // Execute file mappings
        handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

        // Check that files were copied correctly
        let output_test = output_dir.path().join("output_test.txt");
        assert!(output_test.exists());
        assert_eq!(fs::read_to_string(output_test).unwrap(), "test content");

        let output_llms = output_dir.path().join("llms.txt");
        assert!(output_llms.exists());
        assert_eq!(fs::read_to_string(output_llms).unwrap(), "llms content");
    }

    #[test]
    fn test_file_mapping_glob() {
        use crate::config::FileMapping;
        use tempfile::TempDir;

        // Create temp directories
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create test files
        let assets_dir = input_dir.path().join("assets");
        let imgs_dir = assets_dir.join("imgs");
        fs::create_dir_all(&imgs_dir).unwrap();

        fs::write(imgs_dir.join("image1.jpg"), "image1").unwrap();
        fs::write(imgs_dir.join("image2.jpg"), "image2").unwrap();
        fs::write(imgs_dir.join("doc.pdf"), "pdf").unwrap();

        // Create glob mapping
        let mappings = vec![FileMapping {
            source: "assets/imgs/*.jpg".to_string(),
            dest: "media/photos".to_string(),
        }];

        // Execute file mappings
        handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

        // Check that only jpg files were copied
        let photos_dir = output_dir.path().join("media/photos");
        assert!(photos_dir.exists());
        assert!(photos_dir.join("image1.jpg").exists());
        assert!(photos_dir.join("image2.jpg").exists());
        assert!(!photos_dir.join("doc.pdf").exists());
    }

    #[test]
    fn test_file_mapping_nested_destination() {
        use crate::config::FileMapping;
        use tempfile::TempDir;

        // Create temp directories
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create test file
        let test_file = input_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Create mapping with nested destination
        let mappings = vec![FileMapping {
            source: "test.txt".to_string(),
            dest: "foo/bar/baz/test.txt".to_string(),
        }];

        // Execute file mappings
        handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

        // Check that nested directories were created and file was copied
        let nested_file = output_dir.path().join("foo/bar/baz/test.txt");
        assert!(nested_file.exists());
        assert_eq!(fs::read_to_string(nested_file).unwrap(), "test content");
    }

    #[test]
    fn test_file_mapping_glob_nested_destination() {
        use crate::config::FileMapping;
        use tempfile::TempDir;

        // Create temp directories
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create test files
        let docs_dir = input_dir.path().join("docs");
        fs::create_dir(&docs_dir).unwrap();
        fs::write(docs_dir.join("manual.pdf"), "manual").unwrap();
        fs::write(docs_dir.join("guide.pdf"), "guide").unwrap();
        fs::write(docs_dir.join("readme.txt"), "readme").unwrap();

        // Create glob mapping with nested destination
        let mappings = vec![FileMapping {
            source: "docs/*.pdf".to_string(),
            dest: "assets/downloads/pdfs".to_string(),
        }];

        // Execute file mappings
        handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

        // Check that nested directories were created and files were copied
        let pdfs_dir = output_dir.path().join("assets/downloads/pdfs");
        assert!(pdfs_dir.exists());
        assert!(pdfs_dir.join("manual.pdf").exists());
        assert!(pdfs_dir.join("guide.pdf").exists());
        assert!(!pdfs_dir.join("readme.txt").exists());
    }

    #[test]
    fn test_file_mapping_directory() {
        use crate::config::FileMapping;
        use tempfile::TempDir;

        // Create temp directories
        let input_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create a directory with files
        let src_dir = input_dir.path().join("source_dir");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();
        fs::write(src_dir.join("file2.txt"), "content2").unwrap();

        // Create directory mapping
        let mappings = vec![FileMapping {
            source: "source_dir".to_string(),
            dest: "dest_dir".to_string(),
        }];

        // Execute file mappings
        handle_file_mappings(input_dir.path(), output_dir.path(), &mappings);

        // Check that directory was copied
        // fs_extra::dir::copy copies the contents AND the directory itself into dest
        let dest_dir = output_dir.path().join("dest_dir");

        // First check if dest_dir was created
        assert!(dest_dir.exists(), "dest_dir should exist");

        // The source directory 'source_dir' should be copied INTO dest_dir
        let copied_dir = dest_dir.join("source_dir");
        assert!(
            copied_dir.exists(),
            "source_dir should be copied into dest_dir"
        );
        assert!(copied_dir.join("file1.txt").exists());
        assert!(copied_dir.join("file2.txt").exists());
    }
}
