use crate::config::Marmite;
use crate::content::{check_for_duplicate_slugs, group_by_tags, slugify, Content};
use crate::embedded::{generate_static, EMBEDDED_TERA};
use crate::markdown::process_file;
use crate::tera_functions::UrlFor;
use fs_extra::dir::{copy as dircopy, CopyOptions};
use log::{debug, error, info};
use serde::Serialize;
use std::path::Path;
use std::{fs, process, sync::Arc};
use tera::{Context, Tera};
use walkdir::WalkDir;

const NAME_BASED_SLUG_FILES: [&str; 1] = ["404.md"];

#[derive(Serialize)]
pub struct Data<'a> {
    pub site: Marmite<'a>,
    pub posts: Vec<Content>,
    pub pages: Vec<Content>,
}

impl<'a> Data<'a> {
    pub fn new(config_content: &'a str) -> Self {
        let site: Marmite = match serde_yaml::from_str(config_content) {
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
    list_context.insert("title", site_data.site.list_title);
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
    list_context.insert("title", site_data.site.pages_title);
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

    // Check and guarantees that page 404 was generated even if 404.md is removed
    handle_404(&global_context, tera, output_dir)?;

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

fn handle_404(global_context: &Context, tera: &Tera, output_dir: &Path) -> Result<(), String> {
    let file_404_path = output_dir.join("404.html");
    if !file_404_path.exists() {
        let mut context = global_context.clone();
        let page_404_content = Content {
            html: String::from("Page not found :/"),
            title: String::from("Page not found"),
            date: None,
            slug: String::new(),
            extra: None,
            tags: vec![],
        };
        context.insert("title", &page_404_content.title);
        context.insert("content", &page_404_content);
        render_html("content.html", "404.html", tera, &context, output_dir)?;
    };
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
) {
    let config_str = fs::read_to_string(config_path).unwrap_or_else(|e| {
        debug!(
            "Unable to read '{}', assuming defaults.: {}",
            &config_path.display(),
            e
        );
        String::new()
    });
    let mut site_data = Data::new(&config_str);

    // Define the content directory
    let content_dir = Some(input_folder.join(site_data.site.content_path))
        .filter(|path| path.is_dir()) // Take if exists
        .unwrap_or_else(|| input_folder.to_path_buf());
    // Fallback to input_folder if not

    // Walk through the content directory
    collect_content(&content_dir, &mut site_data);

    // Detect slug collision
    detect_slug_collision(&site_data);

    // Sort posts by date (newest first)
    site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));
    // Sort pages on title
    site_data.pages.sort_by(|a, b| b.title.cmp(&a.title));

    // Create the output directory
    let output_path = output_folder.join(site_data.site.site_path);
    if let Err(e) = fs::create_dir_all(&output_path) {
        error!("Unable to create output directory: {}", e);
        process::exit(1);
    }

    // Initialize Tera templates
    let tera = initialize_tera(input_folder, &site_data);

    // Render templates
    if let Err(e) = render_templates(&site_data, &tera, &output_path) {
        error!("Failed to render templates: {}", e);
        process::exit(1);
    }

    // Copy static folder if present
    handle_static_artifacts(input_folder, &site_data, output_folder, &content_dir);

    info!("Site generated at: {}/", output_folder.display());
}

fn handle_static_artifacts(
    input_folder: &Path,
    site_data: &Data,
    output_folder: &Arc<std::path::PathBuf>,
    content_dir: &std::path::Path,
) {
    // Copy static files
    let static_source = input_folder.join(site_data.site.static_path);
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
        // generate from embedded
        generate_static(&output_folder.join(site_data.site.static_path));
    }

    // Copy content/media folder if present
    let media_source = content_dir.join(site_data.site.media_path);
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
        ("custom.css", site_data.site.static_path),
        ("custom.js", site_data.site.static_path),
        ("favicon.ico", ""),
        ("robots.txt", ""),
    ];
    let output_static_destiny = output_folder.join(site_data.site.static_path);
    let possible_sources = [input_folder, content_dir, output_static_destiny.as_path()];
    let mut copied_custom_files = Vec::new();
    for possible_source in &possible_sources {
        for custom_file in custom_files {
            let source_file = possible_source.join(custom_file.0);
            if copied_custom_files.contains(&custom_file.0.to_string()) {
                continue;
            }
            if source_file.exists() {
                let destiny_path = output_folder.join(custom_file.1).join(custom_file.0);
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

fn initialize_tera(input_folder: &Path, site_data: &Data) -> Tera {
    let templates_path = input_folder.join(site_data.site.templates_path);
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
            let file_name = e
                .path()
                .file_name()
                .and_then(|ext| ext.to_str())
                .expect("Could not get file name");
            let file_extension = e.path().extension().and_then(|ext| ext.to_str());
            e.path().is_file()
                && !NAME_BASED_SLUG_FILES.contains(&file_name)
                && file_extension == Some("md")
        })
        .for_each(|entry| {
            if let Err(e) = process_file(entry.path(), site_data, false) {
                error!("Failed to process file {}: {}", entry.path().display(), e);
            }
        });

    for slugged_file in NAME_BASED_SLUG_FILES {
        let slugged_path = content_dir.join(slugged_file);
        if slugged_path.exists() {
            if let Err(e) = process_file(slugged_path.as_path(), site_data, true) {
                error!(
                    "Failed to process file {}: {}",
                    slugged_path.as_path().display(),
                    e
                );
            }
        }
    }
}
