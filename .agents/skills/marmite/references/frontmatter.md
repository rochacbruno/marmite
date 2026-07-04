# Marmite Frontmatter Reference

Frontmatter is metadata at the top of markdown files that controls how marmite processes and displays content.

## Frontmatter Formats

### YAML (recommended)

```yaml
---
title: "My Post"
date: 2024-06-15
tags: rust, web
---

Content starts here.
```

### TOML

```toml
+++
title = "My Post"
date = "2024-06-15"
+++

Content starts here.
```

### JSON

```json
{
  "title": "My Post",
  "date": "2024-06-15"
}

Content starts here.
```

## Content Types

The presence of a **date** determines the content type:

| Has Date? | Type | Behavior |
|-----------|------|----------|
| Yes | **Post** | Appears in index, feeds, search, archive, pagination |
| No | **Page** | Standalone HTML, accessible only by direct link or menu |

Dates can come from the filename or frontmatter. Filename dates take this form: `2024-06-15-my-post.md` or `2024-06-15-14-30-my-post.md`.

## All Frontmatter Fields

### title

- **Type:** String
- **Default:** Extracted from the first `# Heading` in the markdown, or derived from the filename
- **Purpose:** The title of the post or page

```yaml
title: "Getting Started with Rust"
```

### date

- **Type:** String (date or datetime)
- **Default:** Extracted from the filename pattern `YYYY-MM-DD-*`
- **Purpose:** Publication date. Determines if content is a post (has date) or page (no date)
- **Formats:** `YYYY-MM-DD`, `YYYY-MM-DD HH:MM`, `YYYY-MM-DD HH:MM:SS`, ISO 8601 variants

```yaml
date: 2024-06-15
date: 2024-06-15 14:30
date: 2024-06-15T14:30:00
```

### slug

- **Type:** String
- **Default:** Generated from `title` (slugified) or from the filename (with date prefix removed)
- **Purpose:** The URL path component. The generated file will be `{slug}.html`

```yaml
slug: my-custom-url
```

Slugification converts to lowercase, replaces spaces with hyphens, and removes special characters.

### description

- **Type:** String
- **Default:** Extracted from the first sentence of the content
- **Purpose:** Meta description for SEO, shown in RSS feeds and search results

```yaml
description: "A comprehensive guide to getting started with Rust programming"
```

### tags

- **Type:** String (comma-separated) or Array
- **Purpose:** Categorize content by topic. Generates tag pages and per-tag feeds

```yaml
# Comma-separated string
tags: rust, web, tutorial

# YAML array
tags:
  - rust
  - web
  - tutorial
```

### authors / author

- **Type:** String (comma-separated) or Array
- **Purpose:** Content author(s). Links to author profile pages. `authors` takes precedence over `author`

```yaml
# Single author
author: alice

# Multiple authors
authors: alice, bob

# Array format
authors:
  - alice
  - bob
```

If no author is specified, falls back to the `default_author` config value.

### stream

- **Type:** String
- **Purpose:** Assign content to a stream (category). Creates stream index pages and feeds

```yaml
stream: tutorial
```

Special stream `draft` excludes the post from main feeds and search while keeping it accessible by URL. Useful for work-in-progress content.

Streams can also be set via filename prefix: `tutorial-2024-06-15-my-post.md` sets the stream to `tutorial`.

### series

- **Type:** String
- **Purpose:** Group posts into an ordered multi-part series. Posts in a series get automatic prev/next navigation

```yaml
series: python-tutorial
```

Posts within a series are ordered oldest to newest. When both `series` and `stream` are set, series navigation takes precedence for prev/next links.

### pinned

- **Type:** Boolean
- **Default:** `false`
- **Purpose:** Pin content to the top of its stream page

```yaml
pinned: true
```

### toc

- **Type:** Boolean
- **Default:** Inherits from site config `toc` field
- **Purpose:** Generate a table of contents from headings

```yaml
toc: true
```

### comments

- **Type:** Boolean
- **Default:** `true` (if comments are configured site-wide)
- **Purpose:** Enable or disable comments on this specific post

```yaml
comments: false
```

### card_image

- **Type:** String
- **Purpose:** Image URL for Open Graph / social media cards

```yaml
card_image: media/social-preview.jpg
card_image: https://example.com/image.jpg
```

### banner_image

- **Type:** String
- **Purpose:** Featured image displayed at the top of the post

```yaml
banner_image: media/hero-banner.jpg
```

If not specified, marmite may auto-detect an image from the content or use the `image_provider` config.

### aliases

- **Type:** String (comma-separated) or Array
- **Default:** Empty
- **Purpose:** Generate redirect pages at old URLs that point to this content. Useful when renaming slugs to preserve old links

```yaml
# Comma-separated string
aliases: old-post-url, legacy-path

# YAML array
aliases:
  - old-post-url
  - legacy-path
```

Each alias generates a lightweight HTML file (e.g. `old-post-url.html`) with a `<meta http-equiv="refresh">` redirect, a `<link rel="canonical">` tag, and a JavaScript fallback pointing to the current content URL. Redirect pages are excluded from the sitemap, feeds, and search index.

A warning is logged if an alias conflicts with an existing content slug or is defined by more than one content file.

### language

- **Type:** String
- **Purpose:** Explicitly set the content's language code (e.g., `en`, `pt`)

```yaml
language: pt
```

When set, overrides automatic language detection. Used with the `languages` config to enable multilingual features (translation links, hreflang SEO tags). If not set, the language is inferred from the stream name when it matches a configured language code.

When `language` is set but no `stream` is specified, marmite automatically assigns the post to the language's stream. A post with `language: pt` (and no `stream`) will appear on `pt.html` with a `pt-` slug prefix. An explicit `stream` always takes precedence.

### translations

- **Type:** Array of strings (slugs)
- **Purpose:** Manually link content to its translations in other languages

```yaml
translations:
  - en-hello-world
  - es-hola-mundo
```

Each entry is the slug of a translated version of this content. Marmite resolves the slug to the actual content, fills in the language code and display name from the `languages` config, and creates bidirectional links (if A lists B, B also gets A). Translation links appear in templates as "Also available in: ..." and as `<link rel="alternate" hreflang="...">` tags.

When using subfolder-based content organization, translations are auto-discovered and this field is not needed.

### translates

- **Type:** String (a slug)
- **Purpose:** Points to the slug of the "original" content that this item translates. Marmite automatically creates bidirectional translation links between the source and target.

```yaml
translates: hello
```

This is an alternative to listing all translations in the `translations` field. Instead of maintaining a full list of translated slugs on every content file, you only set `translates` on each translation to point back to the original. Marmite then builds the complete cross-link network automatically.

For example, given an English post with slug `hello` and a Portuguese translation with `language: pt` and `translates: hello`, marmite will link both posts to each other as translations - the English post will show a link to the Portuguese version and vice versa.

### extra

- **Type:** Object (key-value map)
- **Purpose:** Arbitrary custom data accessible in templates via `content.extra`

```yaml
extra:
  math: true
  mermaid: true
  mermaid_theme: dark
  custom_field: "any value"
```

Access in templates: `{{ content.extra.math }}`, `{{ content.extra.custom_field }}`.

Common extra fields:
- `math: true` - Enable KaTeX math rendering for this post
- `mermaid: true` - Enable Mermaid diagram rendering
- `mermaid_theme: dark` - Mermaid theme variant

## Date Detection from Filenames

Marmite extracts dates from filenames automatically:

| Filename Pattern | Detected Date |
|-----------------|---------------|
| `2024-06-15-my-post.md` | 2024-06-15 00:00:00 |
| `2024-06-15-14-30-my-post.md` | 2024-06-15 14:30:00 |
| `2024-06-15T14:30-my-post.md` | 2024-06-15 14:30:00 |
| `about.md` | None (becomes a page) |

Frontmatter `date` overrides the filename date if both are present.

## Stream Detection from Filenames

Streams can be set via filename prefix instead of frontmatter:

| Filename | Stream |
|----------|--------|
| `tutorial-2024-06-15-my-post.md` | `tutorial` |
| `news-2024-06-15-update.md` | `news` |
| `draft-my-page.md` | `draft` |

The stream prefix is removed from the slug.

## Slug Generation Priority

The slug is determined in this order:

1. Explicit `slug` field in frontmatter (slugified)
2. `title` field in frontmatter (slugified)
3. Filename with date and stream prefixes removed (slugified)

## No-Frontmatter Content

Files without any frontmatter are still processed:
- Title is extracted from the first `# Heading`
- Date is extracted from the filename (if present)
- Slug is derived from the filename
- All other fields use defaults

```markdown
# My Simple Post

This file has no frontmatter. The title comes from the heading above.
```

## Folder-Level Frontmatter Defaults

A `frontmatter.yaml` file in a content subfolder provides default values for all `.md` files in that folder. The `content/` root can also have one for site-wide defaults.

```yaml
# content/python/frontmatter.yaml
date: 2026-01-01
stream: python
tags:
  - python
  - programming
```

All markdown files in `content/python/` inherit these values. Per-file frontmatter overrides the defaults. `title` and `slug` are never inherited from folder defaults.

**Merge priority** (lowest to highest):

1. Root `content/frontmatter.yaml`
2. Subfolder `content/{folder}/frontmatter.yaml`
3. Filename conventions (date, stream, language)
4. Per-file frontmatter

**Subfolder gating**: Files in subfolders are only rendered when the subfolder has a `frontmatter.yaml`, is named `pages`, or is a translation group. Other subfolders are ignored.

## Fragment Files

Files prefixed with `_` are treated as layout fragments, not regular content:

| File | Injected Into |
|------|---------------|
| `_hero.md` | `{{ hero }}` template variable |
| `_announce.md` | `{{ announce }}` template variable |
| `_header.md` | `{{ header }}` template variable |
| `_footer.md` | `{{ footer }}` template variable |
| `_sidebar.md` | `{{ sidebar }}` template variable |
| `_comments.md` | `{{ comments }}` template variable |
| `_references.md` | Appended to every markdown file before processing |
| `_htmlhead.md` | `{{ htmlhead }}` - injected into HTML `<head>` |
| `_markdown_header.md` | Prepended to every markdown file |
| `_markdown_footer.md` | Appended to every markdown file |
| `_404.md` | Custom 404 error page |

Fragment files do not generate their own HTML pages.
