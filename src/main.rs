use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter, Value};
use fs_extra::dir::{copy, CopyOptions};
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

mod cli; // Import the CLI module
mod server; // Import the server module

fn main() -> io::Result<()> {
    let args = cli::Cli::parse();

    let input_folder = args.input_folder;
    let output_folder = Arc::new(args.output_folder); 
    let serve = args.serve;
    let debug = args.debug;
    let config_path = input_folder.join(args.config);
    let bind_address: &str = args.bind.as_str();

    // Initialize site data
    let marmite = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        if debug {
            eprintln!("Unable to read '{}': {}", &config_path.display(), e);
        }
        // Default to empty string if config not found, so defaults are applied
        String::new()
    });
    let site: Marmite = match serde_yaml::from_str(&marmite) {
        Ok(site) => site,
        Err(e) => {
            eprintln!("Failed to parse '{}' YAML: {}", &config_path.display(), e);
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
                eprintln!("Failed to process file {}: {}", entry.path().display(), e);
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
        eprintln!(
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
        eprintln!("Unable to create output directory: {}", e);
        process::exit(1);
    }

    // Initialize Tera templates
    let templates_path = input_folder.join(&site_data.site.templates_path);
    let tera = match Tera::new(&format!("{}/**/*.html", templates_path.display())) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error loading templates: {}", e);
            process::exit(1);
        }
    };

    // Render templates
    if let Err(e) = render_templates(&site_data, &tera, &output_path, debug) {
        eprintln!("Failed to render templates: {}", e);
        process::exit(1);
    }

    // Copy static folder if present
    let static_source = input_folder.join(site_data.site.static_path);
    if static_source.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true; // Overwrite files if they already exist

        if let Err(e) = copy(&static_source, &*output_folder, &options) {
            eprintln!("Failed to copy static directory: {}", e);
            process::exit(1);
        }

        println!(
            "Copied '{}' to '{}/'",
            &static_source.display(),
            &output_folder.display()
        );
    }

    // Serve the site if the flag was provided
    if serve {
        println!("Starting built-in HTTP server...");
        server::start_server(&bind_address, output_folder.clone().into());
    }

    println!("Site generated at: {}/", output_folder.display());

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
        eprintln!(
            "ERROR: Invalid date format {} when parsing {}",
            input.to_string_representation(),
            path.display()
        );
        process::exit(1);
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
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap()
        .to_string()
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

fn render_templates(
    site_data: &SiteData,
    tera: &Tera,
    output_dir: &Path,
    debug: bool,
) -> Result<(), String> {
    // Build the context of variables that are global on every template
    let mut global_context = Context::new();
    global_context.insert("site_data", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);
    if debug {
        println!("Global Context: {:?}", &site_data.site)
    }

    // Render index.html from list.html template
    let mut list_context = global_context.clone();
    list_context.insert("title", site_data.site.list_title);
    list_context.insert("content_list", &site_data.posts);
    if debug {
        println!(
            "Index Context: {:?}",
            &site_data
                .posts
                .iter()
                .map(|p| format!("{},{}", p.title, p.slug))
                .collect::<Vec<_>>()
        )
    }
    generate_html("list.html", "index.html", &tera, &list_context, output_dir)?;

    // Render pages.html from list.html template
    let mut list_context = global_context.clone();
    list_context.insert("title", site_data.site.pages_title);
    list_context.insert("content_list", &site_data.pages);
    if debug {
        println!(
            "Pages Context: {:?}",
            &site_data
                .pages
                .iter()
                .map(|p| format!("{},{}", p.title, p.slug))
                .collect::<Vec<_>>()
        )
    }
    generate_html("list.html", "pages.html", &tera, &list_context, output_dir)?;

    // Render individual content-slug.html from content.html template
    for content in site_data.posts.iter().chain(&site_data.pages) {
        let mut content_context = global_context.clone();
        content_context.insert("title", &content.title);
        content_context.insert("content", &content);
        if debug {
            println!(
                "{} context: {:?}",
                &content.slug,
                format!(
                    "title: {},date: {:?},tags: {:?}",
                    &content.title, &content.date, &content.tags
                )
            )
        }
        generate_html(
            "content.html",
            &format!("{}.html", &content.slug),
            &tera,
            &content_context,
            output_dir,
        )?;
    }

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
        eprintln!("Error rendering template `{}`: {}", template, e);
        e.to_string()
    })?;

    fs::write(output_dir.join(filename), rendered).map_err(|e| e.to_string())?;
    println!("Generated {filename}");
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct Marmite<'a> {
    #[serde(default = "default_name")]
    name: &'a str,
    #[serde(default = "default_tagline")]
    tagline: &'a str,
    #[serde(default = "default_url")]
    url: &'a str,
    #[serde(default = "default_footer")]
    footer: &'a str,
    #[serde(default = "default_pagination")]
    pagination: u32,

    #[serde(default = "default_list_title")]
    list_title: &'a str,
    #[serde(default = "default_pages_title")]
    pages_title: &'a str,
    #[serde(default = "default_tags_title")]
    tags_title: &'a str,
    #[serde(default = "default_archives_title")]
    archives_title: &'a str,

    #[serde(default = "default_content_path")]
    content_path: &'a str,
    #[serde(default = "default_site_path")]
    site_path: &'a str,
    #[serde(default = "default_templates_path")]
    templates_path: &'a str,
    #[serde(default = "default_static_path")]
    static_path: &'a str,
    #[serde(default = "default_media_path")]
    media_path: &'a str,

    #[serde(default = "default_card_image")]
    card_image: &'a str,
    #[serde(default = "default_logo_image")]
    logo_image: &'a str,

    #[serde(default = "default_menu")]
    menu: Option<Vec<(String, String)>>,

    #[serde(default = "default_data")]
    data: Option<HashMap<String, String>>,
}

fn default_name() -> &'static str {
    "Home"
}

fn default_tagline() -> &'static str {
    "Site generated from markdown content"
}

fn default_url() -> &'static str {
    "https://example.com"
}

fn default_footer() -> &'static str {
    r#"<a href="https://creativecommons.org/licenses/by-nc-sa/4.0/">CC-BY_NC-SA</a> | Site generated with <a href="https://github.com/rochacbruno/marmite">Marmite</a>"#
}

fn default_pagination() -> u32 {
    10
}

fn default_list_title() -> &'static str {
    "Posts"
}

fn default_tags_title() -> &'static str {
    "Tags"
}

fn default_pages_title() -> &'static str {
    "Pages"
}

fn default_archives_title() -> &'static str {
    "Archive"
}

fn default_site_path() -> &'static str {
    ""
}

fn default_content_path() -> &'static str {
    "content"
}

fn default_templates_path() -> &'static str {
    "templates"
}

fn default_static_path() -> &'static str {
    "static"
}

fn default_media_path() -> &'static str {
    "content/media"
}

fn default_card_image() -> &'static str {
    ""
}

fn default_logo_image() -> &'static str {
    ""
}

fn default_menu() -> Option<Vec<(String, String)>> {
    vec![("Pages".to_string(), "pages.html".to_string())].into()
}

fn default_data() -> Option<HashMap<String, String>> {
    None
}