use chrono::{NaiveDate, NaiveDateTime};
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter, Value};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use tera::{Context, Tera};
use walkdir::WalkDir;

fn main() {
    // Argument Parsing
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: program <folder>");
        process::exit(1);
    }
    let folder = PathBuf::from(&args[1]);

    // Initialize site data
    let marmite = match fs::read_to_string("marmite.yaml") {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Unable to read marmite.yaml: {}", e);
            process::exit(1);
        }
    };
    let site: Site = match serde_yaml::from_str(&marmite) {
        Ok(site) => site,
        Err(e) => {
            eprintln!("Failed to parse YAML: {}", e);
            process::exit(1);
        }
    };
    let mut site_data = SiteData::new(&site);

    // Walk through the content directory
    for entry in WalkDir::new(folder.join(site_data.site.content_path)) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Err(e) = process_file(path, &mut site_data) {
                        eprintln!("Failed to process file {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => eprintln!("Error reading entry: {}", e),
        }
    }

    // Sort posts by date (newest first)
    site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));
    // Sort pages on title
    site_data.pages.sort_by(|a, b| b.title.cmp(&a.title));

    // Create the output directory
    if let Err(e) = fs::create_dir_all(folder.join(&site_data.site.site_path)) {
        eprintln!("Unable to create output directory: {}", e);
        process::exit(1);
    }

    // Initialize Tera templates
    let tera = match Tera::new(format!("{}/**/*", site_data.site.templates_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    // Render templates
    if let Err(e) = render_templates(&site_data, &tera, &folder.join(&site_data.site.site_path)) {
        eprintln!("Failed to render templates: {}", e);
        process::exit(1);
    }

    println!("Site generated at: {}/", site_data.site.site_path);
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
struct Content {
    title: String,
    slug: String,
    html: String,
    tags: Vec<String>,
    date: Option<NaiveDateTime>,
    show_in_menu: bool,
}

struct SiteData<'a> {
    site: &'a Site<'a>,
    posts: Vec<Content>,
    pages: Vec<Content>,
}

impl<'a> SiteData<'a> {
    fn new(site: &'a Site) -> Self {
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
    // TODO: Trim empty first and trailing lines of markdown
    let html = markdown_to_html(markdown, &ComrakOptions::default());

    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, &path);
    let date = get_date(&frontmatter, &path);
    let show_in_menu = get_show_in_menu(&frontmatter);

    let content = Content {
        title,
        slug,
        tags,
        html,
        date,
        show_in_menu,
    };

    if date.is_some() {
        site_data.posts.push(content);
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

fn get_show_in_menu(frontmatter: &Frontmatter) -> bool {
    if let Some(show_in_menu) = frontmatter.get("show_in_menu") {
        return show_in_menu.as_bool().unwrap_or(false);
    }
    false
}

fn get_date(frontmatter: &Frontmatter, path: &Path) -> Option<NaiveDateTime> {
    if let Some(input) = frontmatter.get("date") {
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M:%S")
        {
            return Some(date);
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M")
        {
            return Some(date);
        } else if let Ok(date) = NaiveDate::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d") {
            // Add a default time (00:00:00)
            return date.and_hms_opt(0, 0, 0);
        } else {
            eprintln!(
                "ERROR: Invalid date format {} when parsing {}",
                input.to_string_representation(),
                path.display()
            );
            process::exit(1);
        }
    }
    None
}

fn get_slug<'a>(frontmatter: &'a Frontmatter, path: &'a Path) -> String {
    match frontmatter.get("slug") {
        Some(Value::String(slug)) => slug.to_string(),
        _ => path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_string(),
    }
}

fn get_title<'a>(frontmatter: &'a Frontmatter, html: &'a str) -> String {
    match frontmatter.get("title") {
        Some(Value::String(t)) => t.to_string(),
        _ => html
            .lines()
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
    // Render index.html
    let mut context = Context::new();
    context.insert("site", &site_data.site);
    context.insert("pages", &site_data.pages);
    context.insert("posts", &site_data.posts);
    context.insert("title", "Blog Posts"); // Get from marmite.yaml
    let index_output = tera.render("list.html", &context).map_err(|e| e.to_string())?;
    fs::write(output_dir.join("index.html"), index_output).map_err(|e| e.to_string())?;

    // Render individual posts and pages
    for post in &site_data.posts {
        let mut post_context = Context::new();
        post_context.insert("site", &site_data.site);
        post_context.insert("pages", &site_data.pages);
        post_context.insert("title", &post.title);
        post_context.insert("content", &post);
        let post_output = tera.render("content.html", &post_context).map_err(|e| e.to_string())?;
        fs::write(output_dir.join(format!("{}.html", post.slug)), post_output)
            .map_err(|e| e.to_string())?;
    }

    for page in &site_data.pages {
        let mut page_context = Context::new();
        page_context.insert("site", &site_data.site);
        page_context.insert("pages", &site_data.pages);
        page_context.insert("title", &page.title);
        page_context.insert("content", &page);
        let page_output = tera.render("content.html", &page_context).map_err(|e| e.to_string())?;
        fs::write(output_dir.join(format!("{}.html", page.slug)), page_output)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
struct Site<'a> {
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
    #[serde(default = "default_tags_title")]
    tags_title: &'a str,
    #[serde(default = "default_content_path")]
    content_path: &'a str,
    #[serde(default = "default_templates_path")]
    templates_path: &'a str,
    #[serde(default = "default_static_path")]
    static_path: &'a str,
    #[serde(default = "default_media_path")]
    media_path: &'a str,
    #[serde(default = "default_site_path")]
    site_path: &'a str,
}

fn default_name() -> &'static str {
    "Marmite Site"
}

fn default_tagline() -> &'static str {
    "A website generated with Marmite"
}

fn default_url() -> &'static str {
    "https://example.com"
}

fn default_footer() -> &'static str {
    r#"<a href=\"https://creativecommons.org/licenses/by-nc-sa/4.0/\">CC-BY_NC-SA</a> | Site generated with <a href=\"https://github.com/rochacbruno/marmite\">Marmite</a>"#
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

fn default_site_path() -> &'static str {
    "site"
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
