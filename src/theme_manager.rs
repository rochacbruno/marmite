use log::{error, info};
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ThemeMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub marmite_version: Option<String>,
    pub features: Option<Vec<String>>,
}

/// Downloads and sets a theme from a remote URL or local folder
pub fn set_theme(input_folder: &Path, theme_source: &str, config_theme: Option<String>) {
    info!("Setting theme from: {theme_source}");

    let theme_name = if theme_source.starts_with("http://") || theme_source.starts_with("https://")
    {
        // Download remote theme
        match download_theme(input_folder, theme_source) {
            Ok(name) => name,
            Err(e) => {
                error!("Failed to download theme: {e}");
                return;
            }
        }
    } else {
        // Local theme folder
        let theme_path = input_folder.join(theme_source);
        if !theme_path.exists() {
            error!("Theme folder does not exist: {}", theme_path.display());
            return;
        }
        theme_source.to_string()
    };

    // Validate theme.json exists
    let theme_path = input_folder.join(&theme_name);
    let theme_json_path = theme_path.join("theme.json");

    if !theme_json_path.exists() {
        error!(
            "theme.json not found in theme folder: {}",
            theme_path.display()
        );
        // Remove the theme folder if it was downloaded
        if theme_source.starts_with("http://") || theme_source.starts_with("https://") {
            if let Err(e) = fs::remove_dir_all(&theme_path) {
                error!("Failed to remove invalid theme folder: {e}");
            }
        }
        return;
    }

    // Read and parse theme.json
    let theme_metadata = match read_theme_metadata(&theme_json_path) {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Failed to read theme metadata: {e}");
            return;
        }
    };

    // Update marmite.yaml
    if let Err(e) = update_config_theme(input_folder, &theme_name, config_theme) {
        error!("Failed to update configuration: {e}");
        return;
    }

    // Display success message with metadata
    println!("\n✅ Theme '{theme_name}' has been successfully set!\n");
    println!("📦 Theme Information:");
    println!("   Name: {}", theme_metadata.name);
    println!("   Version: {}", theme_metadata.version);
    println!("   Author: {}", theme_metadata.author);
    println!("   Description: {}", theme_metadata.description);

    if let Some(features) = &theme_metadata.features {
        println!("\n✨ Features:");
        for feature in features {
            println!("   - {feature}");
        }
    }

    if let Some(tags) = &theme_metadata.tags {
        println!("\n🏷️  Tags: {}", tags.join(", "));
    }

    println!(
        "\n🚀 To use this theme, run: marmite {} --theme {}",
        input_folder.display(),
        theme_name
    );
}

/// Downloads a theme from a remote URL
fn download_theme(input_folder: &Path, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let download_url = determine_download_url(url)?;
    let theme_name = extract_theme_name(url)?;

    info!("Downloading theme from: {download_url}");

    // Create temporary directory for download
    let temp_dir = input_folder.join(".theme_download_temp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir(&temp_dir)?;

    // Download the zip file
    let zip_path = temp_dir.join("theme.zip");
    download_file(&download_url, &zip_path)?;

    // Extract the zip file
    extract_zip(&zip_path, &temp_dir)?;

    // Find the theme root directory (it might be nested)
    let theme_root = find_theme_root(&temp_dir)?;

    // Move to final location
    let final_path = input_folder.join(&theme_name);
    if final_path.exists() {
        return Err(format!("Theme '{theme_name}' already exists").into());
    }

    fs::rename(&theme_root, &final_path)?;

    // Clean up
    fs::remove_dir_all(&temp_dir)?;

    Ok(theme_name)
}

/// Determines the download URL based on the repository URL
fn determine_download_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    if std::path::Path::new(url)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        // Direct zip file URL
        Ok(url.to_string())
    } else if url.contains("github.com") {
        // GitHub repository
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        if parts.len() < 5 {
            return Err("Invalid GitHub URL".into());
        }
        let owner = parts[3];
        let repo = parts[4];
        let branch = if parts.len() > 6 && parts[5] == "tree" {
            parts[6]
        } else {
            "main"
        };
        Ok(format!(
            "https://github.com/{owner}/{repo}/archive/refs/heads/{branch}.zip"
        ))
    } else if url.contains("gitlab.com") {
        // GitLab repository
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        if parts.len() < 5 {
            return Err("Invalid GitLab URL".into());
        }
        let owner = parts[3];
        let repo = parts[4];
        let branch = if parts.len() > 6 && parts[5] == "-/tree" {
            parts[6]
        } else {
            "main"
        };
        Ok(format!(
            "https://gitlab.com/{owner}/{repo}/-/archive/{branch}/{repo}-{branch}.zip"
        ))
    } else if url.contains("codeberg.org") {
        // Codeberg/Forgejo repository
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        if parts.len() < 5 {
            return Err("Invalid Codeberg URL".into());
        }
        let owner = parts[3];
        let repo = parts[4];
        let branch = if parts.len() > 6 && parts[5] == "src" && parts[6] == "branch" {
            parts[7]
        } else {
            "main"
        };
        Ok(format!(
            "https://codeberg.org/{owner}/{repo}/archive/{branch}.zip"
        ))
    } else {
        Err("Unsupported repository host. Supported: GitHub, GitLab, Codeberg".into())
    }
}

/// Extracts the theme name from the URL
fn extract_theme_name(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    if std::path::Path::new(url)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        // Extract from zip filename
        let path = std::path::Path::new(url);
        let filename = path
            .file_stem()
            .ok_or("Invalid URL: no filename")?
            .to_str()
            .ok_or("Invalid URL: filename not UTF-8")?;
        Ok(filename.to_string())
    } else {
        // Extract from repository URL
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        if parts.len() < 5 {
            return Err("Invalid repository URL".into());
        }
        Ok(parts[4].to_string())
    }
}

/// Downloads a file from a URL
fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = ureq::get(url).call()?;
    let mut file = fs::File::create(dest)?;
    let mut body = response.into_body();
    io::copy(&mut body.as_reader(), &mut file)?;
    Ok(())
}

/// Extracts a zip file
fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Finds the theme root directory (contains theme.json)
fn find_theme_root(dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // First check if theme.json is in the root
    if dir.join("theme.json").exists() {
        return Ok(dir.to_path_buf());
    }

    // Otherwise look for it in subdirectories (common with GitHub downloads)
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("theme.json").exists() {
            return Ok(path);
        }
    }

    Err("theme.json not found in downloaded theme".into())
}

/// Reads theme metadata from theme.json
fn read_theme_metadata(path: &Path) -> Result<ThemeMetadata, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let metadata: ThemeMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}

/// Updates the marmite.yaml configuration file with the new theme
fn update_config_theme(
    input_folder: &Path,
    theme_name: &str,
    _config_theme: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = input_folder.join("marmite.yaml");

    if config_path.exists() {
        // Update existing config
        info!("Updating marmite.yaml with theme: {theme_name}");

        let content = fs::read_to_string(&config_path)?;
        let mut lines: Vec<String> = content
            .lines()
            .map(std::string::ToString::to_string)
            .collect();

        // Find and update theme line
        let mut theme_found = false;
        for line in &mut lines {
            if line.trim_start().starts_with("theme:") {
                *line = format!("theme: {theme_name}");
                theme_found = true;
                break;
            }
        }

        // If theme not found, add it
        if !theme_found {
            lines.push(format!("theme: {theme_name}"));
        }

        // Write back
        let mut file = fs::File::create(&config_path)?;
        for line in lines {
            writeln!(file, "{line}")?;
        }
    } else {
        // Create new config with theme
        info!("Creating new marmite.yaml with theme: {theme_name}");

        // Use marmite's generate config functionality
        let mut cmd = std::process::Command::new(std::env::current_exe()?);
        cmd.arg(input_folder)
            .arg("--generate-config")
            .arg("--theme")
            .arg(theme_name);

        let output = cmd.output()?;
        if !output.status.success() {
            return Err("Failed to generate config file".into());
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/theme_manager.rs"]
mod tests;
