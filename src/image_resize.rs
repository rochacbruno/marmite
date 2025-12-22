use image::{imageops::FilterType, GenericImageView, ImageError};
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Marmite;

/// Minimum allowed image width for resize configuration (in pixels)
const MIN_IMAGE_WIDTH: u32 = 1;
/// Maximum allowed image width for resize configuration (in pixels)
const MAX_IMAGE_WIDTH: u32 = 10000;

/// Validate that a width value is within acceptable limits
fn validate_width(value: u32, config_key: &str) -> Option<u32> {
    if (MIN_IMAGE_WIDTH..=MAX_IMAGE_WIDTH).contains(&value) {
        Some(value)
    } else {
        warn!(
            "Invalid value for '{config_key}': {value} (must be between {MIN_IMAGE_WIDTH} and {MAX_IMAGE_WIDTH} pixels). Setting ignored."
        );
        None
    }
}

/// Get image resize settings from config.extra
fn get_resize_settings(config: &Marmite) -> (Option<u32>, Option<u32>) {
    let Some(extra) = &config.extra else {
        return (None, None);
    };

    let banner_width = extra
        .get("banner_image_width")
        .and_then(serde_yaml::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .and_then(|v| validate_width(v, "banner_image_width"));

    let max_width = extra
        .get("max_image_width")
        .and_then(serde_yaml::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .and_then(|v| validate_width(v, "max_image_width"));

    // Log configuration when at least one setting is enabled
    match (banner_width, max_width) {
        (Some(bw), Some(mw)) => {
            info!("Image resize enabled: banner_image_width={bw}px, max_image_width={mw}px");
        }
        (Some(bw), None) => {
            info!("Image resize enabled: banner_image_width={bw}px");
        }
        (None, Some(mw)) => {
            info!("Image resize enabled: max_image_width={mw}px");
        }
        (None, None) => {}
    }

    (banner_width, max_width)
}

/// Check if a file is a resizable raster image based on extension.
///
/// Supported formats:
/// - JPEG (jpg, jpeg)
/// - PNG
/// - WebP
/// - GIF
/// - BMP
/// - TIFF
/// - AVIF (modern format with good compression)
///
/// Note: Vector formats (SVG) and icon formats (ICO) are intentionally excluded
/// because they cannot be meaningfully resized as raster images. Use
/// `is_vector_or_icon_image` to detect these formats.
fn is_image_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "tiff" | "avif"
    )
}

/// Check if a file is a vector or icon image format that should be skipped for resizing.
///
/// These formats are excluded from resizing because:
/// - SVG: Vector format that scales infinitely without quality loss.
///   Rasterizing and resizing would defeat the purpose of using SVG.
/// - ICO: Icon format containing multiple sizes. Resizing would corrupt
///   the multi-resolution structure.
fn is_vector_or_icon_image(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    matches!(ext.to_lowercase().as_str(), "svg" | "ico")
}

/// Check if an image is a banner image based on filename pattern
fn is_banner_image(path: &Path) -> bool {
    let Some(filename) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };

    filename.ends_with(".banner") || filename.contains(".banner.")
}

/// Resize an image to a maximum width, maintaining aspect ratio
/// Only resizes if the image is larger than `max_width`
fn resize_image(input_path: &Path, output_path: &Path, max_width: u32) -> Result<bool, ImageError> {
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();

    // Only resize if image is larger than max_width
    if width <= max_width {
        // Copy original file without modification
        if input_path != output_path {
            fs::copy(input_path, output_path)
                .map_err(|e| ImageError::IoError(std::io::Error::other(e.to_string())))?;
        }
        return Ok(false);
    }

    // Calculate new height maintaining aspect ratio
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let new_height = (f64::from(height) * (f64::from(max_width) / f64::from(width))) as u32;

    let resized = img.resize(max_width, new_height, FilterType::Lanczos3);
    resized.save(output_path)?;

    debug!(
        "Resized {} from {}x{} to {}x{}",
        input_path.display(),
        width,
        height,
        max_width,
        new_height
    );

    Ok(true)
}

/// Normalize a banner image path by removing leading media directory prefixes.
/// Handles cross-platform paths (both forward slashes and backslashes) and
/// various relative path formats.
///
/// # Examples
/// - `"media/file.jpg"` -> `"file.jpg"`
/// - `"./media/file.jpg"` -> `"file.jpg"`
/// - `"media\\file.jpg"` -> `"file.jpg"` (Windows)
/// - `"../media/file.jpg"` -> `"file.jpg"`
/// - `"file.jpg"` -> `"file.jpg"` (unchanged)
/// - `"subdir/file.jpg"` -> `"subdir/file.jpg"` (no media prefix)
fn normalize_banner_path(path_str: &str) -> String {
    // Normalize Windows backslashes to forward slashes for consistent handling
    let normalized_str = path_str.replace('\\', "/");
    let path = Path::new(&normalized_str);

    let components: Vec<Component> = path.components().collect();

    // Find the index after which we should keep components
    // Skip CurDir (.), ParentDir (..), and "media" directory
    let mut start_idx = 0;
    for (i, component) in components.iter().enumerate() {
        match component {
            Component::CurDir | Component::ParentDir => {
                start_idx = i + 1;
            }
            Component::Normal(name) if name.to_string_lossy().eq_ignore_ascii_case("media") => {
                start_idx = i + 1;
                break; // Stop after finding media directory
            }
            _ => break, // Stop at first normal component that isn't "media"
        }
    }

    // Rebuild the path from remaining components
    if start_idx >= components.len() {
        // Edge case: path was only "media" or similar
        return String::new();
    }

    let remaining: PathBuf = components[start_idx..].iter().collect();
    // Convert back to string with forward slashes for consistent storage
    remaining.to_string_lossy().replace('\\', "/")
}

/// Collect all banner image paths from content frontmatter
pub fn collect_banner_paths_from_content(
    posts: &[crate::content::Content],
    pages: &[crate::content::Content],
) -> HashSet<String> {
    let mut banner_paths = HashSet::new();

    for content in posts.iter().chain(pages.iter()) {
        if let Some(banner) = &content.banner_image {
            let normalized = normalize_banner_path(banner);
            if !normalized.is_empty() {
                banner_paths.insert(normalized);
            }
        }
    }

    banner_paths
}

/// Process and resize images in the media directory
/// This function should be called after copying media to output
pub fn process_media_images(
    output_media_path: &Path,
    config: &Marmite,
    banner_paths: &HashSet<String>,
) {
    let (banner_width, max_width) = get_resize_settings(config);

    // If neither option is set, do nothing
    if banner_width.is_none() && max_width.is_none() {
        return;
    }

    info!("Processing images for resizing...");

    let mut resized_count = 0;
    let mut skipped_count = 0;

    for entry in WalkDir::new(output_media_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip thumbnails directory
        if path
            .components()
            .any(|c| c.as_os_str() == "thumbnails" || c.as_os_str() == "_resized")
        {
            continue;
        }

        // Skip vector and icon formats with debug logging
        if is_vector_or_icon_image(path) {
            debug!(
                "Skipped vector/icon image (not resizable): {}",
                path.display()
            );
            continue;
        }

        // Skip non-image files
        if !is_image_file(path) {
            continue;
        }

        // Determine if this is a banner image
        let relative_path = path
            .strip_prefix(output_media_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let is_banner = is_banner_image(path) || banner_paths.contains(&relative_path);

        // Determine target width
        let target_width = if is_banner { banner_width } else { max_width };

        if let Some(width) = target_width {
            match resize_image(path, path, width) {
                Ok(true) => {
                    resized_count += 1;
                    info!("Resized: {} (max width: {}px)", path.display(), width);
                }
                Ok(false) => {
                    skipped_count += 1;
                    debug!("Skipped (already smaller): {}", path.display());
                }
                Err(e) => {
                    error!("Failed to resize {}: {}", path.display(), e);
                }
            }
        }
    }

    if resized_count > 0 || skipped_count > 0 {
        info!("Image processing complete: {resized_count} resized, {skipped_count} unchanged");
    }
}

#[cfg(test)]
#[path = "tests/image_resize.rs"]
mod tests;
