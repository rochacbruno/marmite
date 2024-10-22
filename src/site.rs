use crate::config::Marmite;
use crate::content::{check_for_duplicate_slugs, group_by_tags, slugify, Content};
use crate::embedded::{generate_static, EMBEDDED_TERA};
use crate::markdown::process_file;
use crate::robots;
use crate::tera_functions::UrlFor;
use fs_extra::dir::{copy, CopyOptions};
use log::{debug, error, info};
use serde::Serialize;
use std::path::Path;
use std::{fs, process, sync::Arc};
use tera::{Context, Tera};
use walkdir::WalkDir;
use hotwatch::{Hotwatch, EventKind , Event};

#[derive(Serialize)]
#[derive(Clone)]
pub struct Data {
    pub site: Marmite,
    pub posts: Vec<Content>,
    pub pages: Vec<Content>,
}

impl Data {
    pub fn new(config_content: String) -> Self {
        let site: Marmite = match serde_yaml::from_str::<Marmite>(&config_content) {
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

fn render_templates(site_data: &Data, tera: &Tera, output_dir: &Path) -> Result<(), String> {
    // Build the context of variables that are global on every template
    let mut global_context = Context::new();
    global_context.insert("site_data", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);
    debug!("Global Context: {:?}", &site_data.site);

    // Render index.html from list.html template
    let mut list_context = global_context.clone();
    list_context.insert("title", &site_data.site.list_title);
    list_context.insert("content_list", &site_data.posts);
    list_context.insert("current_page", "index.html");
    debug!(
        "Index Context: {:?}",
        &site_data
            .posts
            .iter()
            .map(|p| format!("{},{}", p.title, p.slug))
            .collect::<Vec<_>>()
    );
    render_html("list.html", "index.html", tera, &list_context, output_dir)?;

    // Render pages.html from list.html template
    let mut list_context = global_context.clone();
    list_context.insert("title", &site_data.site.pages_title);
    list_context.insert("content_list", &site_data.pages);
    list_context.insert("current_page", "pages.html");
    debug!(
        "Pages Context: {:?}",
        &site_data
            .pages
            .iter()
            .map(|p| format!("{},{}", p.title, p.slug))
            .collect::<Vec<_>>()
    );
    render_html("list.html", "pages.html", tera, &list_context, output_dir)?;

    // Render individual content-slug.html from content.html template
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

    // Render tagged_contents
    let mut unique_tags: Vec<(String, usize)> = Vec::new();
    let tags_dir = output_dir.join("tag");
    if let Err(e) = fs::create_dir_all(&tags_dir) {
        error!("Unable to create tag directory: {}", e);
        process::exit(1);
    }
    for (tag, tagged_contents) in group_by_tags(site_data.posts.clone()) {
        // aggregate unique tags to render the tags list later
        unique_tags.push((tag.clone(), tagged_contents.len()));

        let mut tag_context = global_context.clone();
        tag_context.insert(
            "title",
            &site_data.site.tags_content_title.replace("$tag", &tag),
        );
        tag_context.insert("content_list", &tagged_contents);
        let tag_slug = slugify(&tag);
        tag_context.insert("current_page", &format!("tag/{}.html", &tag_slug));
        debug!(
            "Tag {} Context: {:?}",
            &tag,
            &site_data
                .pages
                .iter()
                .map(|p| format!("{},{}", p.title, p.slug))
                .collect::<Vec<_>>()
        );
        render_html(
            "list.html",
            &format!("{}.html", &tag_slug),
            tera,
            &tag_context,
            &tags_dir,
        )?;
    }
    // Render Main tags.html list page from group.html template
    let mut tag_list_context = global_context.clone();
    tag_list_context.insert("title", &site_data.site.tags_title);
    unique_tags.sort_by(|a, b| a.0.cmp(&b.0));
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
        error!("Error rendering template `{}`: {}", template, e);
        e.to_string()
    })?;
    let output_file = output_dir.join(filename);
    fs::write(&output_file, rendered).map_err(|e| e.to_string())?;
    info!("Generated {}", &output_file.display());
    Ok(())
}


pub fn generate(
    config_path: &std::path::PathBuf,
    input_folder: &std::path::Path,
    output_folder: &Arc<std::path::PathBuf>,
    watch: bool, // New parameter for watching
) {
    let config_str = fs::read_to_string(config_path).unwrap_or_else(|e| {
        debug!(
            "Unable to read '{}', assuming defaults.: {}",
            &config_path.display(),
            e
        );
        String::new()
    });
    let site_data = Data::new(config_str);
    let site_data_clone = site_data.clone();

    // Define the content directory
    let content_dir = Some(input_folder.join(site_data_clone.site.content_path))
        .filter(|path| path.is_dir()) // Take if exists
        .unwrap_or_else(|| input_folder.to_path_buf());
    // Fallback to input_folder if not

    // Function to trigger site regeneration
    let mut rebuild_site = {
        let content_dir = content_dir.clone();
        let output_folder = Arc::clone(output_folder);
        let input_folder = input_folder.to_path_buf();
        let mut site_data = site_data.clone();

        move || {
            collect_content(&content_dir, &mut site_data);

            // Detect slug collision
            detect_slug_collision(&site_data);

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
            if let Err(e) = render_templates(&site_data, &tera, &output_path) {
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
            .watch(input_folder, move |event: Event| {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        info!("Change detected. Rebuilding site...");
                        rebuild_site();
                    }
                    _ => {}
                }
            })
            .expect("Failed to watch the input folder!");

        info!("Watching for changes in folder: {}", input_folder.display());

        // Keep the thread alive for watching
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

fn handle_static_artifacts(
    input_folder: &Path,
    site_data: &Data,
    output_folder: &Arc<std::path::PathBuf>,
    content_dir: &std::path::Path,
) {
    robots::handle(content_dir, output_folder);

    let static_source = input_folder.join(site_data.site.static_path.clone());
    if static_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = copy(&static_source, &**output_folder, &options) {
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

        if let Err(e) = copy(&media_source, &**output_folder, &options) {
            error!("Failed to copy media directory: {}", e);
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &media_source.display(),
            &output_folder.display()
        );
    }

    // Copy or create favicon.ico
    let favicon_dst = output_folder.join("favicon.ico");

    // Possible paths where favicon.ico might exist
    let favicon_src_paths = [
        input_folder.join("static").join("favicon.ico"), // User's favicon.ico
                                                         // on #20 we may have embedded statics
    ];

    for favicon_src in &favicon_src_paths {
        if favicon_src.exists() {
            match fs::copy(favicon_src, &favicon_dst) {
                Ok(_) => {
                    info!(
                        "Copied favicon.ico from '{}' to output folder",
                        favicon_src.display()
                    );
                    break;
                }
                Err(e) => error!(
                    "Failed to copy favicon.ico from '{}': {}",
                    favicon_src.display(),
                    e
                ),
            }
        }
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

fn collect_content(content_dir: &std::path::PathBuf, site_data: &mut Data) {
    WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|ext| ext.to_str()) == Some("md")
        })
        .for_each(|entry| {
            if let Err(e) = process_file(entry.path(), site_data) {
                error!("Failed to process file {}: {}", entry.path().display(), e);
            }
        });
}
