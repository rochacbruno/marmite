use std::fs;
use std::path::{Path, PathBuf};

pub fn init_project(folder: &Path) -> Result<(), String> {
    let content_path = folder.join("content");
    let templates_path = folder.join("templates");
    let static_path = folder.join("static");

    fs::create_dir_all(&content_path).map_err(|e| e.to_string())?;
    fs::create_dir_all(&templates_path).map_err(|e| e.to_string())?;
    fs::create_dir_all(&static_path).map_err(|e| e.to_string())?;
    
    // Create default content files
    let about_md = r#"---
title: "About"
---
This is the about page."#;
    fs::write(content_path.join("about.md"), about_md).map_err(|e| e.to_string())?;

    let no_metadata_md = "This post has no metadata.";
    fs::write(content_path.join("no-metadata.md"), no_metadata_md).map_err(|e| e.to_string())?;

    let simple_post_md = r#"---
title: "Simple Blog Post"
date: "2024-10-16"
tags: ["intro"]
---
This is a simple blog post."#;
fs::write(content_path.join("simple-post.md"), simple_post_md).map_err(|e| e.to_string())?;

    
    // Create default templates
    let base_template = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>{{ title | default(value=site.name) }}</title>
    </head>
    <body>
        <header>
            <h1>{{ site.name }}</h1>
            <p>{{ site.tagline }}</p>
        </header>
        <main>
            {% block content %}
            {% if content %}
                <div>{{ content }}</div>
            {% endif %}
            {% endblock %}
        </main>
        <footer>
            <p>{{ site.footer }}</p>
        </footer>
    </body>
    </html>
    "#;
    fs::write(templates_path.join("base.html"), base_template).map_err(|e| e.to_string())?;

    let list_template = r#"
    {% extends "base.html" %}

    {% block content %}
    <h2>Blog Posts</h2>
    <ul>
        {% for post in posts %}
            <li><a href="{{ post.slug }}.html">{{ post.title }}</a></li>
        {% endfor %}
    </ul>
    {% endblock %}
    "#;
    fs::write(templates_path.join("list.html"), list_template).map_err(|e| e.to_string())?;

    let content_template = r#"
    {% extends "base.html" %}

    {% block content %}
    <h2>{{ content.title }}</h2>
    <div>{{ content.html | safe }}</div>
    {% endblock %}
    "#;
    fs::write(templates_path.join("content.html"), content_template).map_err(|e| e.to_string())?;

    let default_config = r#"name: "My Marmite Site"
tagline: "A static site generated with Marmite"
url: "https://example.com"
footer: "CC-BY_NC-SA | Site generated with Marmite"
content_path: "content"
templates_path: "templates"
static_path: "static"
site_path: "site"
"#;
    fs::write(folder.join("marmite.yaml"), default_config).map_err(|e| e.to_string())?;
    Ok(())
}
