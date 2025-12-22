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

### When to Use This Feature

This feature is ideal when:

- You have large images from cameras or design tools that need to be web-optimized
- You want consistent image sizes across your site
- You need to reduce bandwidth usage for visitors
- You want to automate image optimization in your build pipeline

## Configuration Options

Add these settings to the `extra` section of your `marmite.yaml`:

```yaml
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
| `max_image_width` | integer | none | Maximum width for regular images. Images wider than this will be resized. |
| `banner_image_width` | integer | none | Maximum width for banner images. Allows larger sizes for hero images. |
| `resize_filter` | string | `"quality"` | Resampling algorithm. See [Filter Options](#filter-options) below. |

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
extra:
  max_image_width: 800
  resize_filter: "fast"
```

Uses faster algorithm for quicker development builds.

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

### When NOT to Use This Feature

- Images already optimized to target sizes
- SVG graphics (they're vector-based)
- Icons and favicons
- Images that need precise pixel dimensions

## How It Works

During site generation:

1. Marmite scans the output media directory
2. For each supported image format:
   - Checks if it's a banner image (filename or frontmatter)
   - Compares current width to target width
   - Resizes if larger, skips if already smaller
3. Uses atomic file operations (safe even if interrupted)
4. Reports statistics on completion

### Build Output

When image resizing is active, you'll see output like:

```
[INFO] Image resize enabled: max_image_width=800px, banner_image_width=1200px
[INFO] Processing 45 images for resizing...
[INFO] Progress: 20/45 (44%) - 12 resized, 8 unchanged
[INFO] Image processing complete in 3.45s: 28 resized, 17 unchanged, 0 errors
```

## Troubleshooting

### Images Not Being Resized

**Check the configuration:**
```yaml
extra:
  max_image_width: 800  # Must be in 'extra' section
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

- Use `resize_filter: "fast"` for development
- Only enable resizing for production builds
- Consider pre-optimizing large images

### Error Messages

**"Invalid value for 'max_image_width'"**
- Width must be between 1 and 10000 pixels

**"Failed to resize [path]"**
- Check if the file is a valid image
- Ensure file permissions allow reading/writing

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

## Performance Impact

| Images | Filter | Approximate Time |
|--------|--------|------------------|
| 10 | quality | ~2 seconds |
| 100 | quality | ~20 seconds |
| 100 | fast | ~8 seconds |
| 1000 | fast | ~80 seconds |

Times vary based on image sizes and hardware.

---

The image optimization feature helps ensure your Marmite site loads quickly while maintaining visual quality. Configure it once, and every build automatically optimizes your images.
