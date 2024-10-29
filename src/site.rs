use crate::config::Marmite;
use crate::content::{check_for_duplicate_slugs, group_by_tags, slugify, Content};
use crate::embedded::{generate_static, EMBEDDED_TERA};
use crate::markdown::{get_content, process_file};
use crate::server;
use crate::tera_functions::UrlFor;
use fs_extra::dir::{copy as dircopy, CopyOptions};
use hotwatch::{Event, EventKind, Hotwatch};
use log::{debug, error, info};
use serde::Serialize;
use std::path::Path;
use std::{fs, process, sync::Arc, sync::Mutex};
use tera::{Context, Tera};
use walkdir::WalkDir;

#[derive(Serialize, Clone)]
pub struct Data {
    pub site: Marmite,
    pub posts: Vec<Content>,
    pub pages: Vec<Content>,
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
        }
    }
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
        debug!(
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
        let content_dir = content_dir.clone();
        let output_folder = Arc::clone(output_folder);
        let input_folder = input_folder.to_path_buf();
        let site_data = site_data.clone();

        move || {
            let mut site_data = site_data.lock().unwrap();
            // cleanup before rebuilding, otherwise we get duplicated slug
            site_data.posts = Vec::new();
            site_data.pages = Vec::new();
            collect_content(&content_dir, &mut site_data);

            // Detect slug collision
            detect_slug_collision(&site_data);

            // Feed back_links
            collect_back_links(&mut site_data);

            // Sort posts by date (newest first)
            site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));
            // Sort pages on title
            site_data.pages.sort_by(|a, b| b.title.cmp(&a.title));

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

fn collect_content(content_dir: &std::path::PathBuf, site_data: &mut Data) {
    WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let file_name = e
                .path()
                .file_name()
                .and_then(|ext| ext.to_str())
                .expect("Could not get file name");
            let file_extension = e.path().extension().and_then(|ext| ext.to_str());
            e.path().is_file() && file_extension == Some("md") && !file_name.starts_with('_')
        })
        .for_each(|entry| {
            if let Err(e) = process_file(entry.path(), site_data) {
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
            "Error: Duplicate slug found: '{}' \
            - try setting any of `title`, `slug` as a unique text, \
            or leave both empty so filename will be assumed.",
            duplicate
        );
        process::exit(1);
    }
}

fn initialize_tera(input_folder: &Path, site_data: &Data) -> Tera {
    let templates_path = input_folder.join(site_data.site.templates_path.clone());
    let mut tera = match Tera::new(&format!("{}/**/*.html", templates_path.display())) {
        Ok(t) => t,
        Err(e) => {
            error!("Error loading templates: {}", e);
            process::exit(1);
        }
    };
    tera.autoescape_on(vec![]);
    // the person writing a static site knows what is doing!
    tera.register_function(
        "url_for",
        UrlFor {
            base_url: site_data.site.url.to_string(),
        },
    );
    tera.extend(&EMBEDDED_TERA).unwrap();
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
    global_context.insert("site_data", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);

    let hero_fragment = get_html_fragment("_hero.md", content_dir);
    if !hero_fragment.is_empty() {
        global_context.insert("hero", &hero_fragment);
        debug!("Hero fragment {}", &hero_fragment);
    }
    debug!("Global Context site: {:?}", &site_data.site);

    handle_list_page(
        &global_context,
        &site_data.site.list_title,
        &site_data.posts,
        site_data,
        tera,
        output_dir,
        "index",
    )?;

    handle_list_page(
        &global_context,
        &site_data.site.pages_title,
        &site_data.pages,
        site_data,
        tera,
        output_dir,
        "pages",
    )?;

    // Render individual content-slug.html from content.html template
    handle_content_pages(site_data, &global_context, tera, output_dir)?;

    // Check and guarantees that page 404 was generated even if 404.md is removed
    handle_404(content_dir, &global_context, tera, output_dir)?;

    // Render tagged_contents
    handle_tag_pages(output_dir, site_data, &global_context, tera)?;

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
    let total_pages = (total_content + per_page - 1) / per_page;
    for page_num in 0..total_pages {
        let mut context = global_context.clone();

        // Slice the content list for this page
        let page_content =
            &all_content[page_num * per_page..(page_num * per_page + per_page).min(total_content)];

        // Set up context for pagination
        context.insert("title", title);
        context.insert("content_list", page_content);
        context.insert("total_pages", &total_pages);
        context.insert("per_page", &per_page);
        context.insert("total_content", &total_content);

        // Determine filename and pagination values
        let (current_page_number, filename) = if page_num == 0 {
            (1, format!("{output_filename}.html"))
        } else {
            (
                page_num + 1,
                format!("{}-{}.html", output_filename, page_num + 1),
            )
        };
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

        // Debug print
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
        render_html("list.html", &filename, tera, &context, output_dir)?;
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
        render_html(
            "content.html",
            &format!("{}.html", &content.slug),
            tera,
            &content_context,
            output_dir,
        )?;
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
    let mut content = Content {
        html: String::from("Page not found :/"),
        title: String::from("Page not found"),
        date: None,
        slug: "404".to_string(),
        extra: None,
        tags: vec![],
        links_to: None,
        back_links: vec![],
    };
    if input_404_path.exists() {
        let custom_content = get_content(&input_404_path)?;
        content.html.clone_from(&custom_content.html);
        content.title.clone_from(&custom_content.title);
    }
    context.insert("title", &content.title);
    context.insert("content", &content);
    render_html("content.html", "404.html", tera, &context, output_dir)?;
    Ok(())
}

fn get_html_fragment(filename: &str, content_dir: &Path) -> String {
    let filepath = content_dir.join(filename);
    let mut fragment = String::new();
    if filepath.exists() {
        match get_content(&filepath) {
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
    let mut unique_tags: Vec<(String, usize)> = Vec::new();
    for (tag, tagged_contents) in group_by_tags(site_data.posts.clone()) {
        // aggregate unique tags to render the tags list later
        unique_tags.push((tag.clone(), tagged_contents.len()));
        let tag_slug = slugify(&tag);
        handle_list_page(
            global_context,
            &site_data.site.tags_content_title.replace("$tag", &tag),
            &tagged_contents,
            site_data,
            tera,
            output_dir,
            format!("tag-{}", &tag_slug).as_ref(),
        )?;
    }

    // Render tags.html group page
    let mut tag_list_context = global_context.clone();
    tag_list_context.insert("title", &site_data.site.tags_title);
    unique_tags.sort_by(|a, b| b.1.cmp(&a.1));
    tag_list_context.insert("group_content", &unique_tags);
    tag_list_context.insert("current_page", "tags.html");
    render_html(
        "group.html",
        "tags.html",
        tera,
        &tag_list_context,
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
    let rendered = tera.render(template, context).map_err(|e| {
        debug!(
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
