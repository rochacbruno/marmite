---
tags: docs, configuration, reference
description: Complete reference for all Marmite configuration options, including site metadata, feature toggles, and advanced settings.
---

# Configuration Reference

This is a comprehensive reference for all configuration options available in Marmite's `marmite.yaml` file. All options can also be overridden via command-line arguments.

## Site Metadata

### Basic Information
```yaml
name: "My Blog"                    # Site name (default: "Home")
tagline: "My awesome blog"         # Site tagline (default: empty)
url: "https://myblog.com"          # Site URL (default: empty)
https: true                        # Force HTTPS in URLs (default: false)
language: "en"                     # Site language (default: "en")
```

### Visual Assets
```yaml
card_image: "media/og-image.jpg"       # Default social media card image
banner_image: "media/banner.jpg"      # Default banner image
logo_image: "media/logo.png"          # Site logo image
```

### Footer
```yaml
footer: |
  <div>
    Powered by <a href="https://github.com/rochacbruno/marmite">Marmite</a>
  </div>
```

## Content Organization

### Pagination
```yaml
pagination: 10                     # Posts per page (default: 10)
```

### Authors
```yaml
default_author: "john"             # Default author for all content

authors:
  john:
    name: "John Doe"
    avatar: "https://github.com/johndoe.png"
    bio: "Software developer and blogger"
    links:
      - ["Website", "https://johndoe.com"]
      - ["Twitter", "https://twitter.com/johndoe"]
      - ["GitHub", "https://github.com/johndoe"]
  
  jane:
    name: "Jane Smith"
    avatar: "media/jane-avatar.jpg"
    bio: "Designer and writer"
    links:
      - ["Portfolio", "https://janesmith.design"]
      - ["LinkedIn", "https://linkedin.com/in/janesmith"]
```

### Streams Configuration

Configure content streams with friendly display names:

```yaml
streams:
  tutorial:
    display_name: "Python Tutorials"
  
  guide:
    display_name: "User Guides"
  
  news:
    display_name: "Latest News"
  
  review:
    display_name: "Product Reviews"
```

Streams help organize content beyond tags and create focused RSS feeds. Posts can be assigned to streams via:
- Frontmatter: `stream: tutorial`  
- Filename patterns: `tutorial-2024-01-01-post-title.md`
- S-pattern for pages: `guide-S-comprehensive-guide.md`

Use the `stream_display_name` template function to show friendly names:
```html
{{ stream_display_name(stream=content.stream) }}
```

### Navigation Menu
```yaml
menu:
  - ["Home", "index.html"]
  - ["About", "about.html"]
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]
  - ["Authors", "authors.html"]
  - ["RSS", "index.rss"]
```

## Feature Toggles

### Search and Content Discovery
```yaml
enable_search: true                # Enable search functionality (default: false)
enable_related_content: true      # Enable backlinks/related content (default: true)
show_next_prev_links: true        # Show next/previous navigation (default: true)
```

### Content Features
```yaml
toc: true                          # Show table of contents (default: false)
json_feed: true                    # Generate JSON feeds (default: false)
```

### Source Publishing
```yaml
publish_md: true                   # Publish markdown source files (default: false)
source_repository: "https://github.com/user/repo/tree/main/content"
```

### Automatic Image Download
```yaml
image_provider: picsum             # Automatic banner image provider (default: None)
```

Configure automatic banner image download for posts. When enabled, Marmite will automatically download banner images for posts (content with dates) when:
- No `banner_image` is specified in the post's frontmatter
- The banner image file doesn't already exist

**Available providers:**
- `picsum` - Uses picsum.photos service to generate beautiful placeholder images

Images are saved as `{slug}.banner.jpg` in the media folder and use the site name, post slug, and tags to generate deterministic, unique images.

## Path Configuration

### Folder Structure
```yaml
content_path: "content"            # Content folder (default: "content")
templates_path: "templates"       # Templates folder (default: "templates")
static_path: "static"             # Static assets folder (default: "static")
media_path: "media"               # Media folder (default: "media")
site_path: ""                     # Output site subfolder (default: empty)
```

### Theme Configuration
```yaml
theme: "mytheme"                  # Theme name (default: none)
```

When a theme is specified, Marmite will:
- Load templates from `{theme}/templates/` instead of `templates/`
- Copy static files from `{theme}/static/` instead of `static/`
- Fall back to embedded templates if theme files are missing

**CLI Override**: Use `--theme mytheme` to override the configuration theme for a single build.

**Examples**:
```bash
# Build with theme from config
marmite myblog output/

# Build with specific theme (overrides config)
marmite myblog output/ --theme darkmode

# Build without theme (ignores config theme)
marmite myblog output/ --theme ""
```

## Section Titles

Customize titles for different sections of your site:

```yaml
# Page sections
pages_title: "Pages"                    # Pages section title
tags_title: "Tags"                      # Tags section title
archives_title: "Archive"               # Archives section title
authors_title: "Authors"                # Authors section title
streams_title: "Streams"                # Streams section title
search_title: "Search"                  # Search section title

# Content listing titles (use $variable for dynamic replacement)
tags_content_title: "Posts tagged with '$tag'"
archives_content_title: "Posts from '$year'"
streams_content_title: "Posts from '$stream'"
```

## Date and Time

### Date Formatting
```yaml
# Date format using strftime format
# See: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
default_date_format: "%B %d, %Y"       # Example: "January 01, 2024"
```

Common date formats:
- `"%B %d, %Y"` → "January 01, 2024"
- `"%b %e, %Y"` → "Jan 1, 2024" (default)
- `"%Y-%m-%d"` → "2024-01-01"
- `"%d/%m/%Y"` → "01/01/2024"

## Advanced Configuration

### Extra Fields
Use `extra` for custom configuration accessible in templates:

```yaml
extra:
  colorscheme: "dark"
  colorscheme_toggle: true
  analytics_id: "UA-123456789-1"
  custom_css: true
  social_media:
    twitter: "@myblog"
    github: "user/repo"
  static_folders:
    - "downloads"
    - "assets"
```

Access in templates:
```html
{% if site.extra.colorscheme_toggle %}
<button id="theme-toggle">Toggle Theme</button>
{% endif %}

{% if site.extra.analytics_id %}
<script>
  // Google Analytics with ID: {{ site.extra.analytics_id }}
</script>
{% endif %}
```

### Comments System

The recommended way of configuring comments is using the file `_comments.md`, see more on [[Enabling Comments]] page, but alternatively 
you can set a `comments` section in the settings file:

```yaml
extra:
  comments:
    title: "Comments"
    source: |
      <script src="https://giscus.app/client.js"
              data-repo="owner/repo"
              data-repo-id="REPO_ID"
              ...
              async>
      </script>
```

## CLI Configuration Overrides

All configuration options can be overridden via command-line arguments:

```bash
# Site metadata
marmite ./site --name "My Site" --tagline "My awesome site"

# URLs and paths
marmite ./site --url "https://mysite.com" --https true

# Features
marmite ./site --enable-search true --toc true --json-feed true

# Site generation
marmite ./site --build-sitemap true --publish-urls-json true

# Source publishing
marmite ./site --publish-md true --source-repository "https://github.com/user/repo"

# Image provider
marmite ./site --image-provider picsum

# Paths
marmite ./site --content-path "posts" --static-path "assets"

# Formatting
marmite ./site --default-date-format "%Y-%m-%d"
```

## Complete Example

Here's a complete `marmite.yaml` configuration example:

```yaml
# Site metadata
name: "Tech Blog"
tagline: "Exploring the latest in technology"
url: "https://techblog.example.com"
https: true
language: "en"

# Visual assets
card_image: "media/og-card.jpg"
banner_image: "media/banner.jpg"
logo_image: "media/logo.png"

# Content settings
pagination: 15
default_author: "alex"
default_date_format: "%B %d, %Y"

# Authors
authors:
  alex:
    name: "Alex Johnson"
    avatar: "https://github.com/alexjohnson.png"
    bio: "Full-stack developer and tech enthusiast"
    links:
      - ["Website", "https://alexjohnson.dev"]
      - ["Twitter", "https://twitter.com/alexjohnson"]
      - ["GitHub", "https://github.com/alexjohnson"]

# Streams
streams:
  tutorial:
    display_name: "Tutorials"
  review:
    display_name: "Product Reviews"
  tip:
    display_name: "Quick Tips"

# Navigation
menu:
  - ["Home", "index.html"]
  - ["Tutorials", "tutorials.html"]
  - ["Reviews", "reviews.html"]
  - ["About", "about.html"]
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]

# Features
enable_search: true
enable_related_content: true
show_next_prev_links: true
toc: true
json_feed: true

# Source publishing
publish_md: true
source_repository: "https://github.com/alexjohnson/techblog/tree/main/content"

# Automatic image download
image_provider: picsum

# Section titles
pages_title: "All Pages"
tags_title: "Topics"
archives_title: "Post Archive"
authors_title: "Contributors"
streams_title: "Categories"

# Custom configuration
extra:
  colorscheme: "dark"
  colorscheme_toggle: true
  analytics_id: "UA-123456789-1"
  social_media:
    twitter: "@techblog"
    github: "alexjohnson/techblog"
  custom_features:
    newsletter: true
    dark_mode: true
  static_folders:
    - "downloads"
    - "resources"
```

## Configuration Validation

Marmite validates your configuration and will warn about:
- Invalid date formats
- Missing required fields
- Malformed YAML syntax
- Invalid boolean values

Run `marmite --generate-config` to create a configuration file with defaults, then customize as needed.

## Environment-Specific Configuration

You can maintain different configurations for different environments:

```bash
# Development
marmite . output/ --config dev.yaml

# Production
marmite . output/ --config prod.yaml

# Testing
marmite . output/ --config test.yaml
```

## Sitemap Generation

Control automatic sitemap.xml generation:

```yaml
# Generate sitemap.xml (default: true)
build_sitemap: true
```

When enabled, Marmite automatically generates a sitemap.xml file containing all your site's URLs. The sitemap uses absolute URLs when a `url` is configured, otherwise relative URLs.

See the [[Automatic Sitemap Generation]] documentation for more details.

## URLs JSON Generation

Control automatic URLs JSON file generation:

```yaml
# Generate urls.json (default: true)
publish_urls_json: true
```

When enabled, Marmite automatically generates a `urls.json` file containing all your site's URLs organized by content type. This file has the same structure as the `--show-urls` command output and includes:

- Posts, pages, tags, authors, series, streams, and archive URLs
- RSS and JSON feed URLs  
- Pagination page URLs
- File mapping URLs
- Summary with counts and metadata

The JSON uses absolute URLs when a `url` is configured, otherwise relative URLs.

## File Mapping

Copy arbitrary files from source to destination during site generation:

```yaml
file_mapping:
  - source: path/to/source.txt
    dest: destination.txt
  - source: assets/imgs/*.jpg
    dest: media/photos
  - source: docs_folder
    dest: documentation
```

**Source types**:
- Single files: `source: file.txt`
- Directories: `source: folder_name`
- Glob patterns: `source: pattern/*.ext`

**Path resolution**:
- Relative paths are resolved from the input directory
- Absolute paths are used as-is
- Destination paths are relative to the output directory

See the [[File Mapping Feature]] documentation for detailed examples and use cases.

## Markdown parser options 

Marmite also allows customizing the markdown parser, the options are described on [[Configurable Markdown Parser Options]]

----

This comprehensive reference covers all available configuration options. Mix and match these settings to customize your Marmite site exactly as you need it.
