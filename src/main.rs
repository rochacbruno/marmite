use crate::config::Marmite;
use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use env_logger::{Builder, Env};
use frontmatter_gen::{extract, Frontmatter, Value};
use fs_extra::dir::{copy, CopyOptions};
use log::{debug, error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::sync::Arc;
use tera::{Context, Tera};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

mod cli;
mod config;
mod robots;
mod server;
mod tera_functions;

fn main() -> io::Result<()> {
    let args = cli::Cli::parse();
    let input_folder = args.input_folder;
    let output_folder = Arc::new(args.output_folder);
    let serve = args.serve;
    let config_path = input_folder.join(args.config);
    let bind_address: &str = args.bind.as_str();

    let env = Env::default().default_filter_or(if args.debug { "debug" } else { "info" });
    if let Err(e) = Builder::from_env(env).try_init() {
        error!("Logger already initialized: {}", e);
    }

    // Initialize site data
    let marmite = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        debug!(
            "Unable to read '{}', assuming defaults.: {}",
            &config_path.display(),
            e
        );
        String::new()
    });
    let site: Marmite = match serde_yaml::from_str(&marmite) {
        Ok(site) => site,
        Err(e) => {
            error!("Failed to parse '{}' YAML: {}", &config_path.display(), e);
            process::exit(1);
        }
    };
    let mut site_data = SiteData::new(&site);

    // Define the content directory
    let content_dir = Some(input_folder.join(&site_data.site.content_path))
        .filter(|path| path.is_dir()) // Take if exists
        .unwrap_or_else(|| input_folder.clone()); // Fallback to input_folder if not

    // Walk through the content directory
    WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|ext| ext.to_str()) == Some("md")
        })
        .for_each(|entry| {
            if let Err(e) = process_file(entry.path(), &mut site_data) {
                error!("Failed to process file {}: {}", entry.path().display(), e);
            }
        });

    // Detect slug collision
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

    // Sort posts by date (newest first)
    site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));
    // Sort pages on title
    site_data.pages.sort_by(|a, b| b.title.cmp(&a.title));

    // Create the output directory
    let output_path = output_folder.join(&site_data.site.site_path);
    if let Err(e) = fs::create_dir_all(&output_path) {
        error!("Unable to create output directory: {}", e);
        process::exit(1);
    }

    robots::handle_robots(&content_dir, &output_path);

    // Initialize Tera templates
    let templates_path = input_folder.join(&site_data.site.templates_path);
    let mut tera = match Tera::new(&format!("{}/**/*.html", templates_path.display())) {
        Ok(t) => t,
        Err(e) => {
            error!("Error loading templates: {}", e);
            process::exit(1);
        }
    };
    tera.autoescape_on(vec![]); // the person writing a static site knows what is doing!
    tera.register_function(
        "url_for",
        tera_functions::UrlFor {
            base_url: site_data.site.url.to_string(),
        },
    );

    // Render templates
    if let Err(e) = render_templates(&site_data, &tera, &output_path) {
        error!("Failed to render templates: {}", e);
        process::exit(1);
    }

    // Copy static folder if present
    let static_source = input_folder.join(site_data.site.static_path);
    if static_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = copy(&static_source, &*output_folder, &options) {
            error!("Failed to copy static directory: {}", e);
            process::exit(1);
        }

        info!(
            "Copied '{}' to '{}/'",
            &static_source.display(),
            &output_folder.display()
        );
    }

    // Copy content/media folder if present
    let media_source = content_dir.join(site_data.site.media_path);
    if media_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = copy(&media_source, &*output_folder, &options) {
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
            match fs::copy(&favicon_src, &favicon_dst) {
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

    // Serve the site if the flag was provided
    if serve {
        info!("Starting built-in HTTP server...");
        server::start_server(&bind_address, output_folder.clone().into());
    }

    info!("Site generated at: {}/", output_folder.display());

    Ok(())
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct Content {
    title: String,
    slug: String,
    html: String,
    tags: Vec<String>,
    date: Option<NaiveDateTime>,
}

fn group_by_tags(posts: Vec<Content>) -> Vec<(String, Vec<Content>)> {
    // Create a HashMap to store the tags and the corresponding Content items.
    let mut tag_map: HashMap<String, Vec<Content>> = HashMap::new();

    // Iterate over the posts
    for post in posts.into_iter() {
        // For each tag in the current post
        for tag in post.tags.clone() {
            // Insert the tag into the map or push the post into the existing vector
            tag_map
                .entry(tag)
                .or_insert_with(Vec::new)
                .push(post.clone());
        }
    }

    // Convert the HashMap into a Vec<(String, Vec<Content>)>
    tag_map.into_iter().collect()
}

#[derive(Serialize)]
struct SiteData<'a> {
    site: &'a Marmite<'a>,
    posts: Vec<Content>,
    pages: Vec<Content>,
}

impl<'a> SiteData<'a> {
    fn new(site: &'a Marmite) -> Self {
        SiteData {
            site,
            posts: Vec::new(),
            pages: Vec::new(),
        }
    }
}

fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
    if content.starts_with("---") {
        extract(&content).map_err(|e| e.to_string())
    } else {
        Ok((Frontmatter::new(), content))
    }
}

fn process_file(path: &Path, site_data: &mut SiteData) -> Result<(), String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = parse_front_matter(&file_content)?;

    let mut options = ComrakOptions::default();
    options.render.unsafe_ = true; // Allow raw html
    let html = markdown_to_html(markdown, &options);

    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, &path);
    let date = get_date(&frontmatter, &path);

    let content = Content {
        title,
        slug,
        tags,
        html,
        date,
    };

    if date.is_some() {
        site_data.posts.push(content);
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

fn extract_date_from_filename(path: &Path) -> Option<NaiveDateTime> {
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    date_re
        .find(path.to_str().unwrap())
        .and_then(|m| NaiveDate::parse_from_str(m.as_str(), "%Y-%m-%d").ok())
        .and_then(|dt| dt.and_hms_opt(0, 0, 0))
}

fn get_date(frontmatter: &Frontmatter, path: &Path) -> Option<NaiveDateTime> {
    if let Some(input) = frontmatter.get("date") {
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M:%S")
        {
            return Some(date);
        }
        if let Ok(date) = NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M")
        {
            return Some(date);
        }
        if let Ok(date) = NaiveDate::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d") {
            return date.and_hms_opt(0, 0, 0);
        }
        error!(
            "ERROR: Invalid date format {} when parsing {}",
            input.to_string_representation(),
            path.display()
        );
        process::exit(1);
    }

    if let Some(date) = extract_date_from_filename(path) {
        return Some(date);
    }

    None
}

fn get_slug<'a>(frontmatter: &'a Frontmatter, path: &'a Path) -> String {
    if let Some(slug) = frontmatter.get("slug") {
        return slugify(&slug.to_string());
    }
    if let Some(title) = frontmatter.get("title") {
        return slugify(&title.to_string());
    }

    let slug = path.file_stem().and_then(|stem| stem.to_str()).unwrap();
    if let Some(date) = extract_date_from_filename(path) {
        return slug
            .replace(&format!("{}-", date.date().to_string()), "")
            .to_string();
    }

    slug.to_string()
}

fn slugify(text: &str) -> String {
    let normalized = text.nfd().collect::<String>().to_lowercase();
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let slug = re.replace_all(&normalized, "-");
    slug.trim_matches('-').to_string()
}

fn check_for_duplicate_slugs(contents: &Vec<&Content>) -> Result<(), String> {
    let mut seen = HashSet::new();

    for content in contents {
        if !seen.insert(&content.slug) {
            return Err(content.slug.clone());
        }
    }

    Ok(())
}

fn get_title<'a>(frontmatter: &'a Frontmatter, html: &'a str) -> String {
    match frontmatter.get("title") {
        Some(Value::String(t)) => t.to_string(),
        _ => html
            .lines()
            .filter(|line| !line.is_empty())
            .next()
            .unwrap_or("")
            .trim_start_matches("#")
            .trim()
            .to_string(),
    }
}

fn get_tags(frontmatter: &Frontmatter) -> Vec<String> {
    let tags: Vec<String> = match frontmatter.get("tags") {
        Some(Value::Array(tags)) => tags
            .iter()
            .map(Value::to_string)
            .map(|t| t.trim_matches('"').to_string())
            .collect(),
        Some(Value::String(tags)) => tags
            .split(",")
            .map(|t| t.trim())
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    };
    tags
}

fn render_templates(site_data: &SiteData, tera: &Tera, output_dir: &Path) -> Result<(), String> {
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
    generate_html("list.html", "index.html", &tera, &list_context, output_dir)?;

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
    generate_html("list.html", "pages.html", &tera, &list_context, output_dir)?;

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
        generate_html(
            "content.html",
            &format!("{}.html", &content.slug),
            &tera,
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
        generate_html(
            "list.html",
            &format!("{}.html", &tag_slug),
            &tera,
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
    generate_html(
        "group.html",
        "tags.html",
        &tera,
        &tag_list_context,
        &output_dir,
    )?;

    Ok(())
}

fn generate_html(
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
