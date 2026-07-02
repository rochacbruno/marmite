---
title: Marmite 0.3.3 Release Notes
slug: marmite-0-3-3-release-notes
description: "Marmite 0.3.3 adds slug-based media subfolders, @/ shorthand for media files, and migrates to Tera 2.0 with new template features."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
stream: draft
date: 2026-06-25 14:00:00
---

## New Features

### Language Streams - Multilingual Content (#154)

Marmite now supports multilingual sites through language streams. Configure available languages in `marmite.yaml`:

```yaml
language: pt
languages:
  pt:
    name: "Portugues"
  en:
    name: "English"
  es:
    name: "Espanol"
```

Content can be organized in subfolder groups for auto-discovery:

```
content/hello/
  hello.md              # Default language (pt), stays on index.html
  en-hello-world.md     # English, generates en-hello-world.html
  es-hola-mundo.md      # Spanish, generates es-hola-mundo.html
```

Each language gets its own stream listing page (`en.html`, `es.html`) and RSS feed. Translation links ("Also available in: English, Espanol") appear automatically on content pages, and `<link rel="alternate" hreflang="...">` tags are added for SEO.

Four content organization modes are supported:
- Subfolder grouping with auto-discovery
- Mixed flat file + subfolder (existing sites can add translations incrementally)
- Flat files with existing stream markers (`pt-S-ola.md`)
- Frontmatter-only (`translations: [slug1, slug2]`)

Default language content stays on `index.html`. Sites without `languages` configured are completely unaffected.

### Build-time Internal Link Validation (#473)

Marmite can now check internal links at build time and warn about broken ones. Enable with:

```yaml
check_internal_links: true
strict_internal_links: false  # set to true to fail the build on broken links
```

When enabled, marmite compares all internal links found in content against the known output URLs after processing. Broken links are reported as warnings. With `strict_internal_links: true`, the build fails if any broken links are found.

Both options are also available as CLI flags: `--check-internal-links true` and `--strict-internal-links true`.

### Media Organization with Slug-Based Subfolders (#149)

Media files can now be organized in subfolders named after the content's slug. Marmite automatically discovers `banner.{ext}` and `card.{ext}` files inside `media/{slug}/` directories.

```
content/
  media/
    my-post/
      banner.jpg       # Auto-discovered as banner image
      card.png         # Auto-discovered as card image
      photo.png        # Referenced via @/ in markdown
  2024-01-15-my-post.md
```

Flat files (`media/my-post.banner.jpg`) still take precedence, so existing sites are unaffected.

### `@/` Shorthand for Media References (#149)

Use `@/` in markdown image and link syntax to reference files in the content's media subfolder:

```markdown
![Photo](@/photo.png)
[Download PDF](@/report.pdf)
```

Marmite replaces `@/` with `media/{slug}/` in the rendered HTML. The replacement only targets `src` and `href` attributes, so `@/` in plain text, code blocks, and fragment files is left untouched.

See the [Media Organization](media-organization.html) guide for details.

### Redirect Aliases (#472)

Content can now define redirect aliases in frontmatter. When a post or page slug changes, the old URL can be preserved as a redirect:

```yaml
---
title: My Renamed Post
slug: my-renamed-post
aliases:
  - old-post-url
  - another-old-url
---
```

For each alias, marmite generates a lightweight HTML file with a `<meta http-equiv="refresh">` redirect, a `<link rel="canonical">` tag, and a JavaScript fallback. Redirect pages are excluded from the sitemap, feeds, and search index to avoid duplicate content issues.

Marmite warns at build time if an alias conflicts with an existing content slug or is defined by more than one content file.

### Tera 2.0 Migration

Marmite now uses Tera 2.0.0 (upgraded from 1.20.1). This is a major version bump of the template engine with new syntax features, better error messages, and 2-4x faster rendering.

#### New template syntax

The default templates now use these Tera 2.0 features:

- **Native array slicing** - `content.tags[:3]` instead of `content.tags | slice(end=3)`
- **Ternary expressions** - `name if name else gallery_data.name` instead of if/else blocks
- **Map literals and spread** - `{...card, "title": title if title else card.title}` for merging defaults with overrides
- **Optional chaining** - `site?.atproto?.publication_uri` instead of `site.atproto and site.atproto.publication_uri`
- **Bracket indexing** - `item[0]` instead of `item.0`
- **Keyword test arguments** - `is starting_with(pat="http")` instead of `is starting_with("http")`

#### `{% shortcode %}` syntax for shortcode definitions

Shortcode HTML files now use `{% shortcode %}` / `{% endshortcode %}` instead of `{% macro %}` / `{% endmacro %}`:

```html
{# shortcodes/alert.html #}
{% shortcode alert(type="info", message="") %}
<div class="alert alert-{{ type }}">{{ message }}</div>
{% endshortcode alert %}
```

The `{% macro %}` syntax still works for backward compatibility. Shortcode bodies have full access to the rendering context (`site_data`, `url_for`, `content`, etc.).

#### Backward compatibility

Existing templates and shortcodes continue to work without changes. Marmite preprocesses templates at load time, automatically converting old Tera 1.x syntax (dot indexing, positional test args, `ignore missing` includes) to Tera 2.0 equivalents.

Tera 2.0 removed or renamed some built-in filters. Marmite provides drop-in replacements for `striptags`, `slice`, `trim_start_matches`, and `date`, so templates using these filters work as before.
