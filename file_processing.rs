use crate::site_data::{Content, SiteData};
use walkdir::WalkDir;
use std::fs;
use std::path::Path;
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter};

pub fn process_files(folder: &Path, site_data: &mut SiteData) -> Result<(), String> {
    for entry in WalkDir::new(folder.join(site_data.site.content_path)) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Err(e) = process_file(path, site_data) {
                        eprintln!("Failed to process file {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => eprintln!("Error reading entry: {}", e),
        }
    }
    Ok(())
}

fn process_file(path: &Path, site_data: &mut SiteData) -> Result<(), String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = extract_front_matter(&file_content)?;
    let html = markdown_to_html(markdown, &ComrakOptions::default());

    let content = Content {
        title: get_title(&frontmatter, markdown),
        slug: get_slug(&frontmatter, path),
        tags: get_tags(&frontmatter),
        html,
        date: get_date(&frontmatter),
        show_in_menu: get_show_in_menu(&frontmatter),
    };

    if content.date.is_some() {
        site_data.posts.push(content);
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

// Helper functions like extract_front_matter(), get_title(), etc., similar to as before
