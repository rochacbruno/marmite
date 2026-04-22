---
title: Marmite 0.3.0 Release Notes
slug: marmite-0-3-0-release-notes
description: "Marmite 0.3.0 brings build-time syntax highlighting with Arborium, enhanced search with inline match previews, improved font rendering with Atkinson Hyperlegible Next, and several bug fixes."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
pinned: true
---

We're excited to announce Marmite 0.3.0! This release brings a major improvement to code rendering with build-time syntax highlighting powered by Arborium, an enhanced search experience with inline match previews, an updated default font, and several important bug fixes.

## 🎉 New Features

### Build-Time Syntax Highlighting with Arborium

Marmite now uses [Arborium](https://docs.rs/arborium) for syntax highlighting at build time instead of relying on client-side JavaScript (highlight.js). (#425)

- **Faster page loads** — no JavaScript needed for syntax highlighting
- **Tree-sitter based** — accurate, language-aware highlighting for 100+ languages
- **Works without JavaScript** — highlighted code renders immediately
- **Configurable** — enable/disable per site, choose themes

Code blocks in your markdown are automatically highlighted during site generation. No configuration changes needed — it just works.

Thanks to **Ian Wagner** for this contribution!

### Search Match Previews

The search box now displays inline match previews directly in the search results, making it easier to find exactly what you're looking for.

- Shows up to 3 match snippets by default when matches are found
- Highlights the matching text within context
- Configurable number of matches to display
- Works with the existing search index

## 🐛 Bug Fixes

### Image Orientation Fix

Fixed an issue where images with EXIF orientation metadata were not displayed correctly during resizing and gallery thumbnail generation. Images now respect their EXIF orientation data. (#430, closes #412)

### Dark/Light Mode Detection at First Load

Fixed a bug where the dark/light color scheme was not correctly detected on the very first page load, causing a flash of the wrong theme. (#416)

Thanks to **Fourflies** for this fix!

### Content Type Handling on Internal Server

Fixed content type detection on the built-in development server. Previously, some file types were served with incorrect MIME types, which could cause browsers to mishandle them. (#427, closes #413)

### Box Shadow Removed from Default Theme

Removed an unintended box shadow artifact from the default theme that appeared across all colorschemes. (#429)

### Code Block Styling

Fixed background color and padding on code blocks to ensure consistent styling with the new build-time syntax highlighting.

## 🎨 Theme & UI Improvements

### Atkinson Hyperlegible Next Font

The default font has been updated from Atkinson Hyperlegible to **Atkinson Hyperlegible Next** (woff2 format), providing improved readability and better font rendering. (#426, closes #423)

## 📦 Dependency Updates

### New Dependencies
- `arborium` 2.16.0 — build-time syntax highlighting via tree-sitter

### Updated Dependencies
- `chrono`: 0.4.42 → 0.4.43
- `clap`: 4.5.53 → 4.5.54
- `indexmap`: 2.12.1 → 2.13.0
- `serde_json`: 1.0.146 → 1.0.149
- `tempfile`: 3.23.0 → 3.24.0
- `url`: 2.5.7 → 2.5.8

## 🛠️ Code Quality

- Pedantic clippy fixes for improved code quality (#428)
- Additional server tests for content type handling

## 🌐 Community

- New showcase entry: Fabiano da Rosa Gomes' personal blog (#424)

## Upgrading

To upgrade to Marmite 0.3.0:

```bash
# If installed via cargo
cargo install marmite --force

# If installed via pip
pip install --upgrade marmite-ssg

# Or use the install script
curl -sSL https://marmite.blog/install.sh | bash
```

### Migration Notes

1. **Syntax highlighting** — Client-side highlight.js is no longer loaded by default. If you have a custom theme that depends on it, you may need to adjust your templates.
2. **Font change** — The default font file changed from `Atkinson-Hyperlegible-Regular-102.woff` to `AtkinsonHyperlegibleNext-Regular.woff2`. Custom themes referencing the old font file should update accordingly.

## Thank You

Thanks to all contributors who made this release possible:

- **Ian Wagner** — Arborium build-time syntax highlighting
- **Fourflies** — Dark/light mode detection fix
- **Fabiano da Rosa Gomes** — Showcase contribution

---

For the complete changelog, see the [GitHub releases page](https://github.com/rochacbruno/marmite/releases/tag/0.3.0).
