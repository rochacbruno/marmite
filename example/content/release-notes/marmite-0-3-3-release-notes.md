---
title: Marmite 0.3.3 Release Notes
slug: marmite-0-3-3-release-notes
description: "Marmite 0.3.3 adds workspace multi-site support, multilingual content via language streams, content subfolder media, internal link validation, redirect aliases, and migrates to Tera 2.0."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
stream: draft
date: 2026-06-25 14:00:00
---

## New Features

### Language Streams - Multilingual Content (#154, #486)

Marmite supports multilingual sites through language streams. Languages are auto-detected from content - no configuration required. Just set `language: pt` in your frontmatter and marmite handles the rest.

Optionally, set display names in `marmite.yaml`:

```yaml
language: en
languages:
  pt:
    display_name: "Portugues"
  es:
    display_name: "Espanol"
```

Content can be organized in subfolder groups for auto-discovery:

```
content/hello/
  hello.md              # Default language (en), stays on index.html
  pt-ola-mundo.md       # Portuguese, generates pt-ola-mundo.html, shows on pt.html
  es-hola-mundo.md      # Spanish, generates es-hola-mundo.html, shows on es.html
```

Each language gets its own stream listing page (`en.html`, `es.html`) and RSS feed. Translation links ("Also available in: Portugues, Espanol") appear automatically on content pages, and `<link rel="alternate" hreflang="...">` tags are added for SEO.

Four content organization modes are supported:
- Subfolder grouping with auto-discovery
- Mixed flat file + subfolder (existing sites can add translations incrementally)
- Frontmatter `translates:` pointer (`translates: original-slug`) - each translation points to the original, marmite builds bidirectional links automatically
- Frontmatter `translations:` list (`translations: [slug1, slug2]`)

Default language content stays on `index.html`. Sites without any language content are completely unaffected.

The `languages:` config key `name` has been renamed to `display_name` (matching `streams:` and `series:` patterns). The old `name` key is still accepted for backward compatibility.

### Languages Group Page

A new `languages.html` group page lists all content organized by language, following the same pattern as tags, authors, archives, streams, and series group pages.

The page is always generated, even on monolingual sites (showing just the default language). Each language entry shows a preview of its content and links to the corresponding stream page (`pt.html`, `es.html`, etc.) or to `index.html` for the default language.

Languages are sorted alphabetically with the site's default language appearing last. Display names from the `languages:` config are used when available, falling back to the two-letter code.

A new `languages_title` config option controls the page heading (default: "Languages"). A `language_display_name` Tera function is also available for custom templates.

### CLI Translation Support and JSON Output

The `--new` command now supports creating translations directly from the CLI and outputs structured JSON instead of a plain file path.

New flags:
- `--lang <CODE>` - Set an ISO 639-1 language code on the new content
- `--translates <SLUG>` - Link the new content as a translation of an existing post (requires `--lang`)

When `--translates` targets a post in a subfolder, the translation is automatically placed in the same folder with a language-code prefix (e.g. `pt-slug.md`), following the subfolder translation convention. When the target is at root level, the translation is placed at root with `translates` frontmatter.

The JSON output includes `file`, `title`, `slug`, `date`, `tags`, `language`, and `translates` fields as applicable, making it easy to pipe into scripts with tools like `jq`.

```bash
marmite myblog --new "Ola Mundo" --lang pt --translates hello-world
# {"file":"myblog/content/hello-world/pt-ola-mundo.md","title":"Ola Mundo","slug":"ola-mundo","language":"pt","translates":"hello-world"}
```

### Content Subfolder Media

Media files can now live inside content subfolders (`content/{slug}/media/`) as an alternative to the global `content/media/{slug}/` location. Content subfolder media takes precedence and is automatically copied to the output.

A generic `banner.jpg` or `card.png` in a content subfolder's media directory is shared by all `.md` files in that subfolder. This is particularly useful with language streams - all translations in a group inherit the same banner image without separate copies or frontmatter overrides.

### Date Extraction from Parent Folder

Content files inside dated subfolders (e.g., `content/2026-07-03-my-post/my-post.md`) now automatically inherit the date from the folder name when no date is set in frontmatter or the filename. This is useful with content subfolders for translations - all files in the folder share the same date without repeating it in every frontmatter.

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

### Workspace Multi-Site Support (#329)

Marmite now supports workspaces for managing multiple sites from a single project directory. A `marmite-workspace.yaml` file at the root identifies a workspace:

```yaml
sites:
  - name: blog
  - name: photos
default_site: blog
defaults:
  language: en
  pagination: 10
```

Key capabilities:
- **Single command builds** - `marmite myworkspace output` builds all sites
- **Configuration inheritance** - workspace defaults flow to all sites, each site's `marmite.yaml` can override
- **Cross-site references** - link between sites using `site::path` syntax (e.g., `photos::gallery.html` resolves to `/photos/gallery.html`)
- **Flexible root handling** - default site renders at root, or use `redirect: true` for a redirect page
- **Watch mode** - monitors all site directories with live reload
- **CLI integration** - `--show-urls` and `--shortcodes` aggregate across sites, `--new --site blog` creates content in a specific site
- **Independent sites** - each subfolder remains a fully independent marmite site that can be built on its own

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

### Folder-Level Frontmatter Defaults (#487)

Content subfolders can now have a `frontmatter.yaml` file that provides default frontmatter values for all `.md` files in that folder. This eliminates repetitive metadata when multiple posts share the same stream, tags, date, authors, or extra fields.

```
content/python/
  frontmatter.yaml    # date, stream, tags defaults
  databases.md        # inherits defaults, only needs title
  classes.md          # can override any field
```

The `content/` root can also have a `frontmatter.yaml` for site-wide defaults. Defaults layer at any nesting depth: root first, then each subfolder level, then filename conventions, then per-file frontmatter on top. Files in nested subfolders without their own `frontmatter.yaml` inherit from the nearest ancestor that has one.

Subfolders without a `frontmatter.yaml` continue to work as before - their content is rendered normally.

Translation groups now work correctly at any nesting depth. Each subfolder forms its own independent group, so `content/poetry/love/` and `content/poetry/nature/` are treated as separate translation groups rather than being lumped together.

See the [Folder-Level Frontmatter Defaults](folder-level-frontmatter-defaults.html) guide for details.
