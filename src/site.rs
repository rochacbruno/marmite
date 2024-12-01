use crate::config::{Author, Marmite};
use crate::content::{
    check_for_duplicate_slugs, slugify, Content, ContentBuilder, GroupedContent, Kind,
};
use crate::embedded::{generate_static, Templates, EMBEDDED_TERA};
use crate::tera_functions::{Group, UrlFor};
use crate::{server, tera_filter};
use chrono::Datelike;
use core::str;
use fs_extra::dir::{copy as dircopy, CopyOptions};
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
}

impl Data {
    pub fn new(config_content: &str) -> Self {
        let site: Marmite = match serde_yaml::from_str::<Marmite>(config_content) {
            Ok(site) => site,
            Err(e) => {
                error!("Failed to parse config YAML: {}", e);
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
        }
    }

    pub fn from_file(config_path: &Path) -> Self {
        let config_str = fs::read_to_string(config_path).unwrap_or_else(|e| {
            info!(
                "Unable to read '{}', assuming defaults.: {}",
                &config_path.display(),
                e
            );
            String::new()
        });

        if config_str.is_empty() {
            info!("Config loaded from: defaults");
        } else {
            info!("Config loaded from: {}", config_path.display());
        }

        Self::new(&config_str)
    }

    pub fn sort_all(&mut self) {
        self.posts.sort_by(|a, b| b.date.cmp(&a.date));
        self.pages.sort_by(|a, b| b.title.cmp(&a.title));
        self.tag.sort_all();
        self.archive.sort_all();
        self.author.sort_all();
        self.stream.sort_all();
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
            };
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
    elapsed_time: f64,
}

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
            site_data.site.override_from_cli_args(&moved_cli_args);

            let fragments = collect_content_fragments(&content_folder);
            collect_content(&content_folder, &mut site_data, &fragments);
            site_data.sort_all();
            detect_slug_collision(&site_data); // Detect slug collision and warn user
            collect_back_links(&mut site_data);

            let site_path = site_data.site.site_path.clone();
            let output_path = moved_output_folder.join(site_path);
            if let Err(e) = fs::create_dir_all(&output_path) {
                error!("Unable to create output directory: {}", e);
                process::exit(1);
            }

            let tera = initialize_tera(&moved_input_folder, &site_data);
            if let Err(e) =
                render_templates(&content_folder, &site_data, &tera, &output_path, &fragments)
            {
                error!("Failed to render templates: {}", e);
                process::exit(1);
            }

            handle_static_artifacts(
                &moved_input_folder,
                &site_data,
                &moved_output_folder,
                &content_folder,
            );

            if site_data.site.enable_search {
                generate_search_index(&site_data, &moved_output_folder);
            }

            let end_time = start_time.elapsed().as_secs_f64();
            write_build_info(&output_path, &site_data, end_time);
            debug!("Site generated in {:.2}s", end_time);
            info!("Site generated at: {}/", moved_output_folder.display());
        }
    };

    // Initial site generation
    rebuild();

    if watch {
        let mut hotwatch = Hotwatch::new().expect("Failed to initialize hotwatch!");
        let watch_folder = Arc::clone(input_folder).as_path().to_path_buf();
        // Watch the input folder for changes
        hotwatch
            .watch(watch_folder, move |event: Event| match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    info!("Change detected. Rebuilding site...");
                    rebuild();
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
fn collect_global_fragments(content_dir: &Path, global_context: &mut Context, tera: &Tera) {
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
        let fragment_content = crate::parser::get_html(&rendered_fragment);
        // global_context.insert((*fragment).to_string(), &fragment_content);
        debug!("{} fragment {}", fragment, &fragment_content);
        ((*fragment).to_string(), fragment_content)
    })
    .collect::<Vec<_>>();
    for (name, content) in fragments {
        global_context.insert(name, &content);
    }
}

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
                if links_to.contains(&slug) {
                    contents[i].back_links.push(other_content.clone());
                }
            }
        }
    }
}

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
                        .unwrap_or_else(|| panic!("Could not get file name {:?}", e.path())),
                );
            let file_extension = e.path().extension().and_then(|ext| ext.to_str());
            e.path().is_file() && file_extension == Some("md") && !file_name.starts_with('_')
        })
        .map(|entry| Content::from_markdown(entry.path(), Some(fragments), &site_data.site))
        .collect::<Vec<_>>();
    for content in contents {
        match content {
            Ok(content) => {
                site_data.push_content(content);
            }
            Err(e) => {
                error!("Failed to process content: {}", e);
            }
        };
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
            "Duplicate slug found: '{}' \
            - try setting `title` or `slug` as a unique text, \
            or leave both empty so filename will be assumed. \
            - The latest content rendered will overwrite the previous one.",
            duplicate
        );
    }
}

fn initialize_tera(input_folder: &Path, site_data: &Data) -> Tera {
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
    tera.register_filter(
        "default_date_format",
        tera_filter::DefaultDateFormat {
            date_format: site_data.site.default_date_format.to_string(),
        },
    );

    let templates_path = input_folder.join(site_data.site.templates_path.clone());
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
                    error!("Failed to load template: {}", template_name);
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
    debug!("{:#?}", &tera);
    tera
}

fn render_templates(
    content_dir: &Path,
    site_data: &Data,
    tera: &Tera,
    output_dir: &Path,
    fragments: &HashMap<String, String>,
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
    collect_global_fragments(content_dir, &mut global_context, tera);

    // Assuming every item on site_data.posts is a Content and has a stream field
    // we can use this to render a {stream}.html page from list.html template
    // by default posts will have a `index` stream.
    // for (stream, stream_contents) in
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
                &global_context,
                &title,
                &sorted_stream_contents,
                &site_data,
                tera,
                output_dir,
                &stream_slug,
            )?;
            // Render {stream}.rss for each stream
            crate::feed::generate_rss(stream_contents, output_dir, &stream_slug, &site_data.site)?;

            if site_data.site.json_feed {
                crate::feed::generate_json(
                    stream_contents,
                    output_dir,
                    &stream_slug,
                    &site_data.site,
                )?;
            };
            Ok(())
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))?;

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

    // Check and guarantees that page 404 was generated even if 404.md is removed
    handle_404(content_dir, &global_context, tera, output_dir)?;

    // render group pages
    handle_tag_pages(output_dir, &site_data, &global_context, tera)?;
    handle_archive_pages(output_dir, &site_data, &global_context, tera)?;
    handle_author_pages(output_dir, &site_data, &global_context, tera)?;
    handle_stream_list_page(output_dir, &site_data, &global_context, tera)?;

    // If site_data.stream.map does not contain the index stream
    // we will render empty index.html from list.html template
    if !site_data.stream.map.contains_key("index") {
        handle_default_empty_site(&global_context, tera, output_dir)?;
    }

    // Render individual content-slug.html from content.html template
    // content is rendered as last step so it gives the user the ability to
    // override some prebuilt pages like tags.html, authors.html, etc.
    handle_content_pages(&site_data, &global_context, tera, output_dir)?;

    Ok(())
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
            let mut author_context = global_context.clone();
            let author = if let Some(author) = site_data.site.authors.get(*username) {
                author
            } else {
                &Author {
                    name: (*username).to_string(),
                    bio: None,
                    avatar: Some("static/avatar-placeholder.png".to_string()),
                    links: None,
                }
            };
            author_context.insert("author", &author);

            let author_slug = slugify(username);
            let mut author_posts = site_data
                .posts
                .iter()
                .filter(|post| post.authors.contains(username))
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

fn handle_static_artifacts(
    input_folder: &Path,
    site_data: &Data,
    output_folder: &Arc<std::path::PathBuf>,
    content_dir: &std::path::Path,
) {
    let static_source = input_folder.join(site_data.site.static_path.clone());
    if static_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = dircopy(&static_source, &**output_folder, &options) {
            error!("Failed to copy static directory: {}", e);
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

    // Copy content/media folder if present
    let media_source = content_dir.join(site_data.site.media_path.clone());
    if media_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = dircopy(&media_source, &**output_folder, &options) {
            error!("Failed to copy media directory: {}", e);
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &media_source.display(),
            &output_folder.display()
        );
    }

    // we want to check if the file exists in `input_folder` and `content_dir`
    // if not then we want to check if exists in `output_folder/static` (came from embedded)
    // the first we find we want to copy to the `output_folder/{destiny_path}`
    let custom_files = [
        // name, destination
        ("custom.css", site_data.site.static_path.clone()),
        ("custom.js", site_data.site.static_path.clone()),
        ("favicon.ico", String::new()),
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
                    Err(e) => error!("Failed to copy {}: {}", source_file.display(), e),
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

    // Merge posts and pages into a single list
    let all_content_json = site_data
        .posts
        .iter()
        .map(convert_items_to_json)
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            site_data
                .pages
                .iter()
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
        error!("Failed to write search_index.json: {}", e);
    } else {
        info!("Generated search_index.json");
    }
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
        elapsed_time: end_time,
    };

    let build_info_path = output_path.join("marmite.json");
    if let Err(e) = fs::write(
        &build_info_path,
        serde_json::to_string_pretty(&build_info).unwrap(),
    ) {
        error!("Failed to write marmite.json: {}", e);
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

    let total_pages = (total_content + per_page - 1) / per_page;
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

fn handle_content_pages(
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    site_data
        .posts
        .iter()
        .chain(&site_data.pages)
        .collect::<Vec<_>>()
        .par_iter()
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

            render_html(
                "content.html",
                &format!("{}.html", &content.slug),
                tera,
                &content_context,
                output_dir,
            )
        })
        .reduce_with(|r1, r2| if r1.is_err() { r1 } else { r2 })
        .unwrap_or(Ok(()))
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
        let custom_content = Content::from_markdown(&input_404_path, None, &Marmite::default())?;
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
            handle_list_page(
                global_context,
                &site_data.site.tags_content_title.replace("$tag", tag),
                tagged_contents,
                site_data,
                tera,
                output_dir,
                &filename,
            )?;
            // Render tag-{tag}.rss for each stream
            crate::feed::generate_rss(
                tagged_contents,
                output_dir,
                &filename.clone(),
                &site_data.site,
            )?;

            if site_data.site.json_feed {
                crate::feed::generate_json(
                    tagged_contents,
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
            handle_list_page(
                global_context,
                &site_data.site.archives_content_title.replace("$year", year),
                archive_contents,
                site_data,
                tera,
                output_dir,
                &filename,
            )?;
            // Render archive-{year}.rss for each stream
            crate::feed::generate_rss(
                archive_contents,
                output_dir,
                &filename.clone(),
                &site_data.site,
            )?;

            if site_data.site.json_feed {
                crate::feed::generate_json(
                    archive_contents,
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
    let templates = template.split(',').collect::<Vec<_>>();
    let template = templates
        .iter()
        .find(|t| tera.get_template(t).is_ok())
        .unwrap_or(&templates[0]);
    let rendered = tera.render(template, context).map_err(|e| {
        error!(
            "Error rendering template `{}` -> {}: {:#?}",
            template, filename, e
        );
        e.to_string()
    })?;
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
        error!("Failed to create input folder: {}", e);
        process::exit(1);
    }
    if input_folder.read_dir().unwrap().next().is_some() {
        error!("Input folder is not empty: {}", input_folder.display());
        process::exit(1);
    }
    crate::config::generate(input_folder, cli_args);
    if let Err(e) = fs::create_dir(&content_folder) {
        error!("Failed to create 'content' folder: {}", e);
        process::exit(1);
    }
    if let Err(e) = fs::create_dir(&media_folder) {
        error!("Failed to create 'content/media' folder: {}", e);
        process::exit(1);
    }
    // create input_folder/custom.css with `/* Custom CSS */` content
    if let Err(e) = fs::write(input_folder.join("custom.css"), "/* Custom CSS */") {
        error!("Failed to create 'custom.css' file: {}", e);
        process::exit(1);
    }
    // create input_folder/custom.js with `// Custom JS` content
    if let Err(e) = fs::write(input_folder.join("custom.js"), "// Custom JS") {
        error!("Failed to create 'custom.js' file: {}", e);
        process::exit(1);
    }
    // create content/_404.md with `# Not Found` content
    if let Err(e) = fs::write(content_folder.join("_404.md"), "# Not Found") {
        error!("Failed to create 'content/_404.md' file: {}", e);
        process::exit(1);
    }
    // create content/_references.md with `[marmite]: https://github.com/rochacbruno/marmite` content
    if let Err(e) = fs::write(
        content_folder.join("_references.md"),
        "[github]: https://github.com/rochacbruno/marmite",
    ) {
        error!("Failed to create 'content/_references.md' file: {}", e);
        process::exit(1);
    }
    // create content/_markdown_header.md with `<!-- Content Injected to every content markdown header -->` content
    if let Err(e) = fs::write(
        content_folder.join("_markdown_header.md"),
        "<!-- Content Injected to every content markdown header -->",
    ) {
        error!("Failed to create 'content/_markdown_header.md' file: {}", e);
        process::exit(1);
    }
    // create content/_markdown_footer.md with `<!-- Content Injected to every content markdown footer -->` content
    if let Err(e) = fs::write(
        content_folder.join("_markdown_footer.md"),
        "<!-- Content Injected to every content markdown footer -->",
    ) {
        error!("Failed to create 'content/_markdown_footer.md' file: {}", e);
        process::exit(1);
    }
    // create content/_announce.md with `Give us a &star; on [github]` content
    if let Err(e) = fs::write(
        content_folder.join("_announce.md"),
        "Give us a &star; on [github]",
    ) {
        error!("Failed to create 'content/_announce.md' file: {}", e);
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
        error!("Failed to create 'content/_sidebar.md' file: {}", e);
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
        error!("Failed to create 'content/_comments.md' file: {}", e);
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
        error!("Failed to create 'content/_hero.md' file: {}", e);
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
        error!("Failed to create 'content/about.md' file: {}", e);
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
        error!("Failed to create 'content/{now}-welcome.md' file: {}", e);
        process::exit(1);
    }
    info!("Site initialized in {}", input_folder.display());
}
