---
title: "Filename-Based Streams: Organize Content with File Naming"
tags: [streams, content-organization, features, docs]
---

# Filename-Based Streams: Organize Content with File Naming

Marmite now supports automatic stream detection from filenames, making it even easier to organize your content without having to manually specify stream names in frontmatter.

## What are Filename-Based Streams?

Filename-based streams allow you to specify which stream a post belongs to directly in the filename, using specific naming patterns. This feature complements the existing frontmatter-based stream assignment.

> [!NOTE]
> Streams are assigned only to `posts` (dated content), pages does not have streams.

## Supported Patterns

### Pattern 1: Stream with Date

Use the pattern `{stream}-{date}-{slug}.md` for posts with dates:

```
content/
├── tutorial-2024-01-01-getting-started.md  # → "tutorial" stream
├── tutorial-2024-01-15-advanced-tips.md    # → "tutorial" stream  
├── news-2024-01-10-site-update.md          # → "news" stream
├── news-2024-01-20-new-features.md         # → "news" stream
└── 2024-01-05-general-post.md              # → "index" stream (default)
```

**Requirements:**
- Stream name must be a single word (no hyphens, underscores, or spaces)
- Must be followed by a valid date in `YYYY-MM-DD` format
- Date must be immediately followed by a hyphen and the slug

**Examples:**
- ✅ `tutorial-2024-01-01-my-first-post.md`
- ✅ `devlog-2024-12-25-christmas-update.md`
- ❌ `code-tutorial-2024-01-01-post.md` (multiple words before date)
- ❌ `tutorial_2024-01-01-post.md` (underscore instead of hyphen)

### Pattern 2: Stream without Date (S-Marker)

For posts without date in the Filename (setting date on frontmatter), use the pattern `{stream}-S-{slug}.md`:

```
content/
├── guide-S-installation-guide.md           # → "guide" stream
├── tutorial-S-comprehensive-overview.md    # → "tutorial" stream
└── about.md                                # → no stream (regular page)
```

**Requirements:**
- Stream name must be a single word
- Must be followed by `-S-` (capital S as a marker)
- The `S` is just a separator and will be ignored

**Examples:**
- ✅ `guide-S-installation.md`
- ✅ `docs-S-api-reference.md`
- ❌ `user-guide-S-setup.md` (multiple words before S marker)

## Priority Order

Stream assignment follows this priority order:

1. **Frontmatter stream** (highest priority)
2. **Filename-based stream** (if no frontmatter stream)
3. **Default "index" stream** (if no stream detected)

```markdown
---
stream: priority-stream  # This takes precedence
---
```

Even in a file named `tutorial-2024-01-01-post.md`, the frontmatter stream will be used.

## Stream Display Names

You can configure display names for filename-based streams in your `marmite.yaml`:

```yaml
streams:
  tutorial:
    display_name: "Python Tutorials"
  news: 
    display_name: "Latest News"
  devlog:
    display_name: "Development Blog"
  guide:
    display_name: "User Guides"
```

## Template Usage

Use the `stream_display_name` function in your templates to show friendly stream names:

```html
{% if content.stream %}
<div class="stream-info">
  <span class="stream-name">
    {{ stream_display_name(stream=content.stream) }}
  </span>
  <a href="{{ content.stream }}.html">
    View all {{ stream_display_name(stream=content.stream) }} posts
  </a>
</div>
{% endif %}
```

## File Organization Best Practices

### Consistent Naming
- Use consistent stream names across your site
- Choose short, descriptive stream names
- Avoid special characters in stream names

### Example Structure

```
content/
├── tutorial-2024-01-01-python-basics.md
├── tutorial-2024-01-15-advanced-concepts.md
├── news-2024-01-10-version-2-release.md
├── news-2024-01-20-new-features.md
├── devlog-2024-01-05-progress-update.md
├── guide-S-installation.md
├── guide-S-configuration.md
├── about.md
└── contact.md
```

## Migration from Frontmatter Streams

To migrate existing frontmatter-based streams to filename-based streams:

1. **Identify your streams**: List all streams currently used in frontmatter
2. **Rename files**: Update filenames to use the appropriate pattern
3. **Remove frontmatter**: Remove `stream:` entries from frontmatter (optional)
4. **Test thoroughly**: Ensure all content is properly categorized

## Troubleshooting

### Stream Not Detected
- Check that stream name is a single word
- Verify date format is `YYYY-MM-DD`
- Ensure proper hyphen separators
- Confirm the file has a valid date for the date pattern

### Wrong Stream Assignment
- Check frontmatter for existing `stream:` entries (they take priority)
- Verify filename follows the exact patterns shown above
- Ensure there are no typos in the stream name

### Missing Stream Pages
- Streams are only created when there's at least one post assigned
- Pages (without dates) using S-pattern won't create stream indexes
- Check that posts have valid dates for stream index generation

This feature makes content organization more intuitive by letting you see stream assignments directly in your file browser, while maintaining full compatibility with existing frontmatter-based streams.
