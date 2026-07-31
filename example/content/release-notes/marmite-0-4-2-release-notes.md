---
date: 2026-07-31
title: Marmite 0.4.2 Release Notes
slug: marmite-0-4-2-release-notes
description: "Marmite 0.4.2 adds media file link validation and fixes broken TOC anchor links."
tags: [release-notes, marmite, features]
author: rochacbruno
---

## New Features

### Media file link validation

Marmite can now validate links to media files (images, PDFs, videos, archives, etc.) at build time. Previously, only internal `.html` links were checked by `check_internal_links`. Media references like `![photo](media/pic.jpg)` or `[download](media/report.pdf)` were silently ignored, so a typo or missing file would only be caught by visiting the page.

Enable it with:

```yaml
check_media_links: true
```

Or via the CLI:

```console
$ marmite mysite --check-media-links true
```

When enabled, marmite extracts all `src` and `href` attributes from rendered HTML that point to local files with known media extensions, then checks whether those files actually exist in the content directory (both global `content/media/` and content subfolder media directories).

Broken media links are reported as warnings:

```
WARN  Broken media link in "my-post": "media/missing-photo.jpg" does not exist
WARN  Found 1 broken media link(s)
```

To make broken media links fail the build, combine with `strict_internal_links`:

```yaml
check_media_links: true
strict_internal_links: true
```

Supported file types include images (jpg, png, gif, webp, svg, avif, bmp, tiff, ico), documents (pdf, doc, docx, xls, xlsx, ppt, pptx, txt, csv), data files (json, xml, yaml, yml, toml), audio (mp3, wav, ogg, flac), video (mp4, mov, avi, mkv, webm), and archives (zip, tar, gz, 7z, rar).

## Bug Fixes

### TOC anchors included comrak anchor tag HTML in slugs

Table of Contents anchor links were broken when using comrak's heading ID generation. The TOC regex expected comrak's anchor `<a>` tag to appear before the heading text, but comrak places it after. This caused the anchor tag's attributes to leak into the generated slug, producing links like `#how-it-works-a-href-how-it-works-aria-label-...` instead of the correct `#how-it-works`.

Fixed by capturing the `id` attribute directly from the `<h>` tag and stripping comrak's auto-generated anchor tags from the heading text before building the TOC.
