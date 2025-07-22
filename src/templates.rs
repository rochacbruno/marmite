#![allow(clippy::module_name_repetitions)]
use crate::embedded::{generate_static, Templates};
use log::{error, info};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Creates the `templates/` folder and writes embedded templates to it.
pub fn initialize_templates(input_folder: &Path) {
    let templates_path = input_folder.join("templates");
    if let Err(e) = fs::create_dir(&templates_path) {
        error!("Failed to create templates directory: {e:?}");
        return;
    }

    for name in Templates::iter() {
        if let Some(template) = Templates::get(name.as_ref()) {
            if let Ok(template_str) = std::str::from_utf8(template.data.as_ref()) {
                let template_path = templates_path.join(name.as_ref());
                if let Err(e) = fs::write(&template_path, template_str) {
                    error!("Failed to write template '{}': {e:?}", name.as_ref());
                } else {
                    info!("Generated {}", template_path.display());
                }
            } else {
                error!("Failed to parse template '{}' as UTF-8", name.as_ref());
            }
        } else {
            error!(
                "Template '{}' not found in embedded resources",
                name.as_ref()
            );
        }
    }
}

/// Uses `generate_static` to populate the `static/` folder with embedded content.
#[allow(dead_code)]
pub fn initialize_theme_assets(input_folder: &Path) {
    let static_path = input_folder.join("static");

    if let Err(e) = fs::create_dir(&static_path) {
        error!("Failed to create static directory: {e:?}");
        return;
    }
    generate_static(&static_path);
}

/// Creates a new theme with templates and static assets from the theme template
pub fn initialize_theme(input_folder: &Path, theme_name: &str) {
    // Validate theme name
    if theme_name.is_empty() || theme_name.contains('/') || theme_name.contains('\\') {
        error!("Invalid theme name: '{theme_name}'");
        return;
    }

    let theme_path = input_folder.join(theme_name);

    // Check if theme directory already exists
    if theme_path.exists() {
        error!("Theme directory already exists: {}", theme_path.display());
        return;
    }

    // Create theme directory structure
    let templates_path = theme_path.join("templates");
    let static_path = theme_path.join("static");

    if let Err(e) = fs::create_dir_all(&templates_path) {
        error!("Failed to create theme templates directory: {e:?}");
        return;
    }

    if let Err(e) = fs::create_dir_all(&static_path) {
        error!("Failed to create theme static directory: {e:?}");
        return;
    }

    // Copy theme template files from example/theme_template
    let theme_template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("example")
        .join("theme_template");

    if !theme_template_path.exists() {
        error!(
            "Theme template directory not found: {}",
            theme_template_path.display()
        );
        return;
    }

    // Copy template files
    let template_source = theme_template_path.join("templates");
    if template_source.exists() {
        copy_directory_contents(&template_source, &templates_path, "template");
    }

    // Copy static files
    let static_source = theme_template_path.join("static");
    if static_source.exists() {
        copy_directory_contents(&static_source, &static_path, "static");
    }

    // Copy theme metadata files
    let theme_json = theme_template_path.join("theme.json");
    if theme_json.exists() {
        if let Err(e) = fs::copy(&theme_json, theme_path.join("theme.json")) {
            error!("Failed to copy theme.json: {e:?}");
        } else {
            info!("Generated {}", theme_path.join("theme.json").display());
        }
    }

    let theme_md = theme_template_path.join("theme.md");
    if theme_md.exists() {
        if let Err(e) = fs::copy(&theme_md, theme_path.join("theme.md")) {
            error!("Failed to copy theme.md: {e:?}");
        } else {
            info!("Generated {}", theme_path.join("theme.md").display());
        }
    }

    info!(
        "Theme '{}' created successfully at {}",
        theme_name,
        theme_path.display()
    );
    info!("To use this theme, add 'theme: {theme_name}' to your marmite.yaml config file",);
}

/// Helper function to copy directory contents recursively
fn copy_directory_contents(source: &Path, destination: &Path, file_type: &str) {
    for entry in WalkDir::new(source) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                error!("Error walking directory: {e:?}");
                continue;
            }
        };

        let source_path = entry.path();
        let relative_path = source_path.strip_prefix(source).unwrap();
        let dest_path = destination.join(relative_path);

        if entry.file_type().is_dir() {
            if let Err(e) = fs::create_dir_all(&dest_path) {
                error!("Failed to create directory {}: {e:?}", dest_path.display());
            }
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("Failed to create parent directory: {e:?}");
                    continue;
                }
            }

            if let Err(e) = fs::copy(source_path, &dest_path) {
                error!(
                    "Failed to copy {} file '{}': {e:?}",
                    file_type,
                    source_path.display()
                );
            } else {
                info!("Generated {}", dest_path.display());
            }
        }
    }
}
