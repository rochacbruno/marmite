---
date: 2026-07-10 10:00:00
tags: docs, features, editor, toolbar, server
description: The marmite editor is a full-featured markdown editor with live preview, metadata management, and file editing built into the dev server.
---

# Marmite Editor

The marmite editor is a three-panel content editor that runs inside the built-in dev server. It provides a CodeMirror 6 markdown editor with autocomplete, a live preview iframe, and a sidebar for metadata and file management.

> [!NOTE]
> The editor is only available when serving with `--serve`. It does not appear on your deployed site.

## Opening the editor

Start your site with the built-in server:

```console
$ marmite myblog --serve --watch
```

Open the toolbar (gear icon at top-left) and click the **Editor** button in the header. On content pages, this opens the editor for that specific post or page. On non-content pages (index, tag listings), it opens the editor with a file tree for navigation.

You can also navigate directly to `http://localhost:8000/__marmite__/editor/my-post` to edit a specific slug, or `http://localhost:8000/__marmite__/editor/` for the file browser.

## Editor layout

The editor has three panels:

- **Sidebar** (left, collapsible) - metadata, file tree, and help reference
- **Editor** (center) - CodeMirror 6 markdown editor with a toolbar above it
- **Preview** (right, toggle-able) - live preview of the rendered page

### Top bar

The top bar contains:

- **Sidebar** / **Back** - toggle sidebar visibility or return to the rendered page
- **New** - create a new post or page
- **Version** - current marmite version
- **Content type** - shows POST or PAGE badge
- **Content title** - the title of the content being edited
- **Config** - open the site configuration dialog
- **Theme** - toggle light/dark editor theme
- **Preview** - show/hide the preview panel

### Editor toolbar

Above the editor area:

- **Insert** menu - headings, bold, italic, links, images, code blocks, tables, lists, blockquotes, horizontal rules, and a media file picker
- **Save** - save to disk (also triggered by Ctrl+S)
- **Save As** - save as a new content file with a different title

### Sidebar tabs

- **Info** - content metadata (title, slug, date, tags, etc.) and a project file tree. Content files link to the editor. Non-content editable files (CSS, JS, YAML, fragments) open in a floating modal editor.
- **Edit** - frontmatter fields with autocomplete for tags, streams, series, authors, languages, images
- **Actions** - add translations, clone content, delete content
- **Help** - markdown syntax reference and available shortcodes list

### Status bar

Below the editor area, shows cursor position (line/column), font size slider, and editor theme selector with 12 themes.

## Auto-save

The editor auto-saves after 1.5 seconds of inactivity. This writes the file to disk, triggers a rebuild (when running with `--watch`), and refreshes the preview panel. A localStorage backup is also kept for crash recovery - if the browser closes unexpectedly, the editor offers to restore the draft on next load.

## Preview

The preview panel shows the rendered page in an iframe. It refreshes automatically after each save/rebuild via a dedicated WebSocket connection. The editor page itself never reloads during saves.

The preview panel can be hidden with the **Preview** button in the top bar. When browsing around in the preview and then clicking back into the editor, the preview automatically navigates back to the page being edited.

Use **Pop Out** to open the preview in a separate browser tab, or **Refresh** to manually reload it.

## Raw file editing

The editor supports editing non-content files without adding frontmatter:

- **Fragment files** (`_hero.md`, `_comments.md`, etc.) - open from the file tree or navigate to `/__marmite__/editor/content/_hero.md`
- **Config, CSS, JS, YAML, and other text files** - click them in the file tree to open a floating modal editor with a save button, or navigate directly via URL for full-screen editing

In raw mode, the editor skips frontmatter handling, hides the preview panel and Edit/Actions sidebar tabs.

## Configuration dialog

Click **Config** in the top bar to open the site configuration dialog with 9 tabs:

- **Site** - name, tagline, URL, language, footer, logo
- **Content** - pagination, default author, date format, TOC, next/prev links, related content
- **Search** - enable search, match snippets, snippets per result, search title
- **Feeds** - JSON feed, sitemap, URLs JSON, publish markdown, shortcodes
- **Appearance** - colorscheme (13 options), colorscheme picker, color mode, light/dark toggle, code highlighting
- **Images** - card image, banner image, skip resize, max width, banner width, resize filter
- **Menu** - add, remove, and reorder navigation menu items
- **Paths** - content, site, and media directory paths
- **Raw YAML** - edit the full marmite.yaml directly

## Disabling the toolbar

To serve without the toolbar and editor:

```yaml
# marmite.yaml
enable_toolbar: false
```

Or via CLI:

```console
$ marmite myblog --serve --enable-toolbar false
```

The API endpoints and editor page still work when the toolbar is disabled - only the automatic injection into served HTML pages is suppressed.

## API endpoints

The editor uses these API endpoints (all under `/__marmite__/`):

| Endpoint | Method | Description |
|----------|--------|-------------|
| `content/{slug}/body` | GET | Read frontmatter and raw markdown body |
| `content/{slug}/body` | PUT | Write markdown body with optional frontmatter updates |
| `file/{path}` | GET | Read any text file in the project |
| `file/{path}` | PUT | Write any text file in the project |
| `files` | GET | List all project files with editable/fragment flags |
| `config` | GET | Read raw marmite.yaml content |
| `config` | PUT | Write raw marmite.yaml (with YAML validation) |
| `editor/{slug}` | GET | Serve the editor page for a content slug |
| `editor/{path}` | GET | Serve the editor in raw mode for a file path |
| `editor/` | GET | Serve the empty editor with file tree navigation |
