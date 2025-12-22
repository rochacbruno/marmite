---
title: Marmite 0.2.7 Release Notes
slug: marmite-0-2-7-release-notes
description: "Marmite 0.2.7 introduces shortcodes, image galleries, automatic image resizing with parallel processing, live reload for development, sitemap generation, Python package, and many improvements. A major update packed with new features!"
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
pinned: true
---

We're excited to announce Marmite 0.2.7, a major release packed with powerful new features, improvements, and bug fixes. This release introduces shortcodes, image galleries, automatic image resizing with parallel processing, live reload for development, sitemap generation, Python package support, and much more.

## ⚠️ Breaking Changes

### Greentext Disabled by Default

The `greentext` markdown extension is now disabled by default. Previously, lines starting with `>` followed by text would be rendered with green styling (4chan-style greentext).

To restore the previous behavior, explicitly enable it in your configuration:

```yaml
# In marmite.yaml
markdown_parser:
  extension:
    greentext: true
```

## 🎉 New Features

### Automatic Image Resizing

Marmite can now automatically resize images during site generation to optimize performance. [[Image Optimization and Resizing|2025-12-22-image-optimization]]

- **Automatic resizing** of images exceeding a specified maximum width
- **Parallel processing** using all CPU cores for faster builds
- **Incremental builds** - skips unchanged images on subsequent builds
- **Separate banner sizes** - different max widths for hero/banner images
- **High-quality resampling** with configurable filter algorithms (fast, balanced, quality)
- **Atomic file operations** for reliability (safe even if interrupted)
- **Progress reporting** for large image collections
- **Supports AVIF, WebP, JPEG, PNG, GIF, BMP, TIFF** formats

```yaml
# In marmite.yaml
skip_image_resize: false

extra:
  max_image_width: 800        # Maximum width for regular images
  banner_image_width: 1200    # Maximum width for banner images
  resize_filter: "quality"    # Options: fast, balanced, quality
```

```bash
# Skip resizing for faster development builds
marmite mysite --skip-image-resize
```

### Shortcodes

Marmite now supports shortcodes - reusable content snippets that can be embedded in your markdown files. [[Shortcodes Guide|2025-08-01-shortcodes-guide]]

- Create custom shortcodes in the `shortcodes/` directory
- Use HTML or Markdown for shortcode templates
- Pass parameters to shortcodes
- Built-in shortcodes for common use cases

```markdown
<!-- Embed a YouTube video -->
<!-- .youtube id=dQw4w9WgXcQ -->

<!-- Display table of contents -->
<!-- .toc -->

<!-- List all tags -->
<!-- .tags -->

<!-- Embed Spotify track -->
<!-- .spotify id=TRACK_ID -->
```

```yaml
# In marmite.yaml
enable_shortcodes: true
```

### Image Gallery

Create beautiful image galleries with automatic thumbnail generation. [[Gallery Shortcode|gallery-shortcode]]

- Organize images in folders under `media/gallery/`
- Automatic thumbnail creation with configurable sizes
- Gallery metadata via `gallery.yaml` files
- Responsive lightbox viewing
- Caption support from image metadata

```yaml
# In marmite.yaml
gallery_path: media/gallery
gallery_create_thumbnails: true
gallery_thumb_size: 300
```

```markdown
<!-- In your content -->
<!-- .gallery folder=summer2025 -->
```

### Live Reload

Automatically refresh your browser when files change during development. (#370)

- WebSocket-based live reload
- Works with `--watch --serve` mode
- No browser extensions required
- Instant feedback during content editing

```bash
marmite myblog --watch --serve
# Edit files and see changes instantly in your browser!
```

### Sitemap Generation

Automatic XML sitemap generation for better SEO. [[Automatic Sitemap Generation|2025-08-01-automatic-sitemap-generation]]

- Enabled by default with `build_sitemap: true`
- Includes all posts, pages, tags, and archives
- Proper lastmod dates and priorities
- JSON URL index with `publish_urls_json: true`

### Show URLs Command

Preview all site URLs without building. [[Show URLs Command|2025-08-01-show-urls-dry-run-command]]

```bash
marmite myblog --show-urls
```

Displays all URLs organized by content type - useful for debugging and verification.

### File Mappings

Copy or process files from arbitrary locations into your site. [[File Mapping Feature|2025-08-01-file-mapping-feature]]

```yaml
# In marmite.yaml
file_mapping:
  - source: ../shared/images
    destination: media/shared
  - source: ../data/config.json
    destination: data/config.json
```

### Python Package

Install and run Marmite via pip! [[Python Installation|2025-08-06-03-35-05-python-installation-with-pip]]

```bash
pip install marmite-ssg
marmite myblog --serve
```

Available on PyPI with pre-built binaries for major platforms.

### Install Script

Easy installation with a single command. [[Installation Guide|2025-08-02-18-02-50-installation]]

```bash
curl -sSL https://marmite.blog/install.sh | bash
```

Cross-platform script that detects your OS and architecture.

### Preferred Color Scheme

Configure the default theme appearance. (#363)

```yaml
# In marmite.yaml
extra:
  colormode: auto  # auto, dark, or light
  colormodetoggle: true  # show toggle button
```

User preferences are saved to local storage and persist across visits.

### Fallback Server Address

The development server now tries alternative ports if the default is in use. (#378)

```bash
marmite myblog --serve --bind 0.0.0.0:8000
# If 8000 is busy, tries 8001, 8002, etc.
```

## 🐛 Bug Fixes

### Obsidian Wikilinks Now Respect Slugs

Fixed an issue where Obsidian-style wikilinks (`[[Title]]`) would not correctly resolve to posts with custom slugs. Now wikilinks are matched by title and correctly link to the post's actual slug. (#403, #362)

### Special Characters Now Accepted in Tags

Tags containing special characters (accents, unicode, symbols) are now properly handled and normalized. Previously these could cause issues with tag pages and URLs. (#406)

### Improved Error Handling (No More Panics)

Removed most `unwrap()` calls throughout the codebase and replaced them with proper error handling using `Result` types. This means:

- Better error messages when something goes wrong
- No more unexpected panics during site generation
- Graceful handling of missing files and configurations
- More robust and reliable builds (#395, #334)

### Other Fixes

- Fixed footnote references requiring blank line before common link refs
- Fixed embedded shortcodes not loading when no shortcodes directory exists (#339)
- Fixed `url_for` function in shortcodes
- Fixed card shortcode to show display names for series and streams
- Fixed theme template accessing JS elements too early
- Fixed theme template license
- Keep post title capitalization in listings (#390)
- Use `media_path` from config instead of hardcoded value (#342)
- Get banner from local folder correctly
- Handle UTF-8 encoded URLs for non-ASCII filenames in server (#325)
- Fixed install script to detect correct file
- Allow multiple colorschemes on alternative theme

## 📦 Dependency Updates

### Major Updates
- `comrak`: 0.40.0 → 0.49.0 (CommonMark parser)
- `tungstenite`: Added 0.28.0 (WebSocket for live reload)
- `zip`: 4.3.0 → 7.0.0
- `image`: Added 0.25.9 (image processing)
- `rayon`: 1.10.0 → 1.11.0 (parallel processing)
- `ureq`: 3.0.12 → 3.1.4
- `tempfile`: Added 3.23.0

### Minor Updates
- `clap`: 4.5.41 → 4.5.53
- `serde`: 1.0.219 → 1.0.228
- `serde_json`: 1.0.141 → 1.0.145
- `rust-embed`: 8.7.2 → 8.9.0
- `indexmap`: 2.10.0 → 2.12.1
- `chrono`: 0.4.41 → 0.4.42
- `tera`: 1.20.0 → 1.20.1
- `regex`: 1.11.1 → 1.12.2
- `glob`: Added 0.3.3
- `urlencoding`: Added 2.1.3
- `slug`: Added 0.1

## 🔧 New Configuration Options

```yaml
# Image Resizing
skip_image_resize: false
extra:
  max_image_width: 800
  banner_image_width: 1200
  resize_filter: "quality"

# Shortcodes
enable_shortcodes: true

# Sitemap and URLs
build_sitemap: true
publish_urls_json: true

# Gallery
gallery_path: media/gallery
gallery_create_thumbnails: true
gallery_thumb_size: 300

# File mappings
file_mapping:
  - source: path/to/source
    destination: path/in/site

# Color scheme
extra:
  colormode: auto
  colormodetoggle: true
```

## 🚀 New CLI Options

- `--shortcodes`: List all available shortcodes
- `--show-urls`: Show all site URLs organized by content type
- `--skip-image-resize`: Skip image resizing for faster builds

## 🛠️ Development Changes

- Centralized regex patterns in dedicated module (#341)
- Pedantic clippy fixes for better code quality
- Increased test coverage (#314)
- Improved CI workflows
- Excluded large directories from crates.io package

## 📚 Documentation

This release includes comprehensive documentation for all new features:

- [Installation Guide](/2025-08-02-18-02-50-installation.html)
- [Python Installation](/2025-08-06-03-35-05-python-installation-with-pip.html)
- [Shortcodes Guide](/2025-08-01-shortcodes-guide.html)
- [Gallery Shortcode](/gallery-shortcode.html)
- [Image Optimization and Resizing](/2025-12-22-image-optimization.html)
- [Automatic Sitemap Generation](/2025-08-01-automatic-sitemap-generation.html)
- [Show URLs Command](/2025-08-01-show-urls-dry-run-command.html)
- [File Mapping Feature](/2025-08-01-file-mapping-feature.html)

## Upgrading

To upgrade to Marmite 0.2.7:

```bash
# If installed via cargo
cargo install marmite --force

# If installed via pip
pip install --upgrade marmite-ssg

# Or use the install script
curl -sSL https://marmite.blog/install.sh | bash
```

### Migration Notes

1. **Greentext disabled**: If you rely on greentext formatting, add `greentext: true` to your markdown parser extension config
2. Shortcodes are now enabled by default - existing sites will work without changes
3. Sitemap generation is enabled by default - a `sitemap.xml` will be created
4. Live reload is automatic when using `--watch --serve`
5. Consider enabling image resizing for large media collections to optimize performance

## What's Next

We're already working on the next release with plans for:
- More shortcode templates
- Additional gallery layouts
- Performance optimizations
- Extended IndieWeb support

## Thank You

Thanks to all contributors who made this release possible! Special thanks to:
- Guilherme Vieira Beira for live reload functionality
- Sven Steinbauer for preferred theme scheme option
- Leandro Damascena for image resizing feature
- makapuf for UTF-8 URL handling
- And everyone who reported bugs, suggested features, and contributed code.

---

For the complete changelog, see the [GitHub releases page](https://github.com/rochacbruno/marmite/releases/tag/0.2.7).
