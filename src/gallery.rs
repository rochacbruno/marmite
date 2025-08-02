use image::{imageops::FilterType, ImageError};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryItem {
    pub thumb: String,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gallery {
    pub name: String,
    pub files: Vec<GalleryItem>,
    pub cover: String,
    pub ord: GalleryOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
struct GalleryConfig {
    name: Option<String>,
    ord: Option<GalleryOrder>,
    cover: Option<String>,
}

pub fn process_galleries(
    media_path: &Path,
    gallery_path: &str,
    create_thumbnails: bool,
    thumb_size: u32,
) -> HashMap<String, Gallery> {
    let mut galleries = HashMap::new();
    let gallery_dir = media_path.join(gallery_path);

    info!("Processing galleries from: {}", gallery_dir.display());

    if !gallery_dir.exists() {
        info!("Gallery directory does not exist: {}", gallery_dir.display());
        return galleries;
    }

    for entry in fs::read_dir(&gallery_dir).unwrap_or_else(|_| {
        panic!("Failed to read gallery directory: {}", gallery_dir.display())
    }) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let gallery = process_single_gallery(&path, folder_name, create_thumbnails, thumb_size);
        info!("Found gallery: {} with {} files", folder_name, gallery.files.len());
        galleries.insert(folder_name.to_string(), gallery);
    }

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
    let config = load_gallery_config(&config_path);

    let mut files = Vec::new();
    
    // Create thumbnails directory if creating thumbnails
    let thumbnails_dir = gallery_path.join("thumbnails");
    if create_thumbnails && !thumbnails_dir.exists() {
        if let Err(e) = fs::create_dir(&thumbnails_dir) {
            error!("Failed to create thumbnails directory: {}", e);
        }
    }

    for entry in WalkDir::new(gallery_path)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !is_image_file(path) {
            continue;
        }

        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Skip thumbnails directory
        if path.parent() == Some(&thumbnails_dir) {
            continue;
        }

        let thumb_name = if create_thumbnails {
            generate_thumbnail(path, &thumbnails_dir, thumb_size).unwrap_or_else(|| filename.to_string())
        } else {
            filename.to_string()
        };

        files.push(GalleryItem {
            thumb: format!("thumbnails/{}", thumb_name),
            image: filename.to_string(),
        });
    }

    let ord = config.ord.unwrap_or_default();
    
    match ord {
        GalleryOrder::Asc => files.sort_by(|a, b| a.image.cmp(&b.image)),
        GalleryOrder::Desc => files.sort_by(|a, b| b.image.cmp(&a.image)),
    }

    let cover = config
        .cover
        .or_else(|| files.first().map(|item| item.image.clone()))
        .unwrap_or_default();

    Gallery {
        name: config.name.unwrap_or_else(|| folder_name.to_string()),
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
        };
    }

    match fs::read_to_string(config_path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
            error!("Failed to parse gallery config: {}", e);
            GalleryConfig {
                name: None,
                ord: None,
                cover: None,
            }
        }),
        Err(_) => GalleryConfig {
            name: None,
            ord: None,
            cover: None,
        },
    }
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
    let Some(filename) = image_path.file_name().and_then(|n| n.to_str()) else {
        return None;
    };

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
        &output_media,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    ) {
        error!("Failed to copy gallery directory: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gallery_order_default() {
        assert_eq!(GalleryOrder::default(), GalleryOrder::Asc);
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.JPEG")));
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.webp")));
        assert!(is_image_file(Path::new("test.gif")));
        assert!(is_image_file(Path::new("test.bmp")));
        assert!(is_image_file(Path::new("test.tiff")));

        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test")));
        assert!(!is_image_file(Path::new("test.mp4")));
    }

    #[test]
    fn test_load_gallery_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("gallery.yaml");

        let config = load_gallery_config(&config_path);
        assert!(config.name.is_none());
        assert!(config.ord.is_none());
        assert!(config.cover.is_none());
    }

    #[test]
    fn test_load_gallery_config_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("gallery.yaml");

        let yaml_content = r#"
name: "My Amazing Summer"
ord: desc
cover: "sunset.jpg"
"#;
        fs::write(&config_path, yaml_content).unwrap();

        let config = load_gallery_config(&config_path);
        assert_eq!(config.name, Some("My Amazing Summer".to_string()));
        assert_eq!(config.ord, Some(GalleryOrder::Desc));
        assert_eq!(config.cover, Some("sunset.jpg".to_string()));
    }

    #[test]
    fn test_process_galleries_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let media_path = temp_dir.path();
        fs::create_dir(media_path.join("gallery")).unwrap();

        let galleries = process_galleries(media_path, "gallery", false, 50);
        assert!(galleries.is_empty());
    }

    #[test]
    fn test_process_single_gallery_without_images() {
        let temp_dir = TempDir::new().unwrap();
        let gallery_path = temp_dir.path();

        let gallery = process_single_gallery(gallery_path, "test", false, 50);
        assert_eq!(gallery.name, "test");
        assert!(gallery.files.is_empty());
        assert_eq!(gallery.cover, "");
        assert_eq!(gallery.ord, GalleryOrder::Asc);
    }

    #[test]
    fn test_process_single_gallery_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let gallery_path = temp_dir.path();

        let config_content = r#"
name: "Test Gallery"
ord: desc
cover: "main.jpg"
"#;
        fs::write(gallery_path.join("gallery.yaml"), config_content).unwrap();

        let gallery = process_single_gallery(gallery_path, "test", false, 50);
        assert_eq!(gallery.name, "Test Gallery");
        assert_eq!(gallery.ord, GalleryOrder::Desc);
        assert_eq!(gallery.cover, "main.jpg");
    }

    #[test]
    fn test_copy_galleries() {
        let temp_dir = TempDir::new().unwrap();
        let input_media = temp_dir.path().join("input_media");
        let output_media = temp_dir.path().join("output_media");
        let gallery_dir = input_media.join("gallery").join("summer");

        fs::create_dir_all(&gallery_dir).unwrap();
        fs::write(gallery_dir.join("test.jpg"), "fake image data").unwrap();
        fs::create_dir_all(&output_media).unwrap();

        copy_galleries(&input_media, &output_media, "gallery");

        let copied_file = output_media.join("gallery").join("summer").join("test.jpg");
        assert!(copied_file.exists());
    }

    #[test]
    fn test_process_galleries_with_images() {
        use image::{ImageBuffer, Rgb};
        
        let temp_dir = TempDir::new().unwrap();
        let media_path = temp_dir.path();
        let gallery_dir = media_path.join("gallery").join("test-gallery");
        fs::create_dir_all(&gallery_dir).unwrap();

        // Create a small test image
        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(100, 100);
        img.save(gallery_dir.join("test1.jpg")).unwrap();
        img.save(gallery_dir.join("test2.png")).unwrap();

        // Create gallery config
        let config_content = r#"
name: "Test Gallery"
ord: asc
cover: "test1.jpg"
"#;
        fs::write(gallery_dir.join("gallery.yaml"), config_content).unwrap();

        let galleries = process_galleries(media_path, "gallery", true, 50);

        assert_eq!(galleries.len(), 1);
        assert!(galleries.contains_key("test-gallery"));
        
        let gallery = &galleries["test-gallery"];
        assert_eq!(gallery.name, "Test Gallery");
        assert_eq!(gallery.cover, "test1.jpg");
        assert_eq!(gallery.ord, GalleryOrder::Asc);
        assert_eq!(gallery.files.len(), 2);
        
        // Check thumbnails were created in thumbnails directory
        assert!(gallery_dir.join("thumbnails").join("test1.jpg").exists());
        assert!(gallery_dir.join("thumbnails").join("test2.png").exists());
    }

    #[test]
    fn test_generate_thumbnail() {
        use image::{ImageBuffer, Rgb};
        
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.jpg");
        let thumbnails_dir = temp_dir.path().join("thumbnails");
        fs::create_dir(&thumbnails_dir).unwrap();
        
        // Create a test image
        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(200, 200);
        img.save(&image_path).unwrap();
        
        let thumb_name = generate_thumbnail(&image_path, &thumbnails_dir, 50);
        
        assert!(thumb_name.is_some());
        assert_eq!(thumb_name.unwrap(), "test.jpg");
        assert!(thumbnails_dir.join("test.jpg").exists());
    }

    #[test]
    fn test_generate_thumbnail_existing() {
        use image::{ImageBuffer, Rgb};
        
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.jpg");
        let thumb_path = temp_dir.path().join("test.thumb.png");
        
        // Create a test image
        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(200, 200);
        img.save(&image_path).unwrap();
        
        // Create existing thumbnail
        let thumb = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(50, 50);
        thumb.save(&thumb_path).unwrap();
        
        let thumb_name = generate_thumbnail(&image_path, 50);
        
        assert!(thumb_name.is_some());
        assert_eq!(thumb_name.unwrap(), "test.thumb.png");
    }
}