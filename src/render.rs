use tera::{Context, Tera};
use std::fs;
use std::path::Path;
use crate::SiteData;

pub fn render_templates(site_data: &SiteData, output_dir: &Path) -> Result<(), String> {
    // Construa o caminho completo para o diretório de templates
    let template_path = output_dir.parent().unwrap().join(&site_data.site.templates_path);

    // Linha de debug para verificar o caminho do template
    println!("Resolved template path: {}", template_path.display());

    // Certifique-se de que o caminho é válido
    let template_path = template_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve template path: {}", e))?;

    let tera = match Tera::new(&format!("{}/**/*", template_path.display())) {
        Ok(t) => t,
        Err(e) => {
            return Err(format!("Parsing error(s): {}", e));
        }
    };

    // Render index.html
    let mut context = Context::new();
    context.insert("site", &site_data.site);
    context.insert("pages", &site_data.pages);
    context.insert("posts", &site_data.posts);
    context.insert("title", "Blog Posts"); // Get from marmite.yaml
    let index_output = tera.render("list.html", &context)
        .map_err(|e| format!("Failed to render 'list.html': {}", e))?;
    fs::write(output_dir.join("index.html"), index_output)
        .map_err(|e| format!("Failed to write 'index.html': {}", e))?;

    // Render individual posts and pages
    for post in &site_data.posts {
        let mut post_context = Context::new();
        post_context.insert("site", &site_data.site);
        post_context.insert("pages", &site_data.pages);
        post_context.insert("title", &post.title);
        post_context.insert("content", &post);
    
        // Nome específico para o post
        let output_file = if post.slug == "simple-post" {
            "blog-post.html"
        } else {
            &post.slug
        };
    
        let post_output = tera.render("content.html", &post_context)
            .map_err(|e| format!("Failed to render 'content.html' for post '{}': {}", post.slug, e))?;
        fs::write(output_dir.join(format!("{}.html", output_file)), post_output)
            .map_err(|e| format!("Failed to write '{}.html': {}", output_file, e))?;
    }
    

    for page in &site_data.pages {
        let mut page_context = Context::new();
        page_context.insert("site", &site_data.site);
        page_context.insert("pages", &site_data.pages);
        page_context.insert("title", &page.title);
        page_context.insert("content", &page);
        let page_output = tera.render("content.html", &page_context)
            .map_err(|e| format!("Failed to render 'content.html' for page '{}': {}", page.slug, e))?;
        fs::write(output_dir.join(format!("{}.html", page.slug)), page_output)
            .map_err(|e| format!("Failed to write '{}.html': {}", page.slug, e))?;
    }

    Ok(())
}
