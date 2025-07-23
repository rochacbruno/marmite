---
title: Marmite 0.2.6 Release Notes
slug: marmite-0-2-6-release-notes
description: "Marmite 0.2.6 brings powerful new features including themes, series support, enhanced markdown parsing, and much more. This release includes breaking changes to the --start-theme command."
tags: [release-notes, marmite, features, announcement]
author: marmite
stream: draft
---

We're excited to announce Marmite 0.2.6, a release packed with powerful new features, improvements, and bug fixes. This release introduces themes, series support for grouping related content, enhanced markdown parsing capabilities, and much more.

## ⚠️ Breaking Changes

### --start-theme Command Updated

The `--start-theme` command now requires a theme name parameter:

```bash
# Old (no longer works)
marmite myblog --start-theme

# New (required)
marmite myblog --start-theme mytheme
```

This change allows you to specify a custom name for your theme when initializing a new site.

## 🎉 New Features

### Themes Support

Marmite now supports themes! [[Introducing themes in marmite]]

- Use `--theme <name>` CLI option to override the theme from configuration
- Configure themes in `marmite.yaml` with the `theme` field
- Themes can have their own `templates/` and `static/` directories
- Embedded theme templates for quick theme creation

```yaml
# Enable a theme in your configuration
theme: mytheme
```

### Series Feature

Group related content into series with chronological ordering. Perfect for tutorials, course materials, or any sequential content. [[Organizing Content with Series in Marmite]]

- Define series in content frontmatter
- Automatic series pages with oldest-to-newest ordering
- Series navigation takes precedence over streams
- RSS and JSON feeds for each series

```yaml
# In your content frontmatter
series: python-tutorial

# In marmite.yaml
series:
  python-tutorial:
    display_name: "Python Tutorial"
    description: "Learn Python programming step by step"
```

### Enhanced Streams

Streams are now more powerful with filename-based detection and display names. [Explore the streams guide](/streams-guide.html) and [[filename-based-streams-organize-content-with-file-naming]]

- Automatically detect streams from filenames (e.g., `dev-my-post.md`)
- Configure display names for streams
- Better stream organization and navigation

```yaml
streams:
  dev:
    display_name: "Development"
    description: "Technical development posts"
```

### Configurable Markdown Parser

Take full control over markdown parsing with extensive configuration options. [[Configurable Markdown Parser Options]]

- Enable/disable CommonMark extensions
- Configure rendering options
- Support for alerts, wikilinks, spoilers, and more

```yaml
markdown_parser:
  render:
    unsafe: true
    figure_with_caption: true
  extension:
    alerts: true
    wikilinks_title_before_pipe: true
```

### Automatic Image Download

Marmite can now automatically download banner images from providers. [Learn about image providers](/automatic-image-download.html)

- Configure with `image_provider: picsum`
- Automatically downloads images when URLs are provided
- Caches images locally for better performance

### IndieWeb Microformats

All templates now include proper IndieWeb microformats for better web interoperability. [Read about IndieWeb compliance](/indieweb-compliance.html)

- h-entry, h-card, and other microformats
- Better semantic HTML structure
- Improved compatibility with IndieWeb tools

### Markdown Source Publishing

Publish your markdown source files alongside HTML output. [See the source publishing guide](/markdown-source-publishing.html)

- Enable with `publish_md: true`
- Link to source repository with `source_repository`
- Transparency for readers who want to see the source

### Additional Features

- **Next/Previous Navigation**: Automatic navigation links between posts
- **HTTPS Support**: New `--https` flag to enforce HTTPS in URLs
- **Draft Filtering**: Draft posts are now properly excluded from feeds
- **Enhanced url_for Function**: Better asset URL handling in templates

## 🐛 Bug Fixes

- Fixed RSS feed URLs duplicating `http://` protocol (#257)
- Prevented media file links from being converted to HTML links (#290)
- Fixed broken documentation links (#280)
- Resolved issue with generated folder being tracked (#267)
- Fixed auto theme switching based on system preferences (#262)

## 📦 Dependency Updates

### Major Updates
- `comrak`: 0.35.0 → 0.40.0 (CommonMark parser)
- `ureq`: Added 3.0.12 (HTTP client for image downloads)

### Minor Updates
- `clap`: 4.5.26 → 4.5.41
- `rust-embed`: 8.5.0 → 8.7.2
- `serde`: 1.0.217 → 1.0.219
- `serde_json`: 1.0.138 → 1.0.141
- `chrono`: 0.4.39 → 0.4.41
- `env_logger`: 0.11.6 → 0.11.8
- `indexmap`: 2.7.1 → 2.10.0
- `rss`: 2.0.11 → 2.0.12

## 🔧 New Configuration Options

```yaml
# Theme configuration
theme: mytheme

# HTTPS enforcement
https: true

# Source publishing
publish_md: true
source_repository: https://github.com/user/repo

# Image provider
image_provider: picsum

# Navigation
show_next_prev_links: true

# Series configuration
series:
  tutorial-name:
    display_name: "Tutorial Display Name"
    description: "Tutorial description"

# Stream display names
streams:
  stream-name:
    display_name: "Stream Display Name"

# Markdown parser (extensive options available)
markdown_parser:
  render:
    unsafe: true
  extension:
    alerts: true
```

## 🚀 New CLI Options

- `--theme <name>`: Override theme from configuration
- `--https`: Enforce HTTPS protocol
- `--publish-md`: Publish markdown source files
- `--source-repository <url>`: Set source repository URL
- `--image-provider <provider>`: Configure image provider
- `--show-next-prev-links`: Enable/disable navigation links

## 📚 Documentation

This release includes comprehensive documentation for all new features:

- [Getting Started Guide](/getting-started.html)
- [Command Line Interface Guide](/marmite-command-line-interface.html)
- [Configuration Reference](/configuration-reference.html)
- [Template Reference](/template-reference.html)
- [Themes Feature Guide](/introducing-themes-in-marmite)
- [Series Feature Guide](/organizing-content-with-series-in-marmite.html)
- [Streams Guide](/streams-guide.html)
- [Draft Posts Guide](/how-to-use-draft-posts-in-marmite.html)
- [Link Checker with Lychee](/how-to-run-a-link-checker-on-your-marmite-website.html)

## 🛠️ Development Changes

- Migrated from Just to Mask task runner for development
- Improved development workflow and tooling

## Upgrading

To upgrade to Marmite 0.2.6:

```bash
# If installed via cargo
cargo install marmite --force

# Or download the latest binary from GitHub releases
```

### Migration Notes

1. If you use `--start-theme`, update your scripts to include a theme name
2. Review the new configuration options and add any you'd like to use
3. Consider organizing sequential content using the new series feature
4. Enable draft filtering if you have draft posts that shouldn't appear in feeds

## What's Next

We're already working on the next release with plans for:
- Theme package management
- More image providers
- Additional IndieWeb features
- Performance improvements
- Image Gallery
- Documentation theme

## Thank You

Thanks to all contributors who made this release possible! Special thanks to those who reported bugs, suggested features, and contributed code.

---

For the complete changelog, see the [GitHub releases page](https://github.com/rochacbruno/marmite/releases/tag/0.2.6).
