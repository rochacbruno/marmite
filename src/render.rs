use crate::site_data::{SiteData};
use tera::{Tera, Context};
use std::fs;
use std::path::Path;

pub fn render_templates(site_data: &SiteData, output_dir: &Path) -> Result<(), String> {
    let tera = Tera::new(format!("{}/**/*", site_data.site.templates_path).as_str())
        .map_err(|e| e.to_string())?;

    // Render index.html
    let mut context = Context::new();
    context.insert("site", &site_data.site);
    context.insert("pages", &site_data.pages);
    context.insert("posts", &site_data.posts);
    context.insert("title", "Blog Posts");
    let index_output = tera.render("list.html", &context).map_err(|e| e.to_string())?;
    fs::write(output_dir.join("index.html"), index_output).map_err(|e| e.to_string())?;

    // Render individual posts and pages
    for content in site_data.posts.iter().chain(site_data.pages.iter()) {
        let mut content_context = Context::new();
        content_context.insert("site", &site_data.site);
        content_context.insert("title", &content.title);
        content_context.insert("content", &content);
        let content_output = tera.render("content.html", &content_context).map_err(|e| e.to_string())?;
        fs::write(output_dir.join(format!("{}.html", content.slug)), content_output)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
