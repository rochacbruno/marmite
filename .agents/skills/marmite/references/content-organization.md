# Marmite Content Organization Reference

This reference covers how to organize content in a marmite site - directory structure, content types, taxonomy, fragment files, and strategies for different site types.

## Directory Structure

A typical marmite project:

```
mysite/
  marmite.yaml              # Site configuration
  content/                   # Markdown files (configurable via content_path)
    pages/                   # Static pages (no date)
      about.md
      contact.md
    posts/                   # Blog posts (or topic subfolders)
      my-post.md             # Post with date in frontmatter
    media/                   # Images and assets for content
      photo.jpg
    _hero.md                 # Fragment (layout injection)
    _references.md           # Global references
  templates/                 # Custom Tera templates (optional)
  static/                    # Static assets (CSS, JS, images)
  shortcodes/                # Custom shortcodes (optional)
  site/                      # Generated output (default)
```

Marmite also works without the `content/` subfolder - markdown files can live directly in the input folder.

For simple sites, a flat folder of markdown files works fine. For growing sites, organizing content into subfolders with `frontmatter.yaml` files for shared defaults is recommended.

## Folder-Level Frontmatter Defaults

Content subfolders can have a `frontmatter.yaml` file that provides default frontmatter values for all `.md` files in that folder. Per-file frontmatter overrides the defaults. `title` and `slug` are never inherited. Works at any nesting depth.

```
content/
  frontmatter.yaml                  # Root-level defaults (apply to all content)
  tutorials/
    frontmatter.yaml                # Inherits from root, adds stream
    python/
      frontmatter.yaml              # Inherits from tutorials, adds tags
      basics.md                     # Gets root + tutorials + python defaults
      advanced/
        decorators.md               # Inherits from python/ (nearest ancestor)
  2024-01-15-standalone.md          # Inherits from root only
```

Files in nested subfolders without their own `frontmatter.yaml` inherit from the nearest ancestor that has one.

**Merge priority** (lowest to highest):

1. Root `content/frontmatter.yaml`
2. Parent subfolder `frontmatter.yaml` files (layered from shallowest to deepest)
3. Filename conventions (date, stream, language from filename)
4. The markdown file's own frontmatter block

Subfolders without a `frontmatter.yaml` continue to work as before - their content is rendered normally.

## Content Types

### Posts

Content **with a date** (from filename or frontmatter). Posts appear in the index, feeds, search, archive, and pagination.

```
content/2024-06-15-my-post.md          # Date from filename
content/2024-06-15-14-30-my-post.md    # Date with time from filename
content/my-post.md                      # Date from frontmatter only
```

Posts are ordered by date (newest first by default).

### Pages

Content **without a date**. Pages generate standalone HTML files accessible by direct link or menu. They do not appear in feeds, search, or archive.

```
content/about.md
content/contact.md
content/privacy-policy.md
```

### Fragment Files

Files prefixed with `_` inject content into template regions. They do not generate their own pages.

**Listing page fragments** (support markdown, HTML, and Tera templating):

| File | Template Variable | Purpose |
|------|-------------------|---------|
| `_announce.md` | `{{ announce }}` | Announcement banner at top of listing pages |
| `_header.md` | `{{ header }}` | Custom header content |
| `_hero.md` | `{{ hero }}` | Hero section on homepage |
| `_sidebar.md` | `{{ sidebar }}` | Sidebar content |
| `_footer.md` | `{{ footer }}` | Custom footer content |

**Content page fragments** (static markdown/HTML only, no Tera):

| File | Purpose |
|------|---------|
| `_markdown_header.md` | Prepended to every content file before rendering |
| `_markdown_footer.md` | Appended to every content file before rendering |

**Template-rendered fragments:**

| File | Template Variable | Purpose |
|------|-------------------|---------|
| `_comments.md` | `{{ comments }}` | Comments section (supports Tera) |

**Global fragments:**

| File | Purpose |
|------|---------|
| `_references.md` | Appended to all markdown - reusable link references and footnotes |
| `_htmlhead.md` | Raw HTML injected into `<head>` (analytics, extra styles) |
| `_htmltail.md` | Raw HTML injected at page bottom |
| `_404.md` | Custom 404 error page |

Example `_references.md`:
```markdown
[Github]: https://github.com/myuser/myrepo
[docs]: <./tag-docs.html> "Documentation"

[^credits]: Built with Marmite
```

Then use `[Github]` or `[^credits]` in any content file.

Example `_hero.md` (with Tera templating):
```markdown
>>>
Welcome to {{ site.name }}! We write about technology and open source.
>>>
```

Example `_sidebar.md` (with Tera templating):
```html
<nav>
  <h4>Tags</h4>
  {% set tags = group(kind="tag", ord="desc", items=10) %}
  {% for name, posts in tags %}
    <a href="{{ url_for(path='tag-' ~ name ~ '.html') }}">{{ name }} ({{ posts | length }})</a>
  {% endfor %}
</nav>
```

Example `_htmlhead.md`:
```html
<script defer src="https://analytics.example.com/script.js" data-website-id="abc123"></script>
```

## Taxonomy

### Tags

Assign multiple tags per post for topical grouping:

```yaml
---
tags: rust, web, tutorial
---
```

Or as an array:
```yaml
---
tags:
  - rust
  - web
  - tutorial
---
```

Generated pages:
- `/tags.html` - all tags with post counts
- `/tag-rust.html` - posts tagged "rust" (paginated)
- `/tag-rust.rss` - RSS feed for tag

### Authors

Assign one or more authors:

```yaml
---
authors: alice, bob
---
```

Configure author profiles in `marmite.yaml`:
```yaml
default_author: alice

authors:
  alice:
    name: Alice Smith
    avatar: https://github.com/alice.png
    bio: "Software engineer and writer"
    links:
      - ["GitHub", "https://github.com/alice"]
      - ["Website", "https://alice.dev"]
  bob:
    name: Bob Jones
    avatar: https://github.com/bob.png
```

Generated pages:
- `/authors.html` - all authors
- `/author-alice.html` - posts by alice (paginated)

### Archive

Automatic grouping by year. No configuration needed.

Generated pages:
- `/archive.html` - all years
- `/archive-2024.html` - posts from 2024 (paginated)

### Streams

Content categories - each post belongs to one stream. Think of streams as channels or sections of a site.

```yaml
---
stream: tutorial
---
```

**Key differences from tags:**
- A post has **one** stream vs **many** tags
- Streams provide sequential (prev/next) navigation
- Streams get their own landing pages and feeds
- Posts without a stream go to the default "index" stream

**Stream detection from filename:**

```
tutorial-2024-01-01-getting-started.md    # stream: tutorial
news-2024-01-10-update.md                 # stream: news
guide-S-installation.md                    # stream: guide (S-pattern for undated)
2024-01-05-general-post.md                # stream: index (default)
```

Filename patterns:
- `{stream}-{YYYY-MM-DD}-{slug}.md` - stream name must be a single word
- `{stream}-S-{slug}.md` - for content without date in filename

Priority: frontmatter `stream` > filename pattern > default "index".

Configure display names:
```yaml
streams:
  tutorial:
    display_name: "Tutorials"
  news:
    display_name: "Latest News"
```

Generated pages:
- `/streams.html` - all streams
- `/tutorial.html` - tutorial stream (paginated)
- `/tutorial.rss` - RSS feed for stream

**Draft stream:** Setting `stream: draft` hides the post from main feeds and search while keeping it accessible by URL. All drafts are listed at `/draft.html`.

```yaml
---
title: "Work in Progress"
stream: draft
---
```

To publish: remove the `stream: draft` line or change to a different stream.

**Pinned posts:** Pin a post to the top of its stream page:
```yaml
---
stream: news
pinned: true
---
```

**Custom stream templates:** Create `templates/custom_{stream}.html` for stream-specific layouts:
```
templates/
  custom_tutorials.html     # Custom layout for tutorials stream
  custom_news.html          # Custom layout for news stream
```

### Series

Ordered multi-part content - posts grouped chronologically (oldest to newest).

```yaml
---
series: python-tutorial
---
```

When both `series` and `stream` are set, series prev/next navigation takes precedence.

Configure in `marmite.yaml`:
```yaml
series:
  python-tutorial:
    display_name: "Python Tutorial"
    description: "Learn Python from scratch"
```

Generated pages:
- `/series.html` - all series
- `/serie-python-tutorial.html` - series page
- `/serie-python-tutorial.rss` - RSS feed for series

Series names should be lowercase and hyphenated: `python-tutorial`, `web-development-basics`.

## Media and Assets

### Content Media

Place images in `content/media/` (or the path configured by `media_path`):

```markdown
![My Photo](media/photo.jpg)
![Screenshot](media/screenshots/example.png)
```

Marmite copies the media folder to the output and optionally resizes images.

#### Slug-based media subfolders

Media files can be organized in subfolders named after the content's slug:

```
content/
  media/
    my-post/
      banner.jpg       # Auto-discovered as banner image
      card.png         # Auto-discovered as card image
      photo.png        # Referenced via @/ in markdown
  2024-01-15-my-post.md
```

Media can also live inside content subfolders, alongside the markdown files:

```
content/
  my-post/
    my-post.md
    pt-meu-post.md     # Translation inherits banner.jpg
    media/
      banner.jpg       # Shared by all files in the subfolder
```

Content subfolder media (`content/{slug}/media/`) takes precedence over global media (`content/media/{slug}/`). A generic `banner.{ext}` or `card.{ext}` without a slug prefix is shared by all `.md` files in the subfolder, so translations automatically inherit the base content's images.

Flat files (`media/{slug}.banner.{ext}`) take precedence over subfolder files for backward compatibility.

#### The `@/` shorthand

Use `@/` in markdown image and link syntax to reference files in the content's media subfolder:

```markdown
![Photo](@/photo.png)          <!-- becomes media/{slug}/photo.png -->
[Download PDF](@/report.pdf)   <!-- becomes media/{slug}/report.pdf -->
```

The replacement targets only `src` and `href` attributes in the rendered HTML, so `@/` in plain text, code blocks, and fragment files (`_` prefixed) is left untouched. The `@/` prefix respects the configured `media_path`.

### Static Assets

Place CSS, JS, fonts, and other assets in `static/`:

```
static/
  custom.css     # Custom styles (loaded automatically by default theme)
  custom.js      # Custom scripts (loaded automatically by default theme)
  favicon.ico
  logo.png
```

### Extra Static Folders

Copy additional folders to the output:
```yaml
extra:
  static_folders:
    - downloads
    - assets
```

### File Mapping

Copy specific files to specific output locations:
```yaml
file_mapping:
  - source: ai/llms.txt
    dest: llms.txt
  - source: static/favicon.ico
    dest: favicon.ico
```

## Organization Strategies

### Blog

```
content/
  pages/
    about.md                          # Page
  posts/
    2024-06-15-my-first-post.md       # Post
    2024-06-20-another-post.md        # Post
  _hero.md                            # Homepage hero
  _comments.md                        # Comments system
  _references.md                      # Shared links
  media/
    banner.jpg
```

```yaml
menu:
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]
  - ["About", "about.html"]
```

### Multi-section Blog

Use subfolders with `frontmatter.yaml` to organize by stream:

```
content/
  tutorials/
    frontmatter.yaml                  # stream: tutorial
    python-basics.md
    advanced-python.md
  news/
    frontmatter.yaml                  # stream: news
    v2-release.md
    roadmap.md
  posts/
    general-thoughts.md               # Default stream
  pages/
    about.md
```

```yaml
streams:
  tutorial:
    display_name: "Tutorials"
  news:
    display_name: "News"

menu:
  - ["Tutorials", "tutorial.html"]
  - ["News", "news.html"]
  - ["Tags", "tags.html"]
  - ["About", "about.html"]
```

### Tutorial Site with Series

```
content/
  2024-01-01-python-part-1.md       # series: python-tutorial
  2024-01-15-python-part-2.md       # series: python-tutorial
  2024-02-01-python-part-3.md       # series: python-tutorial
  2024-03-01-rust-part-1.md         # series: rust-tutorial
  2024-03-15-rust-part-2.md         # series: rust-tutorial
  about.md
```

```yaml
series:
  python-tutorial:
    display_name: "Python from Scratch"
    description: "Complete beginner Python course"
  rust-tutorial:
    display_name: "Rust for Beginners"
    description: "Learn Rust programming"

menu:
  - ["Series", "series.html"]
  - ["Tags", "tags.html"]
  - ["About", "about.html"]
```

### Documentation Site

```
content/
  getting-started.md               # Page
  installation.md                   # Page
  configuration.md                  # Page
  faq.md                           # Page
  2024-06-01-changelog-v2.md       # Post (release notes)
  _sidebar.md                      # Navigation sidebar
  _hero.md                         # Welcome message
```

```yaml
menu:
  - ["Getting Started", "getting-started.html"]
  - ["Installation", "installation.html"]
  - ["Configuration", "configuration.html"]
  - ["FAQ", "faq.html"]
  - ["Changelog", "tags.html"]
```

### Portfolio / Gallery Site

```
content/
  about.md
  contact.md
  2024-01-01-project-alpha.md      # stream: portfolio
  2024-02-01-project-beta.md       # stream: portfolio
  gallery/
    photos/
      img1.jpg
      img2.jpg
```

```yaml
streams:
  portfolio:
    display_name: "My Work"

menu:
  - ["Portfolio", "portfolio.html"]
  - ["About", "about.html"]
  - ["Contact", "contact.html"]
```

## Content Discovery

Marmite generates several discovery mechanisms automatically:

| Output | Content |
|--------|---------|
| `/index.html` | Main post listing (paginated) |
| `/pages.html` | All pages |
| `/tags.html` | All tags with counts |
| `/archive.html` | Posts grouped by year |
| `/authors.html` | All authors |
| `/streams.html` | All streams |
| `/series.html` | All series |
| `/search.html` | Full-text search (when enabled) |
| `/sitemap.xml` | XML sitemap for search engines |
| `/urls.json` | All URLs (for tooling) |
| `/index.rss` | Main RSS feed |
| `/index.json` | Main JSON feed (when enabled) |

Per-taxonomy feeds: `/tag-{name}.rss`, `/author-{name}.rss`, `/{stream}.rss`, `/serie-{name}.rss`.

## Multilingual Content (Language Streams)

Languages are auto-detected from content. Just set `language: xx` in frontmatter or use subfolder naming conventions. Optionally set display names in `marmite.yaml`:

```yaml
language: pt
languages:
  pt:
    display_name: "Portugues"
  en:
    display_name: "English"
```

### Subfolder auto-discovery

Group translations in subfolders. A subfolder is recognized as a translation group when it contains exactly one non-prefixed file (the original) and one or more files prefixed with an ISO 639-1 language code:

```
content/hello/
  hello.md              # Original (no prefix), slug: hello, stream: index
  pt-ola-mundo.md       # Portuguese, slug: pt-ola-mundo, stream: pt
  es-hola-mundo.md      # Spanish, slug: es-hola-mundo, stream: es
```

Translation groups work at any nesting depth. Each subfolder forms its own independent group:

```
content/poetry/
  love/
    love-poem.md        # Original (no prefix)
    pt-poema-amor.md    # Portuguese (grouped with love-poem, not nature-poem)
  nature/
    nature-poem.md      # Original (separate group from love/)
    pt-poema-natureza.md
```

A subfolder is NOT a translation group when it has multiple non-prefixed files (ambiguous original) or no non-prefixed file at all. In that case, all files are treated as independent content:

```
content/myfolder/
  first-post.md         # Independent post
  second-post.md        # Independent post
  pt-some-post.md       # NOT a translation, just a regular post with slug pt-some-post
```

The subfolder can also have a date prefix (e.g., `content/2026-07-02-hello/`) so files inside inherit the date without needing it in frontmatter.

Translation groups can also have a `frontmatter.yaml` to share defaults across all translations in the group.

Subfolder names must match the original post's resolved slug (not the filename) to be automatically linked.

### Mixed flat + subfolder

Add translations to an existing flat site without moving the original file:

```
content/
  hello.md              # Existing file, slug: hello
  hello/
    pt-ola.md           # Translation, auto-linked to hello
```

### Translates pointer

Each translation points to the original content's slug using `translates:`:

```yaml
language: pt
translates: hello
```

Marmite builds bidirectional links from `translates:` automatically - simpler than maintaining `translations:` lists on every file.

### Frontmatter translation link

Set `language` and `translations` in frontmatter:

```yaml
language: pt
translations:
  - hello
```

When `language` is set without `stream`, the post is automatically assigned to the language's stream. An explicit `stream` always takes precedence.

### Output

All output stays flat: `hello.html`, `pt-ola-mundo.html`, `pt.html` (stream listing).
Translation links and hreflang tags are added automatically to content pages.

## Best Practices

1. **Use subfolders for content types** - put pages in `pages/`, posts in `posts/` or topic subfolders
2. **Use `frontmatter.yaml`** for shared defaults per subfolder - avoids repeating stream, tags, or author fields
3. **Use `_references.md`** for links you repeat across posts (project URLs, documentation links)
4. **Keep streams simple** - 3-5 streams maximum. Use tags for fine-grained categorization
5. **Use series for sequential content** - tutorials, courses, multi-part articles
6. **Use `stream: draft`** for work-in-progress - drafts are accessible by URL but hidden from feeds
7. **Name files after content slugs** - the filename becomes the default slug, making files easy to find
8. **Use translation group subfolders** - group translations of the same content in a shared subfolder
9. **Flat layouts always work** - marmite supports any folder structure, even a single folder of markdown files
