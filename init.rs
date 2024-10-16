use std::fs;
use std::path::Path;

pub fn init_project(folder: &Path) -> Result<(), String> {
    let content_path = folder.join("content");
    let templates_path = folder.join("templates");
    let static_path = folder.join("static");

    fs::create_dir_all(&content_path).map_err(|e| e.to_string())?;
    fs::create_dir_all(&templates_path).map_err(|e| e.to_string())?;

    // Create default templates
    let list_template = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>{{ title }}</title>
    </head>
    <body>
        <h1>{{ site.name }}</h1>
        <h2>{{ site.tagline }}</h2>
        <ul>
        {% for post in posts %}
            <li><a href="{{ post.slug }}.html">{{ post.title }}</a></li>
        {% endfor %}
        </ul>
    </body>
    </html>
    "#;
    fs::write(templates_path.join("list.html"), list_template).map_err(|e| e.to_string())?;

    let content_template = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>{{ title }}</title>
    </head>
    <body>
        <h1>{{ content.title }}</h1>
        <div>{{ content.html | safe }}</div>
    </body>
    </html>
    "#;
    fs::write(templates_path.join("content.html"), content_template).map_err(|e| e.to_string())?;
    fs::create_dir_all(&static_path).map_err(|e| e.to_string())?;

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
