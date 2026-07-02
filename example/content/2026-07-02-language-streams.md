---
tags: docs, features, i18n, multilingual
description: Write content in multiple languages with automatic translation linking, language stream pages, and hreflang SEO tags using marmite's language streams feature.
---

# Language Streams - Multilingual Content

Marmite supports multilingual sites through language streams. Each language becomes a stream with its own listing page and RSS feed, while translations are automatically cross-linked with "Also available in" navigation and hreflang SEO tags.

## Configuration

Declare available languages in `marmite.yaml`:

```yaml
language: pt
languages:
  pt:
    name: "Portugues"
  en:
    name: "English"
  es:
    name: "Espanol"
```

The `language` field (which already exists and defaults to `en`) determines the default language. Content in the default language stays on `index.html`. Other languages get their own stream pages (`en.html`, `es.html`) and RSS feeds (`en.rss`, `es.rss`).

When `languages` is not configured, all i18n features are disabled and existing sites behave exactly as before.

## Content Organization

There are four ways to organize multilingual content. All produce flat HTML output.

### Option 1: Subfolder Grouping (Auto-Discovery)

Group translations in a subfolder named after the base content's slug. Files prefixed with a configured language code are automatically detected and cross-linked:

```
content/hello/
  hello.md              # Default language (pt)
  en-hello-world.md     # English translation
  es-hola-mundo.md      # Spanish translation
```

This generates:
- `hello.html` - Portuguese post, listed on `index.html`
- `en-hello-world.html` - English post, listed on `en.html`
- `es-hola-mundo.html` - Spanish post, listed on `es.html`

All three pages automatically show "Also available in" links to each other.

### Option 2: Mixed Flat + Subfolder

If you have an existing flat site and want to add translations without moving original files, create a subfolder matching the existing content's slug:

```
content/
  hello.md              # Existing flat file, slug: hello
  hello/
    pt-ola.md           # Portuguese translation, auto-linked
```

Marmite detects that the subfolder name `hello` matches the flat file's slug and links them as translations.

### Option 3: Flat Files with Stream Markers

Use the existing `-S-` stream marker pattern for flat file organization:

```
content/
  hello.md              # Default language
  pt-S-ola.md           # Portuguese, stream: pt
```

With this pattern, you need to manually link translations using the `translations` frontmatter field (see below).

### Option 4: Frontmatter Only

Set the stream and translations explicitly in frontmatter:

```yaml
---
title: Hello World
date: 2024-01-01
translations:
  - pt-ola
  - es-hola
---
```

The `translations` field accepts a list of slugs. Marmite resolves each slug to the actual content, fills in the language code and display name from the `languages` config, and creates bidirectional links. If post A lists post B as a translation, post B automatically gets a link back to post A.

## Frontmatter Fields

Two new frontmatter fields are available:

### `language`

Explicitly set the content's language code:

```yaml
language: pt
```

Usually not needed - the language is inferred from the stream name or subfolder detection. Use this when you need to override automatic detection.

### `translations`

Manually link to translations by slug:

```yaml
translations:
  - en-hello-world
  - es-hola-mundo
```

Not needed when using subfolder auto-discovery (Options 1 and 2), since translations are linked automatically.

## Template Output

### Translation Links

Content pages with translations show an "Also available in" section with links to each translation:

```
Also available in: English, Espanol
```

### SEO Tags

The `<head>` section of translated content pages includes hreflang alternate link tags:

```html
<link rel="alternate" hreflang="en" href="https://example.com/en-hello-world.html">
<link rel="alternate" hreflang="es" href="https://example.com/es-hola-mundo.html">
<link rel="alternate" hreflang="pt" href="https://example.com/hello.html">
```

The `<html lang="...">` attribute is also set correctly per page.

### Stream Display Names

Language stream pages use the configured language name as the display name. A stream named `pt` automatically displays as "Portugues" without needing a separate `streams.pt.display_name` entry. You can override this by defining both `languages` and `streams` entries for the same code.

## How It Works

1. During content collection, files in subfolders with a configured language code prefix (e.g., `en-`) are detected and assigned to that language stream
2. After all content is collected, a translation discovery phase groups content by subfolder and resolves frontmatter references
3. All members of a translation group get cross-linked with `TranslationRef` entries containing the language code, display name, slug, and title
4. Templates render translation links and hreflang tags from these references

## Compatibility Notes

- The default language's content uses stream `index` and appears on the main `index.html` page
- Language detection from filename prefixes only triggers inside subfolders, never for flat files at the content root (preventing false positives like `essential-guide.md` being detected as language `es`)
- A post can have both a `series` and a language stream - they work independently
- Slug collisions between languages are prevented by the stream prefix in slugs (`en-hello` vs `es-hola`)
- Pages (content without dates) can have `language` and `translations` for template display but do not appear on stream listing pages
