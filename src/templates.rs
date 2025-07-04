#![allow(clippy::module_name_repetitions)]
use crate::embedded::{generate_static, Templates};
use log::{error, info};
use std::fs;
use std::path::Path;

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
pub fn initialize_theme_assets(input_folder: &Path) {
    let static_path = input_folder.join("static");

    if let Err(e) = fs::create_dir(&static_path) {
        error!("Failed to create static directory: {e:?}");
        return;
    }
    generate_static(&static_path);
}
