# Marmite Configuration Reference

All configuration goes in `marmite.yaml` at the project root. Every field is optional - marmite works with no config file at all.

## Site Identity

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | String | `"Home"` | Site name, used in title and feeds |
| `tagline` | String | `""` | Site tagline/subtitle |
| `url` | String | `""` | Site base URL (e.g., `https://myblog.com`) |
| `https` | bool | `false` | Force HTTPS in generated URLs when protocol is missing |
| `footer` | String | Marmite credit HTML | Footer HTML content |
| `language` | String | `"en"` | Site language code (2-letter ISO) |
| `logo_image` | String | `""` | Path to site logo image |

## Content Organization

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `pagination` | int | `10` | Posts per page on list views |
| `default_author` | String | `""` | Default author for posts without explicit author |
| `default_date_format` | String | `"%b %e, %Y"` | Date display format (chrono strftime syntax) |
| `toc` | bool | `false` | Enable table of contents on all posts |
| `show_next_prev_links` | bool | `true` | Show previous/next navigation on posts |
| `enable_related_content` | bool | `true` | Show backlinks and related content |

## Navigation Menu

```yaml
menu:
  - ["Home", "index.html"]
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]
  - ["External", "https://example.com"]
```

Default menu includes Tags, Archive, and Authors links.

## Images

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `card_image` | String | `""` | Default Open Graph / social media card image |
| `banner_image` | String | `""` | Default banner image for posts |
| `image_provider` | String | none | Auto-download banner images. Options: `picsum` |
| `skip_image_resize` | bool | `false` | Skip image optimization (faster dev builds) |

## Search

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable_search` | bool | `false` | Enable full-text search |
| `search_show_matches` | bool | `false` | Show matched text snippets in results |
| `search_match_count` | int | `3` | Number of match snippets per result |
| `search_title` | String | `"Search"` | Title for the search page |

## Paths

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `content_path` | String | `"content"` | Subfolder for markdown files inside input folder |
| `templates_path` | String | `"templates"` | Subfolder for Tera templates |
| `static_path` | String | `"static"` | Subfolder for static assets (CSS, JS, images) |
| `media_path` | String | `"media"` | Subfolder for content media (images in posts) |
| `site_path` | String | `""` | Subfolder within output directory |
| `gallery_path` | String | `"gallery"` | Subfolder for gallery images |

## Feeds and Output

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `json_feed` | bool | `false` | Generate JSON feed (`index.json`) |
| `build_sitemap` | bool | `true` | Generate `sitemap.xml` |
| `publish_urls_json` | bool | `true` | Generate `urls.json` with all site URLs |
| `publish_md` | bool | `false` | Copy markdown source files to output |
| `source_repository` | String | none | URL to source repo (e.g., `https://github.com/user/repo/tree/main/content`) |

RSS feeds (`index.rss`, per-tag, per-stream, per-series) are always generated.

## Link Checking

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `check_internal_links` | bool | `false` | Validate internal links at build time and warn about broken ones |
| `strict_internal_links` | bool | `false` | Fail the build when broken internal links are found (requires `check_internal_links: true`) |

## Shortcodes

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable_shortcodes` | bool | `true` | Process shortcodes in markdown |
| `shortcode_pattern` | String | HTML comment pattern | Custom regex for shortcode syntax |

Default pattern matches HTML comments: `<!-- .name param=value -->`

Alternative patterns:
```yaml
# Hugo-style
shortcode_pattern: '\{\{<\s*(\w+)([^>]*)\s*>\}\}'

# Jekyll-style
shortcode_pattern: '\{%\s*(\w+)([^%]*)\s*%\}'
```

## Themes

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `theme` | String | none | Theme folder name (relative to input folder) |

When a theme is set, templates and static files are loaded from the theme folder instead of the project root.

## AT Protocol standard.site

Configure the AT Protocol standard.site publishing integration:

```yaml
atproto:
  handle: "myhandle.bsky.social"                                   # Your AT Protocol handle
  publication_uri: "at://did:plc:.../site.standard.publication/..."  # The publication AT-URI
  publish_content: true                                            # Publish full markdown body text (default: true)
```

## Authors

```yaml
authors:
  username:
    name: "Display Name"
    avatar: "https://example.com/avatar.png"
    bio: "Short biography"
    links:
      - ["GitHub", "https://github.com/username"]
      - ["Twitter", "https://twitter.com/username"]
```

Fields: `name` (required), `avatar` (optional), `bio` (optional), `links` (optional, list of `[label, url]` pairs).

## Streams

```yaml
streams:
  tutorial:
    display_name: "Tutorials"
  news:
    display_name: "Latest News"
```

Each stream key maps to the `stream` frontmatter value. The `display_name` is shown in navigation and headings.

## Series

```yaml
series:
  python-tutorial:
    display_name: "Python Tutorial"
    description: "A comprehensive Python programming tutorial"
```

Fields: `display_name` (required), `description` (optional).

## File Mapping

Copy files from source to output during build:

```yaml
file_mapping:
  - source: ai/llms.txt
    dest: llms.txt
  - source: static/favicon.ico
    dest: favicon.ico
```

Supports glob patterns.

## Gallery

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `gallery_path` | String | `"gallery"` | Path to gallery images folder |
| `gallery_create_thumbnails` | bool | `true` | Auto-generate thumbnail images |
| `gallery_thumb_size` | int | `50` | Thumbnail size in pixels |

Configure named galleries:
```yaml
galleries:
  photos:
    path: "gallery/photos"
```

## Section Titles

Customize the titles shown on taxonomy pages:

| Field | Default |
|-------|---------|
| `pages_title` | `"Pages"` |
| `tags_title` | `"Tags"` |
| `tags_content_title` | `"Posts tagged with '$tag'"` |
| `archives_title` | `"Archive"` |
| `archives_content_title` | `"Posts from '$year'"` |
| `authors_title` | `"Authors"` |
| `streams_title` | `"Streams"` |
| `streams_content_title` | `"Posts from '$stream'"` |
| `series_title` | `"Series"` |
| `series_content_title` | `"Posts from '$series' series"` |

The `$tag`, `$year`, `$stream`, `$series` placeholders are replaced with actual values.

## Code Highlighting

```yaml
code_highlight:
  enabled: true
  light_theme: "github-light"
  dark_theme: "github-dark"
```

## Markdown Parser Options

Fine-tune the markdown parser (comrak):

```yaml
markdown_parser:
  render:
    unsafe: true                      # Allow raw HTML in markdown (default: true)
    ignore_empty_links: true          # Ignore empty link references (default: true)
    figure_with_caption: true         # Wrap images in <figure> tags (default: true)
  parse:
    relaxed_tasklist_matching: true    # Relaxed task list syntax (default: true)
  extension:
    tagfilter: false                  # HTML tag filtering (default: false)
    strikethrough: true               # ~~strikethrough~~ (default: true)
    table: true                       # Table support (default: true)
    autolink: true                    # Auto-detect URLs (default: true)
    tasklist: true                    # Task list checkboxes (default: true)
    footnotes: true                   # Footnote references (default: true)
    description_lists: true           # Description lists (default: true)
    multiline_block_quotes: true      # >>> block quotes (default: true)
    underline: true                   # __underline__ (default: true)
    spoiler: true                     # ||spoiler|| (default: true)
    greentext: false                  # >greentext (default: false)
    shortcodes: true                  # Shortcode processing (default: true)
    wikilinks_title_before_pipe: true # [[Title|slug]] (default: true)
    wikilinks_title_after_pipe: false # [[slug|Title]] (default: false)
    alerts: true                      # > [!NOTE] alerts (default: true)
```

## Extra Configuration

The `extra` field is a free-form key-value map for template customization:

```yaml
extra:
  # Colorscheme
  colorscheme: dracula          # Name of CSS colorscheme file
  colorscheme_toggle: true      # Show colorscheme picker
  colormode: dark               # Default color mode (light/dark)
  colormodetoggle: false        # Show light/dark toggle

  # Social
  social_networks:
    github:
      url: https://github.com/username
    linkedin:
      url: https://linkedin.com/in/username

  # Comments (e.g., Giscus)
  comments:
    title: "Comments"
    source: |
      <script src="https://giscus.app/client.js" ...></script>

  # Additional static folders to copy
  static_folders:
    - downloads
    - assets

  # Fediverse verification
  fediverse_verification: https://mastodon.social/@me

  # Image sizing
  banner_image_width: 800       # Resize banner images (pixels)
  max_image_width: 600          # Max width for all images (pixels)
  resize_filter: "quality"      # "fast" (Triangle), "balanced" (CatmullRom), "quality" (Lanczos3)

  # Math and diagrams
  math: true                    # Enable KaTeX math rendering
  mermaid: true                 # Enable Mermaid diagrams
```

Access extra values in templates: `{{ site.extra.colorscheme }}`, `{{ site.extra.math }}`.

## CLI Overrides

Most config fields can be overridden via CLI flags:

```bash
marmite <folder> --name "My Site" --pagination 20 --enable-search true --toc true
```

CLI flags take precedence over `marmite.yaml` values.
