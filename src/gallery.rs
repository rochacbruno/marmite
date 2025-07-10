use crate::config::Marmite;
use crate::site::Data;
use image::{imageops, GenericImageView, ImageFormat};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

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

    // Scan for gallery subfolders and process each one
    let gallery_groups = scan_gallery_folders(&gallery_source, &gallery_config)?;

    // Process each gallery folder
    let mut processed_groups = Vec::new();
    for (folder_name, folder_path) in gallery_groups {
        let group = process_gallery_folder(
            &folder_name,
            &folder_path,
            &gallery_source,
            &gallery_output,
            &thumbnails_output,
            &gallery_config,
            content_folder,
        )?;
        processed_groups.push(group);
    }

    // Generate gallery.json
    let gallery_json = serde_json::to_string_pretty(&processed_groups)?;
    fs::write(gallery_output.join("gallery.json"), gallery_json)?;

    info!("Generated gallery with {} folders", processed_groups.len());
    Ok(())
}

fn scan_gallery_folders(
    source_dir: &Path,
    _config: &MediaGalleryConfig,
) -> Result<Vec<(String, PathBuf)>, Box<dyn std::error::Error>> {
    let mut gallery_folders = Vec::new();

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|s| s.to_str()) {
                gallery_folders.push((folder_name.to_string(), path));
            }
        }
    }

    gallery_folders.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(gallery_folders)
}

fn process_gallery_folder(
    folder_name: &str,
    folder_path: &Path,
    gallery_source: &Path,
    gallery_output: &Path,
    thumbnails_output: &Path,
    config: &MediaGalleryConfig,
    content_folder: &Path,
) -> Result<GalleryGroup, Box<dyn std::error::Error>> {
    // Scan for media files in this folder
    let media_files = scan_media_files(folder_path, config)?;

    // Process media files
    let mut items = Vec::new();
    let mut thumbnail_path = None;

    for file in media_files {
        let relative_path = file.strip_prefix(gallery_source)?;
        let output_path = gallery_output.join(relative_path);

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy original file
        fs::copy(&file, &output_path)?;

        // Generate thumbnail
        let thumbnail_name = format!("{}.jpg", file.file_stem().unwrap().to_str().unwrap());
        let thumbnail_output = thumbnails_output.join(&thumbnail_name);

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

    // Generate gallery markdown page for this folder
    generate_folder_gallery_markdown(folder_name, &items, content_folder)?;

    // Generate individual image pages if needed
    for item in &items {
        generate_image_page(folder_name, item, content_folder)?;
    }

    Ok(GalleryGroup {
        name: folder_name.to_string(),
        items,
        thumbnail: thumbnail_path,
    })
}

fn scan_media_files(
    source_dir: &Path,
    config: &MediaGalleryConfig,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut media_files = Vec::new();

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                if config.extensions.contains(&extension.to_lowercase()) {
                    media_files.push(path);
                }
            }
        }
    }

    media_files.sort();
    Ok(media_files)
}

fn generate_folder_gallery_markdown(
    folder_name: &str,
    items: &[MediaItem],
    content_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown_path = content_folder.join(format!("{}-gallery.md", folder_name));

    // Don't overwrite existing markdown files
    if markdown_path.exists() {
        return Ok(());
    }

    let mut markdown_content = String::new();
    markdown_content.push_str("---\n");
    markdown_content.push_str(&format!("title: {} Gallery\n", folder_name));
    markdown_content.push_str(&format!("slug: {}-gallery\n", folder_name));
    markdown_content.push_str("---\n\n");
    markdown_content.push_str(&format!("# {} Gallery\n\n", folder_name));

    for item in items {
        markdown_content.push_str(&format!(
            "[![{}]({})]({}-gallery-{}.html)\n\n",
            item.title, item.thumbnail_path, folder_name, item.title
        ));
    }

    fs::write(markdown_path, markdown_content)?;
    Ok(())
}

fn generate_image_page(
    folder_name: &str,
    item: &MediaItem,
    content_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_md_path = content_folder.join(format!("{}.md", item.title));

    // If there's a custom .md file for this image, don't generate a default one
    if image_md_path.exists() {
        return Ok(());
    }

    let page_path = content_folder.join(format!("{}-gallery-{}.md", folder_name, item.title));

    // Don't overwrite existing files
    if page_path.exists() {
        return Ok(());
    }

    let mut markdown_content = String::new();
    markdown_content.push_str("---\n");
    markdown_content.push_str(&format!("title: {} - {}\n", folder_name, item.title));
    markdown_content.push_str(&format!("slug: {}-gallery-{}\n", folder_name, item.title));
    markdown_content.push_str("---\n\n");
    markdown_content.push_str(&format!("# {} - {}\n\n", folder_name, item.title));
    markdown_content.push_str(&format!("![{}]({})\n\n", item.title, item.path));

    if let Some(desc) = &item.description {
        markdown_content.push_str(&format!("{}\n\n", desc));
    }

    markdown_content.push_str(&format!("**Dimensions:** {}x{}\n", item.width, item.height));
    markdown_content.push_str(&format!("**File Size:** {} bytes\n", item.file_size));

    fs::write(page_path, markdown_content)?;
    Ok(())
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
}
