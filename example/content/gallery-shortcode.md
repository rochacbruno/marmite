---
title: Image Gallery
description: Learn how to use the gallery shortcode to display image galleries in your content
tags: [shortcodes, gallery, images, docs]
date: 2025-08-03
---

# Image Gallery

> [!IMPORTANT]
> This is a **Beta** feature currently available only on the main branch. It has not been released in a stable version yet.

Marmite now includes a powerful gallery shortcode that allows you to create beautiful image galleries in your posts and pages.

## How to Use

### 1. Create a Gallery Folder

First, create a subfolder in your `media/gallery` directory. For example:

```
content/media/gallery/summer2025/
```

### 2. Add Images

Place your images in the gallery folder. Supported formats include:
- JPG/JPEG
- PNG
- WebP
- GIF
- BMP
- TIFF

### 3. Optional: Gallery Configuration

You can create a `gallery.yaml` file in your gallery folder to customize the display:

```yaml
name: "Summer 2025 Vacation"  # Display name for the gallery
ord: asc                      # Sort order: asc or desc
cover: "sunset.jpg"          # Cover image (defaults to first image)
```

### 4. Use the Gallery Shortcode

In your markdown content, use the gallery shortcode:

```html
<!-- .gallery path=summer2025 -->
```

## Shortcode Parameters

The gallery shortcode accepts several optional parameters:

- `path` (required): The folder name of your gallery
- `ord`: Override sort order (`asc` or `desc`)
- `width`: Width of the main image panel (default: 600)
- `height`: Height of the main image panel (default: 600)
- `name`: Override the gallery name
- `cover`: Override the cover image

### Examples

Basic usage:
```html
<!-- .gallery path=summer2025 -->
```

With custom dimensions:
```html
<!-- .gallery path=summer2025 width=800 height=600 -->
```

Override sort order and name:
```html
<!-- .gallery path=summer2025 ord=desc name="My Amazing Summer" -->
```

## Features

### Automatic Thumbnail Generation

When `gallery_create_thumbnails` is set to `true` in your config (default), Marmite automatically generates 50x50 pixel thumbnails for all images in your galleries. This ensures fast loading times for the thumbnail strip.

### Interactive Gallery Interface

The gallery includes:
- A main image panel showing the selected image
- A thumbnail strip at the bottom for navigation
- Click on thumbnails to change the main image
- Click on the main image to view it full-screen
- Navigation arrows to scroll through thumbnails
- Responsive design that works on all devices

### Configuration Options

In your `marmite.yaml`, you can configure gallery behavior:

```yaml
gallery_path: "gallery"           # Path relative to media folder (default: "gallery")
gallery_create_thumbnails: true   # Auto-generate thumbnails (default: true)
gallery_thumb_size: 50           # Thumbnail size in pixels (default: 50)
```

## Example Gallery

Here's a live example of the gallery shortcode in action:

<!-- .gallery path=summer2025 -->

## Tips

1. **Image Optimization**: While Marmite generates thumbnails automatically, consider optimizing your full-size images before uploading for better performance.

2. **Naming Convention**: Use descriptive file names for your images. They will be used as alt text for accessibility.

3. **Gallery Organization**: Create multiple galleries for different events or categories by creating separate folders in `media/gallery/`.

4. **Cover Images**: Choose an eye-catching cover image that represents your gallery well, as it will be shown by default when the gallery loads.

## Styling

The gallery uses the CSS class `shortcode-gallery` which can be styled in your theme. The default styling provides a clean, functional interface that works well with most themes.