---
tags: docs, features, seo
description: Validate internal links at build time to catch broken references before deployment, with configurable warning and strict failure modes.
---

# Internal Link Validation

Marmite can check all internal links in your content at build time and warn you about broken references. This helps catch issues like typos in URLs, references to renamed content, or links to deleted pages before they reach your readers.

## Enabling Link Checking

Add `check_internal_links: true` to your `marmite.yaml`:

```yaml
check_internal_links: true
```

Or pass it as a CLI flag:

```console
$ marmite mysite --check-internal-links true
```

When enabled, marmite validates every internal link found in your content after all posts and pages are processed. Any link pointing to a slug that does not exist in the generated output produces a warning:

```
WARN: Broken internal link in "my-post.html": "nonexistent-page.html" does not exist
WARN: Found 1 broken internal link(s)
```

The build still succeeds, so you can review and fix the issues at your own pace.

## Strict Mode

For CI/CD pipelines or when you want to enforce link integrity, enable strict mode:

```yaml
check_internal_links: true
strict_internal_links: true
```

With strict mode, the build fails (exits with a non-zero code) when broken internal links are found. This prevents deploying a site with broken references.

## What Gets Checked

Marmite checks links to internal `.html` pages. The following are validated:

- Links between posts (e.g., `[related post](other-post.html)`)
- Links from posts to pages (e.g., `[about](about.html)`)
- Links with anchors (e.g., `[section](post.html#heading)`) - the target page is checked, but the specific heading anchor is not validated

The following are excluded from checking:

- External links (`http://`, `https://`, `mailto:`)
- Anchor-only links (`#section`)
- Links to media files (images, PDFs, etc.)

## What Counts as Valid

A link target is valid if it matches any URL that marmite generates, including:

- Post and page slugs
- Tag pages (`tag-rust.html`)
- Author pages (`author-alice.html`)
- Stream pages (`news.html`)
- Series pages (`series-tutorial.html`)
- Archive pages (`archive-2024.html`)
- Pagination pages
- Redirect alias pages (if using the `aliases` frontmatter field)

## Configuration Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `check_internal_links` | bool | `false` | Enable internal link checking at build time |
| `strict_internal_links` | bool | `false` | Fail the build when broken links are found |

Both fields can be set in `marmite.yaml` or overridden via CLI flags (`--check-internal-links`, `--strict-internal-links`).
