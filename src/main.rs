use chrono::NaiveDate;
use comrak::{markdown_to_html, ComrakOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
// use std::ptr::metadata;
use tera::{Context, Tera};
use walkdir::WalkDir;

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
    r#"<a href="https://creativecommons.org/licenses/by-nc-sa/4.0/">CC-BY_NC-SA</a> | Site generated with <a href="https://github.com/rochacbruno/marmite">Marmite</a>"#
}

fn default_pagination() -> u32 {
    10
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
struct Content {
    title: String,
    slug: String,
    content: String,
    tags: Option<Vec<String>>,
    date: Option<NaiveDate>,
    show_in_menu: Option<bool>,
}

// impl Content {
//     fn new_post(metadata: &HashMap<String, String>, html: &str) -> Self {
//         Self {
//             title: metadata
//                 .get("title")
//                 .unwrap_or_else(|| &get_default_title(html).to_string())
//                 .to_string(),
//             date: Some(metadata.get("date").unwrap().map())
//         }
//     }
// }

// fn get_default_title(input: &str) -> &str {
//     let first_line = input.lines().next().unwrap_or("");
//     first_line.trim_start_matches('#').trim()
// }

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

fn parse_front_matter(content: &str) -> Option<(HashMap<String, String>, &str)> {
    if let Some(start) = content.find("---") {
        if let Some(end) = content[start + 3..].find("---") {
            let yaml = &content[start + 3..start + 3 + end];
            let markdown = &content[start + 3 + end + 3..];
            let fm: HashMap<String, String> = serde_yaml::from_str(yaml).unwrap();
            return Some((fm, markdown));
        }
    }
    None
}

fn process_file(path: &Path, site_data: &mut SiteData) {
    let content = fs::read_to_string(path).expect("Failed to read file");
    dbg!(&content);

    if let Some((front_matter, markdown)) = parse_front_matter(&content) {
        let content_html = markdown_to_html(markdown, &ComrakOptions::default());

        if let Some(date) = front_matter.get("date") {
            // It is a Post
            site_data.posts.push(Content {
                title: front_matter.get("title").unwrap().to_string(),
                date: Some(NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap()),
                slug: front_matter.get("slug").unwrap().to_string(),
                tags: Some(
                    front_matter
                        .get("tags")
                        .unwrap()
                        .split(",")
                        .map(String::from)
                        .collect(),
                ),
                show_in_menu: None,
                content: content_html,
            });
        } else {
            // It is a Page
            site_data.pages.push(Content {
                title: front_matter.get("title").unwrap().to_string(),
                date: None,
                slug: front_matter.get("slug").unwrap().to_string(),
                tags: None,
                show_in_menu: Some(
                    front_matter
                        .get("show_in_menu")
                        .unwrap_or(&"false".to_string())
                        == "true",
                ),
                content: content_html,
            });
        }
    }
}

fn render_templates(site_data: &SiteData, tera: &Tera, output_dir: &Path) {
    // Render index.html
    let mut context = Context::new();
    context.insert("site", &site_data.site);

    let posts = site_data
        .posts
        .iter()
        .map(|post| {
            let mut post_context = HashMap::new();
            post_context.insert("title", post.title.clone());
            post_context.insert("date", post.date.unwrap().to_string());
            post_context.insert("slug", post.slug.clone());
            post_context.insert("content", post.content.to_string());
            post_context.insert("tags", post.tags.as_ref().unwrap().join(", "));
            post_context
        })
        .collect::<Vec<_>>();

    context.insert("posts", &posts);
    dbg!(&context);

    let pages = site_data
        .pages
        .iter()
        .filter(|p| p.show_in_menu.unwrap_or(false))
        .map(|page| {
            let mut page_context = HashMap::new();
            page_context.insert("title", page.title.clone());
            page_context.insert("slug", page.slug.clone());
            page_context
        })
        .collect::<Vec<_>>();

    context.insert("pages", &pages);

    context.insert("title", "Blog Posts");
    dbg!(&context);
    let index_output = tera.render("list.html", &context).unwrap();
    fs::write(output_dir.join("index.html"), index_output).expect("Unable to write file");

    // Render individual posts and pages
    for post in &site_data.posts {
        let mut post_context = Context::new();
        post_context.insert("site", &site_data.site);
        post_context.insert("title", &post.title);
        post_context.insert("content", &post.content);
        post_context.insert("date", &post.date.unwrap().to_string());
        post_context.insert("tags", &post.tags.as_ref().unwrap().join(", "));

        post_context.insert("pages", &pages);
        dbg!(&post_context);

        let post_output = tera.render("content.html", &post_context).unwrap();
        fs::write(output_dir.join(format!("{}.html", post.slug)), post_output)
            .expect("Unable to write post");
    }

    for page in &site_data.pages {
        let mut page_context = Context::new();
        page_context.insert("title", &page.title);
        page_context.insert("content", &page.content);

        let page_output = tera.render("content.html", &page_context).unwrap();
        fs::write(output_dir.join(format!("{}.html", page.slug)), page_output)
            .expect("Unable to write page");
    }
}

fn main() {
    // Argument Parsing
    let args: Vec<String> = std::env::args().collect();
    let folder = PathBuf::from(&args[1]);

    // Initialize Tera templates
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };

    // Initialize site data
    let marmite = fs::read_to_string("marmite.yaml").expect("Unable to read marmite.yaml");
    let site: Site = serde_yaml::from_str(&marmite).expect("Failed to parse YAML");
    let mut site_data = SiteData::new(&site);

    // Walk through the content directory
    for entry in WalkDir::new(folder.join("content")) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "md" {
            process_file(path, &mut site_data);
        }
    }

    // Sort posts by date (newest first)
    site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));

    // Create the output directory
    let output_dir = folder.join("site");
    fs::create_dir_all(&output_dir).expect("Unable to create output directory");

    // Render templates
    render_templates(&site_data, &tera, &output_dir);
}
