use log::error;
use std::fs;
use std::path::PathBuf;

/// Creates the `templates/` folder and base template file.
pub fn initialize_templates(output_folder: &PathBuf) {
    let templates_path = output_folder.join("templates");
    if let Err(e) = fs::create_dir_all(&templates_path) {
        error!("Failed to create templates directory: {}", e);
        return;
    }

    let base_template_path = templates_path.join("base.html");
    if let Err(e) = fs::write(base_template_path, "<!-- Base HTML template -->") {
        error!("Failed to create base template: {}", e);
    }
}

/// Creates the `static/` folder with basic placeholder files.
pub fn initialize_theme_assets(output_folder: &PathBuf) {
    let static_path = output_folder.join("static");
    if let Err(e) = fs::create_dir_all(&static_path) {
        error!("Failed to create static assets directory: {}", e);
        return;
    }

    let css_path = static_path.join("style.css");
    let js_path = static_path.join("script.js");

    if let Err(e) = fs::write(css_path, "/* Add your styles here */") {
        error!("Failed to create CSS file: {}", e);
    }

    if let Err(e) = fs::write(js_path, "// Add your scripts here") {
        error!("Failed to create JavaScript file: {}", e);
    }
}
