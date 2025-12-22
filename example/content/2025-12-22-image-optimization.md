---
title: "Image Optimization and Resizing"
tags: ["docs", "features", "images", "optimization"]
description: "Learn how to automatically resize images during site generation to optimize page load times and bandwidth"
authors: ["rochacbruno"]
---

# Image Optimization and Resizing

Marmite can automatically resize images during site generation to optimize your site's performance. This feature reduces file sizes and improves page load times without requiring manual image processing.

## Feature Overview

The image resize feature:

- **Automatically resizes** images that exceed a specified maximum width
- **Maintains aspect ratio** - images are never stretched or distorted
- **Preserves originals** in your source directory (only output copies are resized)
- **Supports banner images** with separate size settings for hero/banner images
- **Uses high-quality resampling** with configurable filter algorithms
- **Parallel processing** - uses all CPU cores for faster builds
- **Incremental builds** - skips unchanged images on subsequent builds

### When to Use This Feature

This feature is ideal when:

- You have large images from cameras or design tools that need to be web-optimized
- You want consistent image sizes across your site
- You need to reduce bandwidth usage for visitors
- You want to automate image optimization in your build pipeline

## Configuration Options

Add these settings to your `marmite.yaml`:

```yaml
# Skip image resizing entirely (for fast development builds)
skip_image_resize: false

extra:
  # Maximum width for regular images (in pixels)
  max_image_width: 800

  # Maximum width for banner/hero images (in pixels)
  banner_image_width: 1200

  # Resize filter quality (optional)
  # Options: "fast", "balanced", "quality" (default)
  resize_filter: "quality"
```

### Configuration Details

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `skip_image_resize` | boolean | `false` | Skip all image resizing (top-level config) |
| `max_image_width` | integer | none | Maximum width for regular images (in `extra`) |
| `banner_image_width` | integer | none | Maximum width for banner images (in `extra`) |
| `resize_filter` | string | `"quality"` | Resampling algorithm (in `extra`) |

### CLI Flag

You can also skip image resizing via command line:

```bash
# Skip image resizing for this build
marmite mysite --skip-image-resize

# Useful for quick development iterations
marmite mysite --serve --skip-image-resize
```

### Filter Options

Choose based on your speed vs. quality needs:

| Filter | Algorithm | Speed | Quality | Best For |
|--------|-----------|-------|---------|----------|
| `"fast"` | Triangle | Fastest | Good | Large sites, development builds |
| `"balanced"` | CatmullRom | Medium | Better | Production with many images |
| `"quality"` | Lanczos3 | Slowest | Best | Photography, final production |

## Banner Image Detection

Marmite identifies banner images in two ways:

### 1. Filename Pattern

Images with `.banner.` in the filename are treated as banners:

```
media/
  post.banner.jpg      # Detected as banner
  article.banner.png   # Detected as banner
  hero.banner.webp     # Detected as banner
  regular-image.jpg    # Regular image
```

### 2. Frontmatter Reference

Images referenced in the `banner_image` frontmatter field:

```yaml
---
title: "My Blog Post"
banner_image: media/hero-photo.jpg
---
```

The `hero-photo.jpg` will be resized using `banner_image_width` instead of `max_image_width`.

## Supported Image Formats

The following formats are supported for resizing:

| Format | Extensions | Notes |
|--------|------------|-------|
| JPEG | `.jpg`, `.jpeg` | Most common, lossy compression |
| PNG | `.png` | Lossless, supports transparency |
| WebP | `.webp` | Modern format, excellent compression |
| GIF | `.gif` | Animated GIFs are resized (first frame) |
| BMP | `.bmp` | Uncompressed bitmap |
| TIFF | `.tiff` | High-quality, large files |
| AVIF | `.avif` | Modern format, best compression |

### Formats NOT Resized

These formats are intentionally skipped:

- **SVG** - Vector format that scales infinitely without quality loss
- **ICO** - Icon format with multiple embedded sizes

## Usage Examples

### Basic Setup - Optimize All Images

```yaml
# marmite.yaml
extra:
  max_image_width: 800
```

All images wider than 800px will be resized to 800px width.

### Separate Banner Sizes

```yaml
# marmite.yaml
extra:
  max_image_width: 800
  banner_image_width: 1400
```

- Regular images: max 800px wide
- Banner images: max 1400px wide

### Fast Builds During Development

```yaml
# marmite.yaml
skip_image_resize: true  # Skip resizing entirely
```

Or use the CLI flag:

```bash
marmite mysite --skip-image-resize
```

### Production with Fast Filter

```yaml
# marmite.yaml
extra:
  max_image_width: 800
  resize_filter: "fast"
```

Uses faster algorithm for quicker builds while still resizing.

### High-Quality Photography Site

```yaml
# marmite.yaml
extra:
  max_image_width: 1200
  banner_image_width: 2000
  resize_filter: "quality"
```

Larger sizes with highest quality resampling for photography portfolios.

## Best Practices

### Recommended Sizes

| Use Case | max_image_width | banner_image_width |
|----------|-----------------|-------------------|
| Blog | 800 | 1200 |
| Portfolio | 1200 | 1800 |
| Documentation | 700 | 1000 |
| Photography | 1400 | 2400 |

### Tips for Best Results

1. **Start with high-quality source images** - Resizing can't improve quality
2. **Use appropriate formats** - WebP or AVIF for best compression
3. **Consider retina displays** - 2x your target display size
4. **Test different filter settings** - Balance speed vs. quality for your needs
5. **Use `--skip-image-resize` during development** - Faster iteration cycles

### When NOT to Use This Feature

- Images already optimized to target sizes
- SVG graphics (they're vector-based)
- Icons and favicons
- Images that need precise pixel dimensions

## How It Works

During site generation:

1. Marmite scans the output media directory
2. Loads previous state for incremental builds
3. For each supported image format (in parallel):
   - Checks if it's a banner image (filename or frontmatter)
   - Checks if the image has changed since last build
   - Compares current width to target width
   - Resizes if needed, skips if unchanged or already smaller
4. Uses atomic file operations (safe even if interrupted)
5. Saves state for next build
6. Reports statistics on completion

### Parallel Processing

Image resizing uses all available CPU cores via the rayon library. This significantly speeds up processing for sites with many images.

### Incremental Builds

Marmite tracks processed images in a state file (`.marmite-resize-state.json`). On subsequent builds:

- **Unchanged images** are skipped (shown as "cached")
- **Modified images** are reprocessed
- **New images** are processed
- **Config changes** trigger full reprocessing

This makes rebuilds much faster when only a few images change.

### Build Output

When image resizing is active, you'll see output like:

```
[INFO] Image resize enabled: max_image_width=800px, banner_image_width=1200px
[INFO] Processing 45 images for resizing (parallel)...
[INFO] Progress: 20/45 (44%)
[INFO] Image processing complete in 1.23s: 5 resized, 10 unchanged, 30 cached, 0 errors
```

On subsequent builds with unchanged images:

```
[INFO] Processing 45 images for resizing (parallel)...
[INFO] Image processing complete in 0.15s: 0 resized, 0 unchanged, 45 cached, 0 errors
```

## Troubleshooting

### Images Not Being Resized

**Check the configuration:**
```yaml
extra:
  max_image_width: 800  # Must be in 'extra' section
```

**Check if skipping is enabled:**
```yaml
skip_image_resize: false  # Must be false or omitted
```

**Check image dimensions:**
Images smaller than the target width are skipped (they don't need resizing).

**Check file format:**
Only supported raster formats are resized. SVG and ICO are skipped.

### Images Look Blurry

- Use `resize_filter: "quality"` for best results
- Ensure source images are high resolution
- Consider increasing `max_image_width`

### Build is Slow

- Use `resize_filter: "fast"` for faster processing
- Use `--skip-image-resize` during development
- Incremental builds are automatic - second build will be faster
- Consider pre-optimizing large images before adding to your site

### Cached Images Not Updating

If you modify an image but it's not being reprocessed:

1. Delete the state file: `rm output/media/.marmite-resize-state.json`
2. Rebuild the site

Or change the resize configuration (any change triggers full reprocessing).

### Error Messages

**"Invalid value for 'max_image_width'"**
- Width must be between 1 and 10000 pixels

**"Failed to resize [path]"**
- Check if the file is a valid image
- Ensure file permissions allow reading/writing

**"Image resize configuration changed, reprocessing all images"**
- This is normal when you change `max_image_width`, `banner_image_width`, or `resize_filter`

## Verifying Results

After building, check output image dimensions:

```bash
# Using ImageMagick
identify output/media/*.jpg | head -5

# Using file command
file output/media/*.jpg | head -5
```

Or check file sizes to confirm compression:

```bash
# Compare source vs output
ls -lh content/media/large-photo.jpg
ls -lh output/media/large-photo.jpg
```

Check the state file for processed images:

```bash
cat output/media/.marmite-resize-state.json | jq '.images | keys'
```

## Performance

### Parallel Processing Speedup

| Images | Sequential | Parallel (8 cores) | Speedup |
|--------|------------|-------------------|---------|
| 50 | ~10s | ~2s | 5x |
| 200 | ~40s | ~6s | 6-7x |
| 500 | ~100s | ~15s | 6-7x |

### Incremental Build Performance

| Scenario | Time |
|----------|------|
| First build (100 images) | ~6s |
| Rebuild (no changes) | ~0.2s |
| Rebuild (5 images changed) | ~0.5s |
| Rebuild (config changed) | ~6s |

Times vary based on image sizes and hardware.

---

The image optimization feature helps ensure your Marmite site loads quickly while maintaining visual quality. With parallel processing and incremental builds, it's fast enough for development workflows while thorough enough for production deployments.
