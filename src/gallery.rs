use image::{imageops::FilterType, ImageError};
use log::{error, info};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryItem {
    pub thumb: String,
    pub image: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gallery {
    pub name: String,
    pub files: Vec<GalleryItem>,
    pub cover: String,
    pub ord: GalleryOrder,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GalleryOrder {
    Asc,
    Desc,
}

impl Default for GalleryOrder {
    fn default() -> Self {
        Self::Asc
    }
}

#[derive(Debug, Deserialize)]
struct ImageDescription {
    filename: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct GalleryConfig {
    name: Option<String>,
    ord: Option<GalleryOrder>,
    cover: Option<String>,
    images: Option<Vec<ImageDescription>>,
}

pub fn process_galleries(
    media_path: &Path,
    gallery_path: &str,
    create_thumbnails: bool,
    thumb_size: u32,
) -> HashMap<String, Gallery> {
    let gallery_dir = media_path.join(gallery_path);
    info!("Processing galleries from: {}", gallery_dir.display());

    if !gallery_dir.exists() {
        info!(
            "Gallery directory does not exist: {}",
            gallery_dir.display()
        );
        return HashMap::new();
    }

    let entries: Vec<_> = match fs::read_dir(&gallery_dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect(),
        Err(e) => {
            panic!(
                "Failed to read gallery directory: {}: {e}",
                gallery_dir.display()
            );
        }
    };

    let galleries: HashMap<String, Gallery> = entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }

            let folder_name = path.file_name().and_then(|n| n.to_str())?;
            let gallery = process_single_gallery(&path, folder_name, create_thumbnails, thumb_size);
            info!(
                "Found gallery: {} with {} files",
                folder_name,
                gallery.files.len()
            );
            Some((folder_name.to_string(), gallery))
        })
        .collect();

    info!("Total galleries found: {}", galleries.len());
    galleries
}

fn process_single_gallery(
    gallery_path: &Path,
    folder_name: &str,
    create_thumbnails: bool,
    thumb_size: u32,
) -> Gallery {
    let config_path = gallery_path.join("gallery.yaml");
    let config = Arc::new(load_gallery_config(&config_path));

    // Create thumbnails directory if creating thumbnails
    let thumbnails_dir = gallery_path.join("thumbnails");
    if create_thumbnails && !thumbnails_dir.exists() {
        if let Err(e) = fs::create_dir(&thumbnails_dir) {
            error!("Failed to create thumbnails directory: {e}");
        }
    }

    let image_entries: Vec<_> = WalkDir::new(gallery_path)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_image_file(e.path()))
        .collect();

    let config_clone = Arc::clone(&config);
    let mut files: Vec<GalleryItem> = image_entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str())?;

            // Skip thumbnails directory
            if path.parent()? == thumbnails_dir {
                return None;
            }

            let thumb_name = if create_thumbnails {
                generate_thumbnail(path, &thumbnails_dir, thumb_size)
                    .unwrap_or_else(|| filename.to_string())
            } else {
                filename.to_string()
            };

            Some(GalleryItem {
                thumb: format!("thumbnails/{thumb_name}"),
                image: filename.to_string(),
                description: get_description(filename, &config_clone),
            })
        })
        .collect();

    let ord = config.ord.unwrap_or_default();

    match ord {
        GalleryOrder::Asc => files.sort_by(|a, b| a.image.cmp(&b.image)),
        GalleryOrder::Desc => files.sort_by(|a, b| b.image.cmp(&a.image)),
    }

    let cover = config
        .cover
        .clone()
        .or_else(|| files.first().map(|item| item.image.clone()))
        .unwrap_or_default();

    Gallery {
        name: config
            .name
            .clone()
            .unwrap_or_else(|| folder_name.to_string()),
        files,
        cover,
        ord,
    }
}

fn load_gallery_config(config_path: &Path) -> GalleryConfig {
    if !config_path.exists() {
        return GalleryConfig {
            name: None,
            ord: None,
            cover: None,
            images: None,
        };
    }

    match fs::read_to_string(config_path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
            error!("Failed to parse gallery config: {e}");
            GalleryConfig {
                name: None,
                ord: None,
                cover: None,
                images: None,
            }
        }),
        Err(e) => {
            error!(
                "Failed to read gallery config file {}: {}",
                config_path.display(),
                e
            );
            GalleryConfig {
                name: None,
                ord: None,
                cover: None,
                images: None,
            }
        }
    }
}

fn get_description(filename: &str, config: &GalleryConfig) -> Option<String> {
    if let Some(images) = &config.images {
        for image_desc in images {
            // 1. Check for exact match first (case insensitive)
            if image_desc.filename.to_lowercase() == filename.to_lowercase() {
                return Some(image_desc.description.clone());
            }

            // 2. Convert "*" to ".*" for regex catch-all, then try regex match
            let pattern_str = if image_desc.filename == "*" {
                ".*"
            } else {
                &image_desc.filename
            };

            // Try regex match (always case insensitive)
            let pattern = format!("(?i){pattern_str}");
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(filename) {
                    return Some(image_desc.description.clone());
                }
            }
        }
    }
    None
}

fn is_image_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "tiff"
    )
}

fn generate_thumbnail(image_path: &Path, thumbnails_dir: &Path, size: u32) -> Option<String> {
    let filename = image_path.file_name().and_then(|n| n.to_str())?;

    let thumb_path = thumbnails_dir.join(filename);

    if thumb_path.exists() {
        return Some(filename.to_string());
    }

    match create_thumbnail(image_path, &thumb_path, size) {
        Ok(()) => {
            info!("Created thumbnail: {}", thumb_path.display());
            Some(filename.to_string())
        }
        Err(e) => {
            error!(
                "Failed to create thumbnail for {}: {}",
                image_path.display(),
                e
            );
            None
        }
    }
}

fn create_thumbnail(input_path: &Path, output_path: &Path, size: u32) -> Result<(), ImageError> {
    let img = image::open(input_path)?;
    let thumbnail = img.resize(size, size, FilterType::Nearest);
    thumbnail.save(output_path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn copy_galleries(input_media: &Path, output_media: &Path, gallery_path: &str) {
    let input_gallery = input_media.join(gallery_path);

    if !input_gallery.exists() {
        return;
    }

    if let Err(e) = fs_extra::dir::copy(
        &input_gallery,
        output_media,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    ) {
        error!("Failed to copy gallery directory: {e}");
    }
}

#[cfg(test)]
#[path = "tests/gallery.rs"]
mod tests;
