#![allow(clippy::module_name_repetitions)]
use crate::embedded::{generate_static, Templates, ThemeTemplate};
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
#[allow(dead_code)]
pub fn initialize_theme_assets(input_folder: &Path) {
    let static_path = input_folder.join("static");

    if let Err(e) = fs::create_dir(&static_path) {
        error!("Failed to create static directory: {e:?}");
        return;
    }
    generate_static(&static_path);
}

/// Creates a new theme with templates and static assets from the embedded theme template
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

    // Copy theme template files from embedded ThemeTemplate
    for name in ThemeTemplate::iter() {
        if let Some(file) = ThemeTemplate::get(name.as_ref()) {
            let file_path = Path::new(name.as_ref());
            let dest_path = theme_path.join(file_path);

            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("Failed to create directory {}: {e:?}", parent.display());
                    continue;
                }
            }

            // Write file data
            if let Err(e) = fs::write(&dest_path, file.data.as_ref()) {
                error!("Failed to copy theme file '{}': {e:?}", name.as_ref());
            } else {
                info!("Generated {}", dest_path.display());
            }
        } else {
            error!(
                "Theme template file '{}' not found in embedded resources",
                name.as_ref()
            );
        }
    }

    info!(
        "Theme '{}' created successfully at {}",
        theme_name,
        theme_path.display()
    );
    info!("To use this theme, add 'theme: {theme_name}' to your marmite.yaml config file",);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_templates() {
        let temp_dir = TempDir::new().unwrap();
        let input_folder = temp_dir.path();

        initialize_templates(input_folder);

        let templates_path = input_folder.join("templates");
        assert!(templates_path.exists());
        assert!(templates_path.is_dir());
    }

    #[test]
    fn test_initialize_theme_assets() {
        let temp_dir = TempDir::new().unwrap();
        let input_folder = temp_dir.path();

        initialize_theme_assets(input_folder);

        let static_path = input_folder.join("static");
        assert!(static_path.exists());
        assert!(static_path.is_dir());
    }

    #[test]
    fn test_initialize_theme_valid_name() {
        let temp_dir = TempDir::new().unwrap();
        let input_folder = temp_dir.path();
        let theme_name = "my_theme";

        initialize_theme(input_folder, theme_name);

        let theme_path = input_folder.join(theme_name);
        assert!(theme_path.exists());
        assert!(theme_path.is_dir());

        let templates_path = theme_path.join("templates");
        let static_path = theme_path.join("static");
        assert!(templates_path.exists());
        assert!(static_path.exists());
    }

    #[test]
    fn test_initialize_theme_invalid_name() {
        let temp_dir = TempDir::new().unwrap();
        let input_folder = temp_dir.path();

        // Test empty name - should not create any directory
        let original_entries = std::fs::read_dir(input_folder).unwrap().count();
        initialize_theme(input_folder, "");
        let new_entries = std::fs::read_dir(input_folder).unwrap().count();
        assert_eq!(original_entries, new_entries);

        // Test name with slash
        initialize_theme(input_folder, "invalid/theme");
        assert!(!input_folder.join("invalid/theme").exists());

        // Test name with backslash
        initialize_theme(input_folder, "invalid\\theme");
        assert!(!input_folder.join("invalid\\theme").exists());
    }

    #[test]
    fn test_initialize_theme_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let input_folder = temp_dir.path();
        let theme_name = "existing_theme";
        let theme_path = input_folder.join(theme_name);

        // Create the theme directory first
        fs::create_dir(&theme_path).unwrap();

        // Try to initialize theme with existing directory
        initialize_theme(input_folder, theme_name);

        // Should still exist but not be modified
        assert!(theme_path.exists());
    }
}
