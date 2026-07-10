---
title: Marmite 0.4.1 Release Notes
slug: marmite-0-4-1-release-notes
description: "Marmite 0.4.1 adds a content editor, development toolbar, content management API, smart directory auto-detection, and duplicate slug detection."
tags: [release-notes, marmite, features]
author: rochacbruno
stream: draft
---

## New Features

### Content Editor

Marmite now includes a full-featured content editor accessible from the toolbar. Click the **Editor** button in the toolbar header to open a three-panel editing environment with a CodeMirror 6 markdown editor, live preview, and metadata sidebar.

Key features:

- **CodeMirror 6 editor** with markdown syntax highlighting, 12 editor themes, and adjustable font size
- **Autocomplete** for wikilinks, shortcodes, media paths, and frontmatter keys/values
- **Auto-save** after 1.5 seconds of inactivity, with automatic preview refresh via WebSocket
- **Insert menu** for headings, bold, italic, links, images, code blocks, tables, and a visual media file picker
- **Metadata sidebar** with frontmatter editing, content actions (translate, clone, delete), project file tree, and markdown/shortcode help reference
- **Config dialog** with 9 tabs covering all marmite.yaml options, including raw YAML editing
- **Raw file editing** for fragments (`_hero.md`), CSS, JS, YAML, and other non-content files without frontmatter handling
- **File tree navigation** at `/__marmite__/editor/` for browsing and opening project files
- **Save As** for creating new content from the editor
- **`enable_toolbar`** config option and `--enable-toolbar` CLI flag to disable toolbar injection

New API endpoints: `GET/PUT /__marmite__/content/{slug}/body`, `GET/PUT /__marmite__/file/{path}`, `GET/PUT /__marmite__/config`, `GET /__marmite__/files`.

See [[marmite-editor]] for full documentation.

### Development Toolbar

Marmite now includes a floating development toolbar that appears when running with `--serve`. Click the gear icon at the top-left corner to open a sidebar panel with tabs for managing your site directly from the browser.

The toolbar provides:

- **Info** - view content metadata (title, slug, date, tags, stream, series, source path)
- **Edit** - edit frontmatter fields with autocomplete for tags, streams, series, authors, and languages. Includes native date picker, slug editing with auto-redirect, and image path fields
- **Actions** - add translations, clone content (full file copy), move/rename files, delete content
- **Site** - stats dashboard with clickable cards (posts, pages, tags, streams, authors, series, render time), and a content creation form with advanced options
- **Layout** - visual menu editor (add, remove, reorder items), section title editor, and display name management for streams, series, languages, and author profiles
- **Config** - edit general site configuration (name, tagline, URL, pagination, search, images, paths, extra)
- **404 "Create it!" button** - when visiting a nonexistent page, a button appears to create it with one click

The toolbar state (open/closed, active tab) is saved in localStorage and persists across page reloads.

See [[marmite-toolbar]] for full documentation.

### Content Management API

The built-in server now exposes a REST API under `/__marmite__/` for programmatic content and configuration management:

- `POST /__marmite__/content` - create new posts and pages
- `PATCH /__marmite__/content/{slug}` - update frontmatter fields
- `POST /__marmite__/content/{slug}/clone` - full-file clone with new title/slug
- `POST /__marmite__/content/{slug}/move` - move or rename content files
- `DELETE /__marmite__/content/{slug}` - delete content
- `POST /__marmite__/config` - create default config
- `PATCH /__marmite__/config` - update config fields
- `GET /__marmite__/data` - aggregated site data (tags, streams, series, authors, slugs, config, build stats)

See [[content-management-api]] for full documentation.

### Smart directory auto-detection for `--new` command

The `--new` CLI command now automatically detects `posts/` and `pages/` subdirectories in structured projects and places content in the right location without requiring the `-d` flag.

```console
$ marmite myblog --new "My Post"
# -> created in content/posts/ (if posts/ exists)

$ marmite myblog --new "About" -p
# -> created in content/pages/ (if pages/ exists)
```

The `-d` flag still works to override auto-detection. Flat projects without `posts/`/`pages/` subdirectories are unaffected.

## Bug Fixes

### Duplicate slug detection in `--new` command

The `--new` command now detects duplicate slugs and uses clean filenames. Previously, creating content with a title that matched an existing slug could overwrite or conflict with existing content. Now it reports an error.

### 404 pages return proper HTTP 404 status

The built-in server was returning HTTP 200 for custom 404 pages. It now correctly returns HTTP 404 status, which is important for link checkers and SEO tools.

### Link checker wikilink awareness

The link checker now properly handles wikilink title resolution, avoiding false positives when checking internal links that use wikilink syntax.

## Other Changes

- Fixed Arch Linux installation instructions
- Added `install` shortcode
- CI: allow CLI overrides, fixed CI for fork PRs
