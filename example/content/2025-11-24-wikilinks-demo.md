---
title: "Wikilinks Demo & How To Guide"
description: "A demonstration of Obsidian-style wikilinks in Marmite with proper slug resolution"
tags: ["features", "documentation", "wikilinks"]
---

# Wikilinks Demo & How To Guide

This post demonstrates the **Obsidian-style wikilink** functionality in Marmite. Wikilinks allow you to create internal links using double square brackets, like in Obsidian, Notion, and other note-taking apps.

## What are Wikilinks?

Wikilinks are internal links written using double square brackets: `[[Title of Target Content]]`. They're especially useful for:

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

## Example Wikilinks

Here are some example wikilinks to other content in this blog:

- [[About]] - Links to the about page
- [[Getting Started]] - Links to the getting started guide  
- [[Python Tutorial - Part 1: Getting Started]] - Links to the first Python tutorial
- [[Title is A]] - Links to a content with a different slug
- [[Markdown Format]] - Links to the markdown documentation
- [[Configuration Reference]] - Links to configuration docs

You can also reference posts with special characters:
- [[TWSBI Eco Indigo Blue & de Atramentis Document Brown]] - Would link to a post with that exact title

## Wikilink Configuration

Wikilinks are enabled by default in Marmite. You can configure them in your `marmite.yaml`:

```yaml
markdown_parser:
  extension:
    wikilinks_title_before_pipe: true   # [[Title|Display Text]]
    wikilinks_title_after_pipe: false   # [[Display Text|Title]]
```

## Benefits Over Regular Markdown Links

**Wikilinks** (`[[Title]]`):
- ✅ No need to remember slugs or filenames
- ✅ Automatic slug resolution
- ✅ Works with titles containing special characters
- ✅ Case-insensitive matching
- ✅ Obsidian/Notion compatibility

**Regular links** (`[Text](slug.html)`):
- ❌ Must know exact slug or filename
- ❌ Manual slug maintenance if titles change
- ❌ Case-sensitive

## Technical Implementation

The wikilink processing happens during HTML generation:

1. **Markdown → HTML**: `comrak` converts `[[Title]]` to `<a href="auto-generated.html" data-wikilink="true">Title</a>`
2. **Post-processing**: Marmite finds `data-wikilink="true"` links and attempts title matching
3. **Slug resolution**: If a matching title is found, the href is replaced with the correct slug
4. **Fallback**: If no match is found, the original auto-generated href is preserved

This approach provides a pragmatic solution that works with the current Marmite architecture while maintaining compatibility with existing functionality.

## Try It Yourself

To test wikilinks in your own Marmite site:

1. Ensure wikilinks are enabled in your config (they are by default)
2. Create content with exact titles you want to reference
3. Use `[[Exact Title]]` syntax in your markdown
4. Build your site and verify the links work correctly

Happy linking! 🔗
