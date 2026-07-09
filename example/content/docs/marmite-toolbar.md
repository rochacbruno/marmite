---
date: 2026-07-09 12:30:00
tags: docs, features, toolbar, server
description: The marmite toolbar is a floating development panel injected during --serve mode that lets you create, edit, move, clone, and delete content directly from the browser.
---

# Marmite Toolbar

The marmite toolbar is a floating development panel that appears when you serve your site with `--serve`. It provides a visual interface for managing content, editing frontmatter, configuring your site, and viewing site stats - all without leaving the browser.

> [!NOTE]
> The toolbar is only injected by the built-in dev server. It does not appear on your deployed site.

## Getting started

Start your site with the built-in server:

```console
$ marmite myblog --serve --watch
```

A small gear icon appears at the top-left corner of every page. Click it to open the toolbar sidebar. Press `Escape` or click the overlay to close it. The toolbar remembers its open/closed state and active tab across page reloads.

## Tabs

The toolbar organizes its features into tabs. On content pages (posts and pages), all tabs are available. On non-content pages (index, tag listings, archive), only the Site, Layout, and Config tabs are shown.

### Info

Displays metadata for the current content page:

- Title, slug, date, description
- Tags, authors, stream, series
- Language, translations, translates
- Pinned status
- Source file path and last modification time

The data comes from the `{slug}.metadata.json` file generated during `--serve` mode.

### Edit

A form for editing the current page's frontmatter. Fields include:

- **Title** and **Slug** - changing the slug redirects you to the new URL after rebuild
- **Description** and **Date** - date uses the browser's native date/time picker
- **Tags**, **Stream**, **Series**, **Authors** - all with autocomplete from existing values
- **Language** - autocomplete from all ISO 639-1 codes
- **Translates** - autocomplete from all existing content slugs
- **Banner Image** and **Card Image**
- **Pinned** and **Comments** toggles
- **Extra** - JSON editor for the `extra` frontmatter field

Click **Save Frontmatter** to write changes. The site rebuilds automatically and the page reloads (or redirects if the slug changed).

The **Edit Content** button is reserved for a future in-browser markdown editor.

### Actions

Content-specific actions for the current page:

#### Add Translation

Select a language code and enter a translated title. Marmite creates the translation file in the correct location:

- If the original is in a flat directory (e.g. `posts/hello.md`), the translation goes to `posts/ola.md` with explicit `translates: hello` and `language: pt` in frontmatter
- If the original is in a slug-named subfolder (e.g. `posts/hello/hello.md`), the translation goes to `posts/hello/pt-ola.md` with a language prefix in the filename (auto-discovered by marmite)

#### Clone / Copy

Creates a full copy of the current content including the markdown body and all frontmatter. Only the title and slug are changed. The `aliases` and `translates` fields are stripped from the clone. Enter a new title and click **Clone Content**.

#### Move / Rename

Rename the file or move it to a different directory. Enter the new filename (e.g. `new-name.md` or `tutorials/new-name.md`). If the content has an explicit `slug:` in frontmatter, the URL does not change.

#### Delete

Deletes the content file after a confirmation prompt. Redirects to the home page.

### Site

Site overview and quick content creation.

#### Stats

Six clickable cards showing counts for posts, pages, tags, streams, authors, and series. Each card links to its corresponding listing page. Below the cards, the last build's rendering time is displayed.

#### Create Content

A form with title and tags fields, plus an expandable **+ Advanced** section with stream, series, language, translates (slug picker), and directory fields. All fields support autocomplete. Two buttons:

- **New Post** - creates a dated post
- **New Page** - creates an undated page

### Layout

Edit site layout and navigation settings. Changes are saved to `marmite.yaml`.

#### Menu

Visual editor for the navigation menu. Each menu item has a label and URL. Drag items up/down with the arrow buttons, remove with the X button, or add new items at the bottom.

#### Titles

Edit section titles used across the site: Pages, Tags, Archives, Authors, Streams, Series, Languages, and Search.

#### Streams, Series, Languages, Authors

Edit display names and metadata for each configured entry:

- **Streams** - set `display_name` for each stream
- **Series** - set `display_name` and `description` for each series
- **Languages** - set `display_name` for each language
- **Authors** - collapsible panels with name, avatar URL, bio, and links (formatted as `Label=URL, Label=URL`)

Each section has a **+** button to add new entries and an **X** button on each row to remove entries. The key field autocompletes from existing values.

### Config

Edit general site configuration fields from `marmite.yaml`:

- **General** - site name, tagline, URL, language, footer, default author, date format
- **Display** - pagination, search, related content, next/prev links
- **Images** - card image, banner image, logo image
- **Paths** - content path, site path, media path
- **Extra** - JSON editor for the `extra` config object

## 404 Page - "Create it!" button

When you navigate to a page that doesn't exist (e.g. `http://localhost:8000/my-new-page.html`), the 404 page shows a **"Page not found - Click here to create it!"** button. Clicking it creates the page using the slug from the URL as the title (converting hyphens to spaces and capitalizing words), then redirects you to the newly created page after the site rebuilds.

## How it works

The toolbar is a self-contained JavaScript and CSS bundle served from `/__marmite__/toolbar.js` and `/__marmite__/toolbar.css`. The server injects `<link>` and `<script>` tags before `</body>` in every HTML response.

All toolbar actions use the [[content-management-api]] endpoints. After any mutation (create, edit, move, clone, delete), the toolbar intercepts the live-reload WebSocket signal to redirect to the correct page instead of blindly reloading the current URL.

The toolbar state (open/closed, active tab) is persisted in `localStorage` so it survives page reloads.

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `Escape` | Close the toolbar panel |
