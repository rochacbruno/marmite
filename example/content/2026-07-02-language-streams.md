---
tags: docs, features, i18n, multilingual
description: Write content in multiple languages with automatic translation linking, language stream pages, and hreflang SEO tags using marmite's language streams feature.
---

# Language Streams - Multilingual Content

Marmite supports multilingual sites through language streams. Each language becomes a stream with its own listing page and RSS feed, while translations are automatically cross-linked with "Also available in" navigation and hreflang SEO tags.

## How It Works

Languages are auto-detected from content. Just set `language: xx` in your frontmatter or use subfolder naming conventions, and marmite handles the rest - no configuration required.

The `language` field in `marmite.yaml` (defaults to `en`) determines the site's default language. Content in the default language stays on `index.html`. Other languages get their own stream pages (`pt.html`, `es.html`) and RSS feeds (`pt.rss`, `es.rss`).

### Optional: Display Names

By default, language streams are labeled with their two-letter code (e.g., "pt", "es"). To set human-readable names, add an optional `languages` section to `marmite.yaml`:

```yaml
language: en
languages:
  pt:
    display_name: "Portugues"
  es:
    display_name: "Espanol"
```

This follows the same pattern as `streams:` and `series:` - the config is purely cosmetic. Sites without any language content are completely unaffected.

## Content Organization

There are four ways to organize multilingual content. All produce flat HTML output.

### Option 1: Subfolder Grouping (Auto-Discovery)

> **RECOMMENDED**

Group translations in a subfolder named after the base content's slug. Files prefixed with an ISO 639-1 language code are automatically detected and cross-linked:

```
content/hello/
  hello.md              # Default language (en)
  pt-ola-mundo.md       # Portuguese translation
  es-hola-mundo.md      # Spanish translation
```

This generates:
- `hello.html` - English post, listed on `index.html`
- `pt-ola-mundo.html` - Portuguese post, listed on `pt.html`
- `es-hola-mundo.html` - Spanish post, listed on `es.html`

All three pages automatically show "Also available in" links to each other.

> [!TIP]
> The subfolder can also have the date in it, e.g. `content/2026-07-02-hello/` this way you don't have to specify date on each translation frontmatter.

### Option 2: Mixed Flat + Subfolder

> **RECOMMENDED**

If you have an existing flat site and want to add translations without moving original files, create a subfolder matching the existing content's slug:

```
content/
  hello.md              # Existing flat file, slug: hello
  hello/
    pt-ola.md           # Portuguese translation, auto-linked
```

Marmite detects that the subfolder name `hello` matches the flat file's slug and links them as translations.

> [!IMPORTANT]
> Subfolder names must match the original post's slug (not the filename, but the resolved slug, sometimes taken from the title) to be automatically linked as translations.

### Option 3: Translates Pointer

Each translation points to the original content's slug using the `translates` field. Marmite builds the full bidirectional link network automatically:

```yaml
---
title: Ola Mundo
date: 2024-01-01
language: pt
translates: hello
---
```

This is simpler than maintaining a `translations` list on every file. Just set `language` and `translates` on each translation and marmite connects everything. Setting `language` to a value different from the site default automatically places the content on the corresponding language stream.

### Option 4: Frontmatter Translation Link

Set the language and translations explicitly in frontmatter:

```yaml
---
title: Hello World
date: 2024-01-01
language: en # can omit because it's the default language
translations:
  - pt-ola  # then you write a post with slug `ola` and language: set to `pt`
  - es-hola
---
```

The `translations` field accepts a list of slugs. Marmite resolves each slug to the actual content, fills in the language code and display name, and creates bidirectional links. If post A lists post B as a translation, post B automatically gets a link back to post A.

> [!IMPORTANT]
> Prefer options 1 and 2 for auto discovery. Options 3 and 4 require explicit frontmatter but give you more control over the linking.

## Frontmatter Fields

### `language`

Explicitly set the content's language code:

```yaml
language: pt
```

Usually not needed when using subfolder detection (Options 1 and 2). Use this when you need to set the language explicitly (Options 3 and 4).

When `language` is set to a value different from the site default and no explicit `stream` is set, marmite automatically uses the language as the stream. A post with `language: pt` will be published to `pt.html`.

### `translates`

Point a translation to the original content's slug:

```yaml
translates: hello
```

Marmite creates bidirectional links between the source and all its translations. Not needed with subfolder auto-discovery. See Option 3 above for details.

### `translations`

Manually link to translations by slug:

```yaml
translations:
  - en-hello-world
  - es-hola-mundo
```

Not needed when using subfolder auto-discovery (Options 1 and 2) or `translates:` (Option 3), since translations are linked automatically in those modes.

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

## How It Works Internally

1. During content collection, files in subfolders with an ISO 639-1 language code prefix (e.g., `en-`) are detected and assigned to that language stream
2. Languages are auto-populated from all observed content - no config needed
3. After all content is collected, a translation discovery phase groups content by subfolder, processes `translates:` pointers, and resolves frontmatter references
4. All members of a translation group get cross-linked with `TranslationRef` entries containing the language code, display name, slug, and title
5. Templates render translation links and hreflang tags from these references

## Compatibility Notes

- The default language's content uses stream `index` and appears on the main `index.html` page
- Language detection from filename prefixes only triggers inside subfolders, never for flat files at the content root (preventing false positives like `essential-guide.md` being detected as language `es`)
- A post can have both a `series` and a language stream - they work independently
- Slug collisions between languages are prevented by the stream prefix in slugs (`en-hello` vs `es-hola`)
- Pages (content without dates) can have `language` and `translations` for template display but do not appear on stream listing pages
