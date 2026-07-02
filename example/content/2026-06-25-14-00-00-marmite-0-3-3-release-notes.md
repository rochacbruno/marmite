---
title: Marmite 0.3.3 Release Notes
slug: marmite-0-3-3-release-notes
description: "Marmite 0.3.3 adds slug-based media subfolders and the @/ shorthand for per-content media files."
tags: [release-notes, marmite, features, announcement]
author: rochacbruno
stream: draft
date: 2026-06-25 14:00:00
---

## New Features

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

### Tera 2.0 Migration

Marmite now uses Tera 2.0.0 (upgraded from 1.20.1). This is a major version bump of the template engine that brings several improvements:

- **Bracket indexing** - Access map and array values with `value["key"]` and `value[0]` syntax
- **Keyword test arguments** - Tests like `is defined` now accept keyword arguments
- **Optional chaining** - Use `value?.nested?.field` to safely access nested values without errors

**Backward compatibility:** Existing templates continue to work without changes. Marmite preprocesses templates at load time, automatically converting old Tera 1.x syntax to the Tera 2.0 equivalents. Shortcode files are also preprocessed transparently.

**Compatibility filters:** Tera 2.0 removed or renamed some built-in filters. Marmite provides drop-in replacements for `striptags`, `slice`, `trim_start_matches`, and `date`, so templates using these filters work as before.
