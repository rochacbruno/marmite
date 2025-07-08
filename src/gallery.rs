use crate::config::Marmite;
use crate::site::Data;
use image::{imageops, GenericImageView, ImageFormat};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaGalleryConfig {
    pub source: String,
    pub extensions: Vec<String>,
    pub thumbnail_size: u32,
    pub aggregation_pattern: String,
}

impl Default for MediaGalleryConfig {
    fn default() -> Self {
        Self {
            source: "gallery".to_string(),
            extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
            thumbnail_size: 300,
            aggregation_pattern: r"(.+)_\d+$".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaItem {
    pub filename: String,
    pub path: String,
    pub thumbnail_path: String,
    pub title: String,
    pub description: Option<String>,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GalleryGroup {
    pub name: String,
    pub items: Vec<MediaItem>,
    pub thumbnail: Option<String>,
}

pub fn get_media_gallery_config(config: &Marmite) -> Option<MediaGalleryConfig> {
    config
        .extra
        .as_ref()?
        .get("media_gallery")
        .and_then(|v| match v {
            Value::String(source) => Some(MediaGalleryConfig {
                source: source.clone(),
                ..Default::default()
            }),
            Value::Mapping(map) => {
                let mut gallery_config = MediaGalleryConfig::default();

                if let Some(Value::String(source)) = map.get("source") {
                    gallery_config.source = source.clone();
                }

                if let Some(Value::Sequence(extensions)) = map.get("extensions") {
                    gallery_config.extensions = extensions
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }

                if let Some(Value::Number(size)) = map.get("thumbnail_size") {
                    if let Some(size_u64) = size.as_u64() {
                        gallery_config.thumbnail_size = size_u64 as u32;
                    }
                }

                Some(gallery_config)
            }
            _ => None,
        })
}

pub fn handle_media_gallery(
    input_folder: &Path,
    site_data: &Data,
    output_folder: &Path,
    content_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let gallery_config = match get_media_gallery_config(&site_data.site) {
        Some(config) => config,
        None => {
            debug!("No media gallery configuration found");
            return Ok(());
        }
    };

    info!("Processing media gallery from: {}", gallery_config.source);

    let gallery_source = input_folder.join(&gallery_config.source);
    if !gallery_source.exists() {
        warn!(
            "Gallery source directory does not exist: {}",
            gallery_source.display()
        );
        return Ok(());
    }

    let gallery_output = output_folder
        .join(&site_data.site.site_path)
        .join("gallery");
    let thumbnails_output = gallery_output.join("thumbnails");

    // Create output directories
    fs::create_dir_all(&gallery_output)?;
    fs::create_dir_all(&thumbnails_output)?;

    // Scan for media files
    let media_files = scan_media_files(&gallery_source, &gallery_config)?;

    // Group media files by aggregation pattern
    let grouped_media = group_media_files(media_files, &gallery_config)?;

    // Process groups and create thumbnails
    let mut gallery_groups = Vec::new();
    for (group_name, files) in grouped_media {
        let group = process_media_group(
            &group_name,
            files,
            &gallery_source,
            &gallery_output,
            &thumbnails_output,
            &gallery_config,
        )?;
        gallery_groups.push(group);
    }

    // Generate gallery.json
    let gallery_json = serde_json::to_string_pretty(&gallery_groups)?;
    fs::write(gallery_output.join("gallery.json"), gallery_json)?;

    // Generate markdown files for each group
    for group in &gallery_groups {
        generate_gallery_markdown(group, content_folder, &gallery_config)?;
    }

    info!("Generated gallery with {} groups", gallery_groups.len());
    Ok(())
}

fn scan_media_files(
    source_dir: &Path,
    config: &MediaGalleryConfig,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut media_files = Vec::new();

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                if config.extensions.contains(&extension.to_lowercase()) {
                    media_files.push(path.to_path_buf());
                }
            }
        }
    }

    media_files.sort();
    Ok(media_files)
}

fn group_media_files(
    media_files: Vec<PathBuf>,
    config: &MediaGalleryConfig,
) -> Result<HashMap<String, Vec<PathBuf>>, Box<dyn std::error::Error>> {
    let mut groups = HashMap::new();

    for file in media_files {
        let filename = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed");

        // Try to match aggregation pattern
        let group_name = if let Ok(regex) = regex::Regex::new(&config.aggregation_pattern) {
            if let Some(captures) = regex.captures(filename) {
                captures.get(1).map_or(filename, |m| m.as_str()).to_string()
            } else {
                filename.to_string()
            }
        } else {
            filename.to_string()
        };

        groups.entry(group_name).or_insert_with(Vec::new).push(file);
    }

    Ok(groups)
}

fn process_media_group(
    group_name: &str,
    files: Vec<PathBuf>,
    source_dir: &Path,
    output_dir: &Path,
    thumbnails_dir: &Path,
    config: &MediaGalleryConfig,
) -> Result<GalleryGroup, Box<dyn std::error::Error>> {
    let mut items = Vec::new();
    let mut thumbnail_path = None;

    for file in files {
        let relative_path = file.strip_prefix(source_dir)?;
        let output_path = output_dir.join(relative_path);

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy original file
        fs::copy(&file, &output_path)?;

        // Generate thumbnail
        let thumbnail_name = format!("{}.jpg", file.file_stem().unwrap().to_str().unwrap());
        let thumbnail_output = thumbnails_dir.join(&thumbnail_name);

        if let Ok(dimensions) = create_thumbnail(&file, &thumbnail_output, config.thumbnail_size) {
            let file_size = file.metadata()?.len();

            let item = MediaItem {
                filename: file.file_name().unwrap().to_str().unwrap().to_string(),
                path: format!("gallery/{}", relative_path.to_str().unwrap()),
                thumbnail_path: format!("gallery/thumbnails/{}", thumbnail_name),
                title: file.file_stem().unwrap().to_str().unwrap().to_string(),
                description: None,
                width: dimensions.0,
                height: dimensions.1,
                file_size,
            };

            // Use first item's thumbnail as group thumbnail
            if thumbnail_path.is_none() {
                thumbnail_path = Some(item.thumbnail_path.clone());
            }

            items.push(item);
        }
    }

    Ok(GalleryGroup {
        name: group_name.to_string(),
        items,
        thumbnail: thumbnail_path,
    })
}

fn create_thumbnail(
    input_path: &Path,
    output_path: &Path,
    size: u32,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let img = match image::open(input_path) {
        Ok(img) => img,
        Err(e) => {
            warn!("Failed to open image {}: {}", input_path.display(), e);
            return Err(e.into());
        }
    };

    let (width, height) = img.dimensions();

    // Create thumbnail maintaining aspect ratio
    let thumbnail = img.resize(size, size, imageops::FilterType::Lanczos3);

    // Save as JPEG
    match thumbnail.save_with_format(output_path, ImageFormat::Jpeg) {
        Ok(_) => Ok((width, height)),
        Err(e) => {
            warn!("Failed to save thumbnail {}: {}", output_path.display(), e);
            Err(e.into())
        }
    }
}

fn generate_gallery_markdown(
    group: &GalleryGroup,
    content_folder: &Path,
    _config: &MediaGalleryConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown_path = content_folder.join(format!("gallery-{}.md", group.name));

    // Don't overwrite existing markdown files
    if markdown_path.exists() {
        return Ok(());
    }

    let mut markdown_content = String::new();
    markdown_content.push_str("---\n");
    markdown_content.push_str(&format!("title: Gallery - {}\n", group.name));
    markdown_content.push_str("layout: gallery\n");
    markdown_content.push_str("---\n\n");
    markdown_content.push_str(&format!("# Gallery - {}\n\n", group.name));

    for item in &group.items {
        markdown_content.push_str(&format!(
            "![{}]({})\n\n",
            item.title, item.thumbnail_path
        ));
    }

    fs::write(markdown_path, markdown_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_gallery_config_default() {
        let config = MediaGalleryConfig::default();
        assert_eq!(config.source, "gallery");
        assert_eq!(config.thumbnail_size, 300);
        assert!(config.extensions.contains(&"jpg".to_string()));
    }

    #[test]
    fn test_get_media_gallery_config_none() {
        let config = Marmite::new();
        assert!(get_media_gallery_config(&config).is_none());
    }

    #[test]
    fn test_group_media_files() {
        let config = MediaGalleryConfig::default();
        let files = vec![
            PathBuf::from("test_01.jpg"),
            PathBuf::from("test_02.jpg"),
            PathBuf::from("other.png"),
        ];

        let groups = group_media_files(files, &config).unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("test"));
        assert!(groups.contains_key("other"));
    }
}
