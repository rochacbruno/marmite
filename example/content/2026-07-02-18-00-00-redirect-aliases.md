---
tags: docs, features, seo
description: Generate redirect pages for old URLs when you rename content slugs, preserving links and SEO with the aliases frontmatter field.
---

# Redirect Aliases

When you rename a post's slug, any existing links to the old URL break. Marmite's redirect aliases feature lets you generate lightweight redirect pages at old URLs so that visitors and search engines are automatically sent to the new location.

## How It Works

Add an `aliases` field to the frontmatter of any post or page. Each alias generates an HTML file at that path containing a redirect to the current content URL.

```yaml
---
title: My Renamed Post
slug: my-renamed-post
date: 2024-06-15
aliases:
  - old-post-url
  - another-old-url
---
```

When marmite builds the site, it generates `old-post-url.html` and `another-old-url.html` in addition to `my-renamed-post.html`. Each alias file redirects to the canonical URL using three mechanisms for maximum compatibility:

1. A `<meta http-equiv="refresh">` tag for browsers that process meta redirects
2. A `<link rel="canonical">` tag for search engines
3. A JavaScript `window.location.href` redirect as a fallback

## Specifying Aliases

Aliases can be specified as a YAML array or as a comma-separated string:

```yaml
# Array format
aliases:
  - old-url
  - legacy-url

# Comma-separated string
aliases: old-url, legacy-url
```

Each alias value is used directly as the filename (with `.html` appended), so use the same format as slugs - lowercase with hyphens.

## Use Cases

### Renaming a post slug

You published a post as `getting-started-with-rust.html` but want to shorten it to `rust-intro.html`:

```yaml
---
title: Introduction to Rust
slug: rust-intro
aliases:
  - getting-started-with-rust
---
```

### Migrating from another static site generator

If you are moving from a tool that used different URL patterns, add the old paths as aliases:

```yaml
---
title: My Post
slug: my-post
aliases:
  - posts/2024/01/my-post
  - blog/my-post
---
```

### Consolidating duplicate content

If you had two pages covering the same topic and want to merge them:

```yaml
---
title: Complete Guide
slug: complete-guide
aliases:
  - quick-start
  - beginner-tutorial
---
```

## Conflict Detection

Marmite warns at build time if:

- An alias matches an existing content slug (the alias is skipped to avoid overwriting the real page)
- Two different content files define the same alias (the duplicate is skipped)

These warnings appear in the build output so you can fix conflicts before deploying.

## SEO Considerations

Redirect alias pages are automatically excluded from:

- **Sitemap** - search engines only see the canonical URLs
- **RSS/JSON feeds** - only real content appears in feeds
- **Search index** - the client-side search does not index redirect pages

The `<link rel="canonical">` tag in each redirect page tells search engines to transfer link equity to the canonical URL, which helps maintain your SEO ranking after a URL change.

## Viewing Redirect URLs

Use the `--show-urls` command to see all generated URLs, including redirects:

```console
$ marmite mysite --show-urls
```

Redirect URLs appear under the `redirects` category in the JSON output.
