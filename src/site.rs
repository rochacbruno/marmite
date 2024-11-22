use crate::config::{Author, Marmite};
use crate::content::{
    check_for_duplicate_slugs, slugify, Content, ContentBuilder, GroupedContent, Kind,
};
use crate::embedded::{generate_static, Templates, EMBEDDED_TERA};
use crate::markdown::{get_content, process_file};
use crate::tera_functions::{Group, UrlFor};
use crate::{server, tera_filter};
use fs_extra::dir::{copy as dircopy, CopyOptions};
use hotwatch::{Event, EventKind, Hotwatch};
use log::{debug, error, info};
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

    pub fn sort_all(&mut self) {
        self.posts.sort_by(|a, b| b.date.cmp(&a.date));
        self.pages.sort_by(|a, b| b.title.cmp(&a.title));
    }

    pub fn clear_all(&mut self) {
        self.posts.clear();
        self.pages.clear();
        self.tag.map.clear();
        self.archive.map.clear();
        self.author.map.clear();
        self.stream.map.clear();
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
    config_path: &std::path::PathBuf,
    input_folder: &std::path::Path,
    output_folder: &Arc<std::path::PathBuf>,
    watch: bool, // New parameter for watching,
    serve: bool, // Is running on server mode
    bind_address: &str,
) {
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
    let site_data = Arc::new(Mutex::new(Data::new(&config_str)));

    // Define the content directory
    let content_dir = {
        let site_data = site_data.lock().unwrap();
        Some(input_folder.join(site_data.site.content_path.clone()))
    }
    .filter(|path| path.is_dir()) // Take if exists
    .unwrap_or_else(|| input_folder.to_path_buf());
    // Fallback to input_folder if not

    // Function to trigger site regeneration
    let rebuild_site = {
        let start_time = std::time::Instant::now();

        let content_dir = content_dir.clone();
        let output_folder = Arc::clone(output_folder);
        let input_folder = input_folder.to_path_buf();
        let site_data = site_data.clone();

        move || {
            let mut site_data = site_data.lock().unwrap();
            // cleanup before rebuilding, otherwise we get duplicated content
            site_data.clear_all();
            let fragments = collect_fragments(&content_dir);
            collect_content(&content_dir, &mut site_data, &fragments);

            // Detect slug collision
            detect_slug_collision(&site_data);

            // Feed back_links
            collect_back_links(&mut site_data);

            site_data.sort_all();

            // Create the output directory
            let site_path = site_data.site.site_path.clone();
            let output_path = output_folder.join(site_path);
            if let Err(e) = fs::create_dir_all(&output_path) {
                error!("Unable to create output directory: {}", e);
                process::exit(1);
            }

            // Initialize Tera templates
            let tera = initialize_tera(&input_folder, &site_data);

            // Render templates
            if let Err(e) = render_templates(&content_dir, &site_data, &tera, &output_path) {
                error!("Failed to render templates: {}", e);
                process::exit(1);
            }

            // Copy static folder if present
            handle_static_artifacts(&input_folder, &site_data, &output_folder, &content_dir);

            if site_data.site.enable_search {
                generate_search_index(&site_data, &output_folder);
            }

            let end_time = start_time.elapsed().as_secs_f64();
            write_build_info(&output_path, &site_data, &end_time);
            debug!("Site generated in {:.2}s", end_time);
            info!("Site generated at: {}/", output_folder.display());
        }
    };

    // Initial site generation
    rebuild_site();

    // If watch flag is enabled, start hotwatch
    if watch {
        let mut hotwatch = Hotwatch::new().expect("Failed to initialize hotwatch!");

        // Watch the input folder for changes
        hotwatch
            .watch(input_folder, move |event: Event| match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    info!("Change detected. Rebuilding site...");
                    rebuild_site();
                }
                _ => {}
            })
            .expect("Failed to watch the input folder!");

        info!("Watching for changes in folder: {}", input_folder.display());

        // Keep the thread alive for watching
        if serve {
            info!("Starting built-in HTTP server...");
            server::start(bind_address, output_folder);
        } else {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

fn collect_fragments(content_dir: &Path) -> HashMap<String, String> {
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
    WalkDir::new(content_dir)
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
                        .unwrap_or_else(|| panic!("Could not get file name {:?}", e.path())),
                );
            let file_extension = e.path().extension().and_then(|ext| ext.to_str());
            e.path().is_file() && file_extension == Some("md") && !file_name.starts_with('_')
        })
        .for_each(|entry| {
            if let Err(e) = process_file(entry.path(), site_data, fragments) {
                error!("Failed to process file {}: {}", entry.path().display(), e);
            }
        });
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
) -> Result<(), String> {
    // Build the context of variables that are global on every template
    let mut global_context = Context::new();
    let mut site_data = site_data.clone();

    let hero_fragment = get_html_fragment("_hero.md", content_dir);
    if !hero_fragment.is_empty() {
        global_context.insert("hero", &hero_fragment);
        debug!("Hero fragment {}", &hero_fragment);
    }
    let footer_fragment = get_html_fragment("_footer.md", content_dir);
    if !footer_fragment.is_empty() {
        site_data.site.footer.clone_from(&footer_fragment);
        debug!("Footer fragment {}", &footer_fragment);
    }

    global_context.insert("site_data", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);
    global_context.insert("language", &site_data.site.language);
    debug!("Global Context site: {:?}", &site_data.site);

    // Assuming every item on site_data.posts is a Content and has a stream field
    // we can use this to render a {stream}.html page from list.html template
    // by default posts will have a `index` stream.
    for (stream, stream_contents) in site_data.stream.iter() {
        let stream_slug = slugify(stream);
        let title = if stream == "index" {
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
        crate::feed::generate_rss(&stream_contents, output_dir, &stream_slug, &site_data.site)?;
    }

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
    for (username, _) in site_data.author.iter() {
        let mut author_context = global_context.clone();

        let author = if let Some(author) = site_data.site.authors.get(username) {
            author
        } else {
            &Author {
                name: username.to_string(),
                bio: None,
                avatar: Some("static/avatar-placeholder.png".to_string()),
                links: None,
            }
        };

        author_context.insert("author", &author);

        let author_slug = slugify(username);
        let author_posts = site_data
            .posts
            .iter()
            .filter(|post| post.authors.contains(username))
            .cloned()
            .collect::<Vec<Content>>();

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
    }

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
    end_time: &f64,
) {
    let build_info = BuildInfo {
        marmite_version: env!("CARGO_PKG_VERSION").to_string(),
        posts: site_data.posts.len(),
        pages: site_data.pages.len(),
        generated_at: chrono::Local::now().to_string(),
        elapsed_time: *end_time,
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
    context.insert("current_page", &format!("{}.html", output_filename));

    // If all_content is empty, ensure we still generate an empty page
    if total_content == 0 {
        let empty_content_list: Vec<Content> = Vec::new();
        context.insert("content_list", &empty_content_list);
        context.insert("total_pages", &1);
        context.insert("total_content", &1);
        context.insert("current_page_number", &1);
        render_html(
            "custom_list.html,list.html",
            &format!("{}.html", output_filename),
            tera,
            &context,
            output_dir,
        )?;
        return Ok(());
    }

    let total_pages = (total_content + per_page - 1) / per_page;
    context.insert("total_content", &total_content);
    context.insert("total_pages", &total_pages);
    for page_num in 0..total_pages {
        // Slice the content list for this page
        let page_content =
            &all_content[page_num * per_page..(page_num * per_page + per_page).min(total_content)];

        // Set up context for pagination
        context.insert("content_list", page_content);

        // Determine filename and pagination values
        let (current_page_number, filename) = if page_num == 0 {
            (1, format!("{output_filename}.html"))
        } else {
            (
                page_num + 1,
                format!("{}-{}.html", output_filename, page_num + 1),
            )
        };

        if current_page_number > 1 {
            if title.is_empty() {
                context.insert("title", &format!("Page - {current_page_number}"));
            } else {
                context.insert("title", &format!("{title} - {current_page_number}"));
            }
        } else {
            context.insert("title", title);
        }

        context.insert("current_page", &filename);
        context.insert("current_page_number", &current_page_number);

        if page_num > 0 {
            let prev_page = if page_num == 1 {
                format!("{output_filename}.html")
            } else {
                format!("{output_filename}-{page_num}.html")
            };
            context.insert("previous_page", &prev_page);
        }

        if page_num < total_pages - 1 {
            let next_page = format!("{}-{}.html", output_filename, page_num + 2);
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
        let templates = format!("custom_{filename},list.html");
        render_html(&templates, &filename, tera, &context, output_dir)?;
    }
    Ok(())
}

fn handle_content_pages(
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
    for content in site_data.posts.iter().chain(&site_data.pages) {
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

        if let Err(e) = render_html(
            "content.html",
            &format!("{}.html", &content.slug),
            tera,
            &content_context,
            output_dir,
        ) {
            error!("Failed to render content {}: {}", &content.slug, e);
            return Err(e);
        }
    }
    Ok(())
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
        let custom_content = get_content(&input_404_path, None, &Marmite::default())?;
        content.html.clone_from(&custom_content.html);
        content.title.clone_from(&custom_content.title);
    }
    context.insert("title", &content.title);
    context.insert("content", &content);
    context.insert("current_page", "404.html");
    render_html("content.html", "404.html", tera, &context, output_dir)?;
    Ok(())
}

pub fn get_html_fragment(filename: &str, content_dir: &Path) -> String {
    let filepath = content_dir.join(filename);
    let mut fragment = String::new();
    if filepath.exists() {
        match get_content(&filepath, None, &Marmite::default()) {
            Ok(content) => fragment.push_str(&content.html),
            Err(e) => {
                error!("Error parsing {}: {}", filepath.display(), e);
            }
        }
    }
    fragment
}

fn handle_tag_pages(
    output_dir: &Path,
    site_data: &Data,
    global_context: &Context,
    tera: &Tera,
) -> Result<(), String> {
    for (tag, tagged_contents) in site_data.tag.iter() {
        let tag_slug = slugify(tag);
        let filename = format!("tag-{}", &tag_slug);
        handle_list_page(
            global_context,
            &site_data.site.tags_content_title.replace("$tag", tag),
            &tagged_contents,
            site_data,
            tera,
            output_dir,
            &filename,
        )?;
        // Render tag-{tag}.rss for each stream
        crate::feed::generate_rss(
            &tagged_contents,
            output_dir,
            &filename.clone(),
            &site_data.site,
        )?;
    }

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
    for (year, archive_contents) in site_data.archive.iter() {
        let filename = format!("archive-{year}");
        handle_list_page(
            global_context,
            &site_data.site.archives_content_title.replace("$year", year),
            &archive_contents,
            site_data,
            tera,
            output_dir,
            &filename,
        )?;
        // Render archive-{year}.rss for each stream
        crate::feed::generate_rss(
            &archive_contents,
            output_dir,
            &filename.clone(),
            &site_data.site,
        )?;
    }

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
