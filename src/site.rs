use crate::config::Marmite;
use crate::content::{group_by_tags, slugify, Content};
use log::{debug, error, info};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process;
use tera::{Context, Tera};

#[derive(Serialize)]
pub struct SiteData<'a> {
    pub site: &'a Marmite<'a>,
    pub posts: Vec<Content>,
    pub pages: Vec<Content>,
}

impl<'a> SiteData<'a> {
    pub fn new(site: &'a Marmite) -> Self {
        SiteData {
            site,
            posts: Vec::new(),
            pages: Vec::new(),
        }
    }
}

pub fn render_templates(
    site_data: &SiteData,
    tera: &Tera,
    output_dir: &Path,
) -> Result<(), String> {
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
