---
date: 2025-11-24
title: "Wikilinks Demo & How To Guide"
description: "A demonstration of Obsidian-style wikilinks in Marmite with proper slug resolution"
tags: ["features", "documentation", "wikilinks"]
---

# Wikilinks Demo & How To Guide

This post demonstrates the **Obsidian-style wikilink** functionality in Marmite. Wikilinks allow you to create internal links using double square brackets, like in Obsidian, Notion, and other note-taking apps.

Also see the [[Wikilinks|Markdown Format#Wikilinks]] section in the Markdown Format reference for a quick syntax overview.

## What are Wikilinks?

Wikilinks are internal links written using double square brackets. They're especially useful for:

- Creating interconnected note systems
- Linking between related posts and pages
- Building knowledge bases and wikis
- Quick internal linking without worrying about slugs

## How Wikilinks Work in Marmite

When you write a wikilink like `[[Title of Target Content]]`, Marmite:

1. **Converts** it to HTML with a `data-wikilink="true"` attribute
2. **Searches** for content with a matching title (case-insensitive)
3. **Replaces** the auto-generated filename-based href with the proper slug
4. **Falls back** to the original href if no matching content is found

## Wikilink Syntax

Marmite supports three forms of wikilinks.

### Plain wikilinks (Obsidian Links)

The simplest form links directly to a page by its slug or title:

[[About]] - Links by slug  
[[Getting Started]] - Links by title  
[[Markdown Format]] - Links by title

```markdown
[[About]] - Links by slug
[[Getting Started]] - Links by title
[[Markdown Format]] - Links by title
```

You can also use file extensions - they are stripped automatically:

[[about.md]] and [[about.html]] both resolve to the about page.

```markdown
[[about.md]] and [[about.html]] both resolve to the about page.
```

Anchors are supported too:

[[about#faq]] - Links to a specific section  
[[#Wikilink Configuration]] - Links to a section on this page

```markdown
[[about#faq]] - Links to a specific section
[[#Wikilink Configuration]] - Links to a section on this page
```

External URLs also work:

[[https://pudim.com.br]]

```markdown
[[https://pudim.com.br]]
```

### Piped wikilinks (with display text)

By default, Marmite uses `wikilinks_title_before_pipe` mode, where the syntax is `[[Display Text|target]]`. The text before the pipe is what the reader sees, and the part after the pipe is the slug, filename, or title of the target page:

[[Read the Tutorial|getting-started]] - "Read the Tutorial" is shown, links to getting-started  
[[Config Docs|Configuration Reference]] - Links to the Configuration Reference page by title  
[[FAQ Section|about#faq]] - Links to a specific section with custom text

```markdown
[[Read the Tutorial|getting-started]] - display text before pipe, slug after pipe
[[Config Docs|Configuration Reference]] - links by title
[[FAQ Section|about#faq]] - links to a section with custom text
```

You can also link to external URLs with custom display text:

[[Visit Pudim|https://pudim.com.br]]

```markdown
[[Visit Pudim|https://pudim.com.br]]
```

### Alternative pipe direction

If you prefer the opposite convention (target before pipe, display text after), set `wikilinks_title_after_pipe: true` in your config. Then the syntax becomes `[[target|Display Text]]`:

```yaml
markdown_parser:
  extension:
    wikilinks_title_before_pipe: false
    wikilinks_title_after_pipe: true
```

```markdown
[[getting-started|Read the Tutorial]]  # target before pipe, display text after
```

> [!NOTE]
> Only one pipe direction should be enabled at a time. See [[Configurable Markdown Parser Options]] for full details.

## Wikilink Configuration

Wikilinks are enabled by default in Marmite. You can configure them in your `marmite.yaml`:

```yaml
markdown_parser:
  extension:
    wikilinks_title_before_pipe: true   # [[Display Text|target]] (default)
    wikilinks_title_after_pipe: false   # [[target|Display Text]]
```

## Title Resolution

When Marmite encounters a wikilink, it tries to resolve the target in this order:

1. **Slug match** - if the target matches a content slug directly (e.g., `about`, `getting-started`)
2. **Title match** - if the target matches a content title (case-insensitive, e.g., `Getting Started`, `Markdown Format`)
3. **Fallback** - if no match is found, the original auto-generated href is kept

This means you can use either slugs or titles as targets:

[[about]] - matches the slug "about"  
[[About]] - matches the title "About"

```markdown
[[about]] - matches the slug "about"
[[About]] - matches the title "About"
```

Special characters in titles are handled automatically:

[[TWSBI Eco Indigo Blue & de Atramentis Document Brown]] - title with special characters

```markdown
[[TWSBI Eco Indigo Blue & de Atramentis Document Brown]]
```

## Back-links

When you link to another page using wikilinks, Marmite tracks the back-reference. The linked page will show a list of all pages that link to it. This works with all link types - wikilinks, regular markdown links, and HTML links.

## Benefits Over Regular Markdown Links

| Feature | Wikilinks `[[Title]]` | Regular links `[Text](slug.html)` |
|---|---|---|
| Know exact slug? | Not needed | Required |
| Auto slug resolution | Yes | No |
| Special characters | Handled | Manual encoding |
| Case sensitivity | Case-insensitive | Case-sensitive |
| Obsidian compatible | Yes | N/A |

## Try It Yourself

1. Ensure wikilinks are enabled in your config (they are by default)
2. Create content with titles you want to reference
3. Use `[[Exact Title]]` or `[[Display Text|slug]]` in your markdown
4. Build your site and verify the links resolve correctly
