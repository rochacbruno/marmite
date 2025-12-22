use image::{imageops::FilterType, GenericImageView, ImageError};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tempfile::Builder as TempFileBuilder;
use walkdir::WalkDir;

use crate::config::Marmite;

/// Progress reporting interval: log every N images processed
const PROGRESS_INTERVAL: usize = 10;

/// State file name for tracking processed images
const STATE_FILE_NAME: &str = ".marmite-resize-state.json";

/// Minimum allowed image width for resize configuration (in pixels)
const MIN_IMAGE_WIDTH: u32 = 1;
/// Maximum allowed image width for resize configuration (in pixels)
const MAX_IMAGE_WIDTH: u32 = 10000;

/// State entry for a processed image
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageState {
    /// Original file size before processing
    source_size: u64,
    /// Modification time of source file (as unix timestamp)
    source_modified: u64,
    /// Target width used for resizing
    target_width: u32,
    /// Whether the image was actually resized (vs skipped)
    was_resized: bool,
}

/// State tracking for incremental image processing
#[derive(Debug, Default, Serialize, Deserialize)]
struct ResizeState {
    /// Map of relative path -> image state
    images: HashMap<String, ImageState>,
    /// Configuration hash to detect config changes
    config_hash: String,
}

impl ResizeState {
    /// Load state from file, or return empty state if not found
    fn load(output_path: &Path) -> Self {
        let state_file = output_path.join(STATE_FILE_NAME);
        if let Ok(file) = File::open(&state_file) {
            let reader = BufReader::new(file);
            if let Ok(state) = serde_json::from_reader(reader) {
                return state;
            }
        }
        Self::default()
    }

    /// Save state to file
    fn save(&self, output_path: &Path) {
        let state_file = output_path.join(STATE_FILE_NAME);
        if let Ok(file) = File::create(&state_file) {
            let writer = BufWriter::new(file);
            if let Err(e) = serde_json::to_writer(writer, self) {
                warn!("Failed to save resize state: {e}");
            }
        }
    }

    /// Check if an image needs processing
    fn needs_processing(
        &self,
        relative_path: &str,
        source_size: u64,
        source_modified: u64,
        target_width: u32,
        config_hash: &str,
    ) -> bool {
        // If config changed, reprocess everything
        if self.config_hash != config_hash {
            return true;
        }

        // Check if we have state for this image
        if let Some(state) = self.images.get(relative_path) {
            // Skip if source hasn't changed and target width is the same
            state.source_size != source_size
                || state.source_modified != source_modified
                || state.target_width != target_width
        } else {
            // No state means we need to process
            true
        }
    }

    /// Update state for a processed image
    fn update(
        &mut self,
        relative_path: String,
        source_size: u64,
        source_modified: u64,
        target_width: u32,
        was_resized: bool,
    ) {
        self.images.insert(
            relative_path,
            ImageState {
                source_size,
                source_modified,
                target_width,
                was_resized,
            },
        );
    }
}

/// Generate a hash of the resize configuration for change detection
fn config_hash(settings: &ResizeSettings) -> String {
    format!(
        "bw:{:?},mw:{:?},f:{:?}",
        settings.banner_width, settings.max_width, settings.filter
    )
}

/// Get file metadata for state tracking
fn get_file_metadata(path: &Path) -> Option<(u64, u64)> {
    fs::metadata(path).ok().map(|m| {
        let size = m.len();
        let modified = m
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs());
        (size, modified)
    })
}

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

/// Parse the resize filter configuration value.
///
/// Supported values:
/// - `"fast"`: Triangle filter - fastest, lower quality
/// - `"balanced"`: `CatmullRom` filter - good balance of speed and quality
/// - `"quality"`: Lanczos3 filter - highest quality, slowest (default)
///
/// Returns `Lanczos3` if the value is invalid or not specified.
fn parse_resize_filter(value: Option<&str>) -> FilterType {
    match value {
        Some("fast") => {
            info!("Using 'fast' resize filter (Triangle)");
            FilterType::Triangle
        }
        Some("balanced") => {
            info!("Using 'balanced' resize filter (CatmullRom)");
            FilterType::CatmullRom
        }
        Some("quality") => {
            info!("Using 'quality' resize filter (Lanczos3)");
            FilterType::Lanczos3
        }
        Some(invalid) => {
            warn!(
                "Invalid resize_filter value '{invalid}'. Valid options: 'fast', 'balanced', 'quality'. Using default 'quality'."
            );
            FilterType::Lanczos3
        }
        None => FilterType::Lanczos3, // Default, no logging needed
    }
}

/// Image resize settings parsed from configuration.
#[derive(Debug, Clone)]
struct ResizeSettings {
    banner_width: Option<u32>,
    max_width: Option<u32>,
    filter: FilterType,
}

/// Get image resize settings from config.extra
fn get_resize_settings(config: &Marmite) -> ResizeSettings {
    let Some(extra) = &config.extra else {
        return ResizeSettings {
            banner_width: None,
            max_width: None,
            filter: FilterType::Lanczos3,
        };
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

    let filter = extra
        .get("resize_filter")
        .and_then(serde_yaml::Value::as_str);
    let filter = parse_resize_filter(filter);

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

    ResizeSettings {
        banner_width,
        max_width,
        filter,
    }
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

/// Resize an image to a maximum width, maintaining aspect ratio.
/// Only resizes if the image is larger than `max_width`.
///
/// Uses atomic file operations: the resized image is first written to a
/// temporary file in the same directory, then renamed to the target path.
/// This ensures the original image is not corrupted if resizing fails.
///
/// # Arguments
/// * `input_path` - Path to the source image
/// * `output_path` - Path where the resized image will be saved
/// * `max_width` - Maximum width in pixels
/// * `filter` - The resampling filter to use for resizing
fn resize_image(
    input_path: &Path,
    output_path: &Path,
    max_width: u32,
    filter: FilterType,
) -> Result<bool, ImageError> {
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

    let resized = img.resize(max_width, new_height, filter);

    // Use atomic write: save to temp file first, then rename
    // Temp file is created in the same directory to ensure atomic rename works
    // (rename across filesystems is not atomic)
    let output_dir = output_path.parent().unwrap_or(Path::new("."));

    // Preserve file extension so image library can determine output format
    let extension = output_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    let temp_file = TempFileBuilder::new()
        .suffix(&extension)
        .tempfile_in(output_dir)
        .map_err(|e| ImageError::IoError(std::io::Error::other(e.to_string())))?;

    // Save to temp file
    resized.save(temp_file.path())?;

    // Atomically rename temp file to final destination
    // persist() consumes the NamedTempFile and renames it, preventing automatic cleanup
    temp_file
        .persist(output_path)
        .map_err(|e| ImageError::IoError(std::io::Error::other(e.to_string())))?;

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

/// Collect all eligible image paths for resizing, filtering out non-images and special directories.
fn collect_image_paths(output_media_path: &Path) -> Vec<PathBuf> {
    WalkDir::new(output_media_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();

            // Skip thumbnails directory
            if path
                .components()
                .any(|c| c.as_os_str() == "thumbnails" || c.as_os_str() == "_resized")
            {
                return None;
            }

            // Skip vector and icon formats with debug logging
            if is_vector_or_icon_image(path) {
                debug!(
                    "Skipped vector/icon image (not resizable): {}",
                    path.display()
                );
                return None;
            }

            // Skip non-image files
            if !is_image_file(path) {
                return None;
            }

            Some(path.to_path_buf())
        })
        .collect()
}

/// Result of processing a single image
#[derive(Debug)]
struct ProcessResult {
    relative_path: String,
    source_size: u64,
    source_modified: u64,
    target_width: u32,
    result: ProcessOutcome,
}

#[derive(Debug)]
enum ProcessOutcome {
    Resized,
    Skipped,
    Cached,
    Error,
    NoTarget,
}

/// Process a single image and return the result
fn process_single_image(
    path: &Path,
    output_media_path: &Path,
    banner_paths: &HashSet<String>,
    settings: &ResizeSettings,
    state: &ResizeState,
    cfg_hash: &str,
) -> ProcessResult {
    // Determine relative path for state tracking
    let relative_path = path
        .strip_prefix(output_media_path)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    // Get file metadata for change detection
    let (source_size, source_modified) = get_file_metadata(path).unwrap_or((0, 0));

    // Determine if this is a banner image
    let is_banner = is_banner_image(path) || banner_paths.contains(&relative_path);

    // Determine target width
    let target_width = if is_banner {
        settings.banner_width
    } else {
        settings.max_width
    };

    if let Some(width) = target_width {
        // Check if we can skip based on state
        if !state.needs_processing(
            &relative_path,
            source_size,
            source_modified,
            width,
            cfg_hash,
        ) {
            debug!("Cached (unchanged): {}", path.display());
            return ProcessResult {
                relative_path,
                source_size,
                source_modified,
                target_width: width,
                result: ProcessOutcome::Cached,
            };
        }

        // Process the image
        match resize_image(path, path, width, settings.filter) {
            Ok(true) => {
                debug!("Resized: {} (max width: {width}px)", path.display());
                ProcessResult {
                    relative_path,
                    source_size,
                    source_modified,
                    target_width: width,
                    result: ProcessOutcome::Resized,
                }
            }
            Ok(false) => {
                debug!("Skipped (already smaller): {}", path.display());
                ProcessResult {
                    relative_path,
                    source_size,
                    source_modified,
                    target_width: width,
                    result: ProcessOutcome::Skipped,
                }
            }
            Err(e) => {
                error!("Failed to resize {}: {e}", path.display());
                ProcessResult {
                    relative_path,
                    source_size,
                    source_modified,
                    target_width: width,
                    result: ProcessOutcome::Error,
                }
            }
        }
    } else {
        ProcessResult {
            relative_path,
            source_size,
            source_modified,
            target_width: 0,
            result: ProcessOutcome::NoTarget,
        }
    }
}

/// Aggregate processing results and update state
fn aggregate_results(
    results: Vec<ProcessResult>,
    state: &ResizeState,
    cfg_hash: String,
) -> (ResizeState, usize, usize, usize, usize) {
    let mut new_state = ResizeState {
        images: HashMap::new(),
        config_hash: cfg_hash,
    };

    let mut resized_count = 0;
    let mut skipped_count = 0;
    let mut cached_count = 0;
    let mut error_count = 0;

    for result in results {
        match result.result {
            ProcessOutcome::Resized => {
                resized_count += 1;
                new_state.update(
                    result.relative_path,
                    result.source_size,
                    result.source_modified,
                    result.target_width,
                    true,
                );
            }
            ProcessOutcome::Skipped => {
                skipped_count += 1;
                new_state.update(
                    result.relative_path,
                    result.source_size,
                    result.source_modified,
                    result.target_width,
                    false,
                );
            }
            ProcessOutcome::Cached => {
                cached_count += 1;
                // Preserve existing state
                if let Some(existing) = state.images.get(&result.relative_path) {
                    new_state
                        .images
                        .insert(result.relative_path, existing.clone());
                }
            }
            ProcessOutcome::Error => {
                error_count += 1;
                // Don't save state for errors so they're retried
            }
            ProcessOutcome::NoTarget => {
                skipped_count += 1;
            }
        }
    }

    (
        new_state,
        resized_count,
        skipped_count,
        cached_count,
        error_count,
    )
}

/// Process and resize images in the media directory.
/// This function should be called after copying media to output.
///
/// Features:
/// - Parallel processing using rayon for faster builds
/// - Incremental builds: tracks processed images to skip unchanged files
/// - Progress reporting for large image collections
pub fn process_media_images(
    output_media_path: &Path,
    config: &Marmite,
    banner_paths: &HashSet<String>,
) {
    let settings = get_resize_settings(config);

    // If neither option is set, do nothing
    if settings.banner_width.is_none() && settings.max_width.is_none() {
        return;
    }

    // Load previous state for incremental builds
    let state = ResizeState::load(output_media_path);
    let cfg_hash = config_hash(&settings);

    // Check if config changed
    if !state.config_hash.is_empty() && state.config_hash != cfg_hash {
        info!("Image resize configuration changed, reprocessing all images");
    }

    // Collect all image paths first for progress reporting
    let image_paths = collect_image_paths(output_media_path);
    let total_images = image_paths.len();

    if total_images == 0 {
        debug!("No images found to process");
        return;
    }

    info!("Processing {total_images} images for resizing (parallel)...");

    let start_time = Instant::now();

    // Atomic counters for thread-safe progress tracking
    let processed_count = AtomicUsize::new(0);
    let progress_step = (total_images / 10).max(PROGRESS_INTERVAL);

    // Process images in parallel using helper function
    let results: Vec<ProcessResult> = image_paths
        .par_iter()
        .map(|path| {
            let result = process_single_image(
                path,
                output_media_path,
                banner_paths,
                &settings,
                &state,
                &cfg_hash,
            );

            // Update progress counter
            let count = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count % progress_step == 0 && count < total_images {
                let percent = (count * 100) / total_images;
                info!("Progress: {count}/{total_images} ({percent}%)");
            }

            result
        })
        .collect();

    // Aggregate results and update state using helper function
    let (new_state, resized_count, skipped_count, cached_count, error_count) =
        aggregate_results(results, &state, cfg_hash);

    // Save state for incremental builds
    new_state.save(output_media_path);

    // Report final statistics with elapsed time
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();

    let total_processed = resized_count + skipped_count + cached_count + error_count;
    if total_processed > 0 {
        if cached_count > 0 {
            info!(
                "Image processing complete in {elapsed_secs:.2}s: {resized_count} resized, {skipped_count} unchanged, {cached_count} cached, {error_count} errors"
            );
        } else {
            info!(
                "Image processing complete in {elapsed_secs:.2}s: {resized_count} resized, {skipped_count} unchanged, {error_count} errors"
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/image_resize.rs"]
mod tests;
