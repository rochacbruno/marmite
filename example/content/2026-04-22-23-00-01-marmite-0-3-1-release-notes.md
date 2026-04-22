---
title: Marmite 0.3.1 Release Notes
slug: marmite-0-3-1-release-notes
description: "Marmite 0.3.1 fixes static file handling so user files are merged on top of embedded defaults, with drift warnings for core files."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
pinned: true
stream: draft
---

We're excited to announce Marmite 0.3.1! This release fixes how static files are handled when users provide their own `static/` folder, ensuring embedded and user files are properly merged instead of requiring users to provide a complete replacement.

## 🐛 Bug Fixes

### Static Files Now Merged with Embedded Defaults

Previously, if your project had a `static/` folder (even with just one file like `foo.png`), Marmite would copy only your folder to the output and skip all embedded static files. This meant the site would be missing core assets like `marmite.css`, `pico.min.css`, `search.js`, fonts, and colorschemes — breaking the default theme.

Now, when no theme is set, Marmite always writes the embedded static files first, then copies your `static/` folder on top. Your files override matching embedded files while the rest remain available.

**Before (broken):**
```
blog/static/foo.png  →  output/static/foo.png  (only your file, site breaks)
```

**After (fixed):**
```
blog/static/foo.png  →  output/static/foo.png       (your file)
                         output/static/marmite.css   (from embedded)
                         output/static/pico.min.css  (from embedded)
                         output/static/search.js     (from embedded)
                         output/static/...           (all other embedded files)
```

### Drift Warnings for Core Static Files

When you override a core embedded file (like `marmite.css`, `marmite.js`, `search.js`, `pico.min.css`, fonts, or colorschemes), Marmite now warns you if the embedded version differs from yours. This typically happens after upgrading Marmite, when the embedded file contains fixes or improvements.

```
WARN: Static file 'search.js' differs from the embedded version.
      The embedded version may contain updates or fixes.
      To use the embedded version, remove 'blog/static/search.js' from your static folder.
```

Your customized version is always preserved — the warning is informational so you can decide whether to adopt the updated embedded version.

## 🎨 Theme Behavior

When a theme is set via `theme:` in configuration or `--theme` on the CLI, the behavior is unchanged: the theme's `static/` folder provides a complete set of static files and the embedded files are not used.

## Upgrading

To upgrade to Marmite 0.3.1:

```bash
# If installed via cargo
cargo install marmite --force

# If installed via pip
pip install --upgrade marmite

# Or use the install script
curl -sSL https://marmite.blog/install.sh | bash
```

### Migration Notes

1. **If you relied on `static/` replacing all embedded files** — This is no longer the case. Your `static/` folder is now merged on top of embedded files. If you intentionally replaced a core file, it will still be used, but you'll see a drift warning if it differs from the embedded version.
2. **No action needed for most users** — If you had a `static/` folder with only custom files (images, extra CSS, etc.), your site will now work correctly without needing to copy all embedded files manually.

---

For the complete changelog, see the [GitHub releases page](https://github.com/rochacbruno/marmite/releases/tag/0.3.1).
