---
name: marmite
description: Build and manage static sites with marmite - a zero-config static site generator that turns markdown files into websites
---

# Marmite Static Site Generator

Marmite is a static site generator written in Rust that converts a folder of markdown files into a complete website. It requires zero configuration to get started - just point it at a folder with `.md` files and it produces HTML.

- Repository: https://github.com/rochacbruno/marmite
- Documentation: https://marmite.blog
- Template engine: Tera (Jinja2-like)
- Config format: YAML (`marmite.yaml`)

## Installation

```bash
# Quick install (Linux/macOS)
curl -sS https://marmite.blog/install.sh | sh

# Python (pip)
pip install marmite
# or without installing
uvx marmite

# Rust
cargo install marmite
# or pre-compiled via cargo-binstall
cargo binstall marmite

# Homebrew
brew install marmite

# Docker
docker run --rm -v $(pwd):/input ghcr.io/rochacbruno/marmite

# Windows
iwr -useb https://marmite.blog/install.ps1 | iex
```

See `references/installation.md` for all install methods, custom directories, and troubleshooting.

## Essential Commands

```bash
# Build a site from a folder of markdown files
marmite <input_folder> [output_folder]

# Build, serve locally, and watch for changes
marmite <input_folder> --serve --watch

# Scaffold a new project with sample content
marmite <folder> --init-site

# Generate a default marmite.yaml
marmite <folder> --generate-config

# Create a new post
marmite <folder> --new "My Post Title"

# Create a new page (no date)
marmite <folder> --new "About" -p

# Create a post with tags and open in editor
marmite <folder> --new "My Post" -t "rust,web" -e

# Initialize custom templates
marmite <folder> --init-templates

# Create a new theme
marmite <folder> --start-theme mytheme

# Install a remote theme
marmite <folder> --set-theme https://github.com/user/marmite-theme

# Install agent skill (no input folder needed)
marmite --skill-install                      # for standard agents (.agents/)
marmite --skill-install-claude               # for Claude Code (.claude/skills/)
marmite --skill-install --skill-install-claude  # both at once
```

## Workflow: Start a New Project

```bash
mkdir mysite && cd mysite
marmite . --init-site
marmite . --serve --watch
```

This creates:
```
mysite/
  content/          # Markdown files go here
  marmite.yaml      # Site configuration
  site/             # Generated output (default)
```

If you already have markdown files in a folder, just run `marmite <folder>` with no setup needed.

## Workflow: Content Authoring

### Posts vs Pages

Content with a **date** is a **post** (appears in feeds, index, archive, search). Content without a date is a **page** (standalone, accessible by direct link or menu).

Dates can come from the filename or frontmatter:

```bash
# Date in filename (becomes a post automatically)
content/2024-06-15-my-post.md

# Date in frontmatter
content/my-post.md  # with date: 2024-06-15 in frontmatter

# No date (becomes a page)
content/about.md
```

### Frontmatter

Three formats are supported. YAML is recommended:

```yaml
---
title: "My Post Title"
date: 2024-06-15
slug: custom-url-slug
tags: rust, web, tutorial
authors: alice, bob
description: "A short description for SEO and feeds"
series: my-tutorial-series
stream: tutorial
pinned: true
toc: true
card_image: media/social-card.jpg
banner_image: media/banner.jpg
extra:
  math: true
  mermaid: true
---
```

TOML (`+++` delimiters) and JSON (`{}` wrapper) are also supported. See `references/frontmatter.md` for the full field reference.

### Creating Content via CLI

```bash
# New post (date auto-set to today)
marmite . --new "Getting Started with Rust"

# New page
marmite . --new "About Me" -p

# Post with tags, open in editor
marmite . --new "Rust Tips" -t "rust,tips" -e
```

### Taxonomy

**Tags** - group content by topic:
```yaml
tags: rust, web, tutorial
```
Generates: `/tags.html`, `/tag-rust.html`, `/tag-rust.rss`

**Authors** - group by author:
```yaml
authors: alice
```
Generates: `/authors.html`, `/author-alice.html`

Author profiles are configured in `marmite.yaml`:
```yaml
authors:
  alice:
    name: Alice Smith
    avatar: https://example.com/alice.png
    bio: "Rust developer"
    links:
      - ["Github", "https://github.com/alice"]
```

**Streams** - separate content categories:
```yaml
stream: tutorial
```
Generates: `/tutorial.html`, `/tutorial.rss`, `/streams.html`

Special stream `draft` hides posts from main feeds while keeping them accessible by URL.

Configure display names in `marmite.yaml`:
```yaml
streams:
  tutorial:
    display_name: "Tutorials"
```

**Series** - multi-part ordered content:
```yaml
series: python-tutorial
```
Generates: `/serie-python-tutorial.html`, `/series.html`

Posts in a series get automatic prev/next navigation. Configure in `marmite.yaml`:
```yaml
series:
  python-tutorial:
    display_name: "Python Tutorial"
    description: "Learn Python step by step"
```

### Markdown Features

Marmite supports extended markdown:
- Tables, strikethrough, task lists, footnotes
- Wikilinks: `[[page-slug]]` or `[[Display Text|page-slug]]`
- Alerts: `> [!NOTE]`, `> [!WARNING]`, `> [!TIP]`
- Spoilers: `||hidden text||`
- Description lists, underline, multiline block quotes (`>>>`)
- Math (when `extra.math: true`): `$inline$` and `$$display$$`
- Mermaid diagrams (when `extra.mermaid: true`)

## Workflow: Layout Customization with Fragment Files

Files prefixed with `_` in the content directory inject content into template regions without generating their own pages.

| File | Purpose |
|------|---------|
| `_hero.md` | Hero section on the homepage |
| `_announce.md` | Announcement banner |
| `_header.md` | Custom header content |
| `_footer.md` | Custom footer content |
| `_sidebar.md` | Sidebar content |
| `_comments.md` | Comments section (e.g., Giscus script) |
| `_references.md` | Global markdown link references appended to every file |
| `_htmlhead.md` | Raw HTML injected into `<head>` |
| `_markdown_header.md` | Markdown prepended to every content file |
| `_markdown_footer.md` | Markdown appended to every content file |
| `_404.md` | Custom 404 page |

Example `_hero.md`:
```markdown
>>>
Welcome to my blog! I write about Rust and web development.
>>>
```

Example `_references.md`:
```markdown
[Github]: https://github.com/myuser
[docs]: <./tag-docs.html> "Documentation"
```

Example `_comments.md` (Giscus):
```html
<script src="https://giscus.app/client.js"
  data-repo="user/repo"
  data-repo-id="YOUR_ID"
  data-category="Announcements"
  data-category-id="YOUR_CAT_ID"
  data-mapping="pathname"
  crossorigin="anonymous"
  async>
</script>
```

## Workflow: Configuration

Create or edit `marmite.yaml` in the project root. Key options:

```yaml
name: "My Blog"
tagline: "A blog about things"
url: "https://myblog.com"
language: "en"
pagination: 10
enable_search: true
toc: true

menu:
  - ["Home", "index.html"]
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]
  - ["About", "about.html"]

default_author: myuser
default_date_format: "%B %d, %Y"

extra:
  colorscheme: dracula
  colorscheme_toggle: true
  colormode: dark
```

See `references/config-reference.md` for the complete list of all configuration options.

## Workflow: Template Customization

Marmite uses the Tera template engine (Jinja2-like syntax).

```bash
# Export default templates to customize
marmite <folder> --init-templates
```

This creates a `templates/` directory with all template files. The key templates:

| Template | Purpose |
|----------|---------|
| `base.html` | Base layout (all pages extend this) |
| `content.html` | Single post/page view |
| `list.html` | Content listings (index, tag pages, etc.) |
| `group.html` | Grouped content (tags overview, streams overview) |

Template blocks in `base.html`:
- `{% block seo %}` - Open Graph and meta tags
- `{% block head %}` - CSS and head elements
- `{% block main %}` - Main content area
- `{% block tail %}` - Scripts at end of body

Key template variables:
- `site` - The full site configuration object
- `site.name`, `site.tagline`, `site.url`, `site.extra`
- `menu` - Navigation menu items
- `content` - Current post/page object (on content pages)
- `content_list` - Array of posts (on list pages)
- `hero`, `sidebar`, `header`, `footer`, `announce` - Fragment content

Custom Tera functions available:
- `url_for(path="page.html", abs=false)` - Generate URLs
- `group(kind="tag", ord="desc", items=0)` - Group content
- `get_posts(ord="desc", items=10)` - Get sorted posts
- `get_data_by_slug(slug="my-post")` - Look up content by slug
- `source_link(content=content)` - Link to markdown source
- `stream_display_name(stream="tutorial")` - Get display name
- `series_display_name(series="my-series")` - Get display name

Custom filters:
- `{{ content.date | default_date_format }}` - Format dates
- `{{ items | remove_draft }}` - Filter out draft content

See `references/tera-templates.md` for the full template reference.

## Workflow: Theme Creation

```bash
# Create a new theme from the built-in template
marmite <folder> --start-theme mytheme
```

This creates:
```
mytheme/
  templates/
    base.html
    content.html
    list.html
    group.html
  static/
    style.css
    script.js
    custom.css
    custom.js
    favicon.ico
    colorschemes/
  theme.json
  README.md
```

Edit `theme.json` for theme metadata:
```json
{
  "name": "My Theme",
  "version": "0.1.0",
  "author": "Your Name",
  "description": "A custom marmite theme",
  "license": "MIT",
  "marmite_version": ">=0.3.0"
}
```

Activate the theme in `marmite.yaml`:
```yaml
theme: mytheme
```

Install a remote theme:
```bash
marmite <folder> --set-theme https://github.com/user/marmite-theme-name
```

Available built-in colorschemes: catppuccin, clean, dracula, github, gruvbox, iceberg, minimal, minimal_wb, monokai, nord, one, solarized, typewriter.

## Workflow: Shortcodes

Shortcodes are reusable content blocks. Default syntax uses HTML comments:

```markdown
<!-- .youtube id=dQw4w9WgXcQ -->
<!-- .posts items=5 -->
<!-- .tags ord=asc -->
<!-- .gallery path=photos width=200 -->
```

Built-in shortcodes: youtube, spotify, posts, pages, tags, streams, authors, series, card, gallery, toc, socials.

### Creating Custom Shortcodes

Place `.html` or `.md` files in the `shortcodes/` directory.

HTML shortcode (`shortcodes/alert.html`):
```html
{% macro alert(type="info", message="") %}
<div class="alert alert-{{ type }}">{{ message }}</div>
{% endmacro alert %}
```

Usage: `<!-- .alert type=warning message=Be careful! -->`

Markdown shortcode (`shortcodes/note.md`):
```markdown
> **{{ title | default(value="Note") }}**: {{ content }}
```

See `references/shortcodes.md` for the full shortcode reference.

## Workflow: Image Optimization

Marmite automatically resizes images during build to optimize page load times.

```yaml
# marmite.yaml
extra:
  max_image_width: 800          # Max width for regular images (pixels)
  banner_image_width: 1200      # Max width for banner/hero images
  resize_filter: "quality"      # "fast", "balanced", or "quality"
```

Features:
- Parallel processing using all CPU cores
- Incremental builds - unchanged images are cached
- Banner images detected by `.banner.` in filename or `banner_image` frontmatter
- Supports JPEG, PNG, WebP, GIF, AVIF, BMP, TIFF
- Originals preserved - only output copies are resized

Skip during development for faster builds:
```bash
marmite <folder> --serve --skip-image-resize
```

### Automatic Image Download

Marmite can auto-download banner images for posts without one:

```yaml
image_provider: picsum
```

Downloads a deterministic placeholder image as `{slug}.banner.jpg` for each post. Only applies to posts, not pages. Delete the downloaded image and rebuild to get a different one.

## Workflow: Comments

Add a comment system by creating `content/_comments.md`:

```markdown
##### Comments

<script src="https://giscus.app/client.js"
    data-repo="youruser/yourrepo"
    data-repo-id="YOUR_REPO_ID"
    data-category="Comments"
    data-category-id="YOUR_CATEGORY_ID"
    data-mapping="pathname"
    data-theme="preferred_color_scheme"
    data-loading="lazy"
    crossorigin="anonymous"
    async>
</script>
```

Alternatively, configure in `marmite.yaml` under `extra.comments`. Disable per-post with `comments: false` in frontmatter.

See `references/comment-system.md` for Giscus, Utterances, Hatsu, and other options.

## Workflow: Link Checking

Use [lychee](https://lychee.cli.rs/) to validate links in your built site:

```bash
# Build and check
marmite <folder> site
lychee --verbose ./site --extensions html

# Or check against the running server
marmite <folder> --serve &
lychee http://localhost:8000
```

Add to CI with the `lycheeverse/lychee-action` GitHub Action for automated weekly checks.

## IndieWeb Compliance

Marmite's default templates include IndieWeb microformats out of the box:

- `h-card` for author profiles and site identity
- `h-entry` for blog posts and list items
- `h-feed` for content collections
- `p-category` for tags
- `dt-published` for dates

This makes your site machine-readable for feed readers, search engines, and IndieWeb tools. No configuration needed.

For Fediverse verification, add to `marmite.yaml`:
```yaml
extra:
  fediverse_verification: "https://mastodon.social/@username"
```

## Workflow: Building and Deploying

```bash
# Build the site
marmite <input_folder> <output_folder>

# Build with dev server and file watching
marmite <input_folder> --serve --watch

# Custom server address
marmite <input_folder> --serve --bind 127.0.0.1:3000

# Force full rebuild
marmite <input_folder> --force

# Show all generated URLs
marmite <input_folder> --show-urls
```

The output is a flat directory of static HTML, CSS, and JS files. Deploy by copying the output folder to any static hosting provider (Netlify, Vercel, GitHub Pages, Cloudflare Pages, or any web server).

### File Mapping

Copy arbitrary files into the output during build:
```yaml
file_mapping:
  - source: ai/llms.txt
    dest: llms.txt
  - source: static/favicon.ico
    dest: favicon.ico
```

### Sitemap and Feeds

Generated automatically:
- `sitemap.xml` (when `build_sitemap: true`, default)
- `index.rss` (always)
- `index.json` (when `json_feed: true`)
- `urls.json` (when `publish_urls_json: true`, default)
- Per-tag, per-stream, per-series RSS feeds

## Reference Files

- `references/cli-reference.md` - Complete CLI flags, options, and command examples
- `references/installation.md` - All installation methods (curl, pip, cargo, brew, Docker, Windows)
- `references/config-reference.md` - Complete configuration options
- `references/frontmatter.md` - Content frontmatter fields
- `references/content-organization.md` - Directory structure, taxonomy, fragment files, and site organization strategies
- `references/markdown-format.md` - Markdown syntax, extensions, wikilinks, math, diagrams, alerts
- `references/tera-templates.md` - Template system, variables, functions, and filters
- `references/shortcodes.md` - Shortcode creation and built-in shortcodes
- `references/deployment-guide.md` - Deploying to GitHub Pages, GitLab, Netlify, Vercel, Cloudflare, Docker, Nginx, Apache
- `references/comment-system.md` - Setting up Giscus, Utterances, Hatsu, and other comment systems
