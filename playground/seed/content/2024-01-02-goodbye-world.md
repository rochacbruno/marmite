---
title: Shortcodes and Advanced Features
tags: guide, shortcodes
date: 2024-01-02
---

Marmite includes built-in **shortcodes** - reusable components you can embed
in your Markdown content.

## Available Shortcodes

Use shortcodes with HTML comment syntax: `<!-- .shortcode_name param=value -->`

### YouTube Videos

Embed a YouTube video:

<!-- .youtube id="dQw4w9WgXcQ" -->

### Lists

Show recent posts:

<!-- .posts -->

### Table of Contents

Insert a table of contents anywhere in your content:

<!-- .toc -->

## Streams and Series

Organize related content with **streams** (categories) and **series**
(ordered multi-part content).

Add to your frontmatter:

```yaml
---
title: Part 1
stream: tutorials
series: getting-started
series_order: 1
---
```

Configure them in `marmite.yaml`:

```yaml
streams:
  tutorials:
    display_name: Tutorials

series:
  getting-started:
    display_name: Getting Started
    description: A beginner's guide
```

## Wiki-style Links

Link to other content using double brackets: [[about]]

Marmite automatically resolves the slug and tracks backlinks between pages.

## Alerts

> [!WARNING]
> This playground session expires after 1 hour of inactivity. Make sure to
> copy any content you want to keep.

> [!IMPORTANT]
> Shortcodes are enabled by default. Set `enable_shortcodes: false` in
> marmite.yaml to disable them.
