---
tags: docs, streams, content-organization
description: Complete guide to using streams for content organization in Marmite - an alternative to tags for categorizing your content.
---

# Streams Guide: Organizing Content Beyond Tags

Streams are a powerful feature in Marmite that allow you to organize content into distinct categories or series, providing an alternative to traditional tagging systems. Think of streams as content channels or categories that group related posts together.

## What Are Streams?

Streams are content groupings that:
- Create separate content channels within your site
- Generate individual landing pages for each stream
- Provide stream-specific RSS feeds
- Enable next/previous navigation within the stream
- Allow content to be "pinned" to the top of streams

## How Streams Differ from Tags

| Feature | Tags | Streams |
|---------|------|---------|
| **Purpose** | Topical keywords | Content categories |
| **Structure** | Many tags per post | One stream per post |
| **Navigation** | Tag-based browsing | Sequential reading |
| **Feeds** | Tag-specific RSS | Stream-specific RSS |
| **Homepage** | All posts mixed | Stream-specific pages |

## Default Stream: Index

Every post without a specified stream automatically belongs to the "index" stream, which becomes your main blog feed. This is what visitors see on your homepage.

## Creating Streams

### Via Frontmatter

Add a `stream` field to your post's frontmatter:

```yaml
---
title: "My First Tutorial"
stream: tutorials
tags: beginner, guide
---
```

### Via Filename

You can also specify streams using filename patterns:

```
content/
├── tutorial-2024-01-01-getting-started.md  # goes to "tutorial" stream
├── tutorial-2024-01-15-advanced-tips.md    # goes to "tutorial" stream  
├── news-2024-01-10-site-update.md          # goes to "news" stream
├── news-2024-01-20-new-features.md         # goes to "news" stream
├── guide-S-comprehensive-guide.md          # goes to "guide" stream (page)
└── 2024-01-05-general-post.md              # goes to "index" stream
```

**Filename patterns:**
- `{stream}-{date}-{slug}.md` - For posts with dates
- `{stream}-S-{slug}.md` - For pages without dates (S-pattern)

For more details, see [[Filename-Based Streams: Organize Content with File Naming]].

## Stream Configuration

### Stream Display Names

Configure friendly display names for your streams in `marmite.yaml`:

```yaml
streams:
  tutorial:
    display_name: "Python Tutorials"
  news:
    display_name: "Latest News" 
  guide:
    display_name: "User Guides"
  review:
    display_name: "Product Reviews"
```

Use the `stream_display_name` template function to show these friendly names:
```html
{{ stream_display_name(stream=content.stream) }}
```

### Stream Titles

Configure section titles in `marmite.yaml`:

```yaml
streams_title: "Content Streams"
streams_content_title: "Posts from '$stream'"
```

The `$stream` placeholder gets replaced with the actual stream name.

### Stream Menu

Add streams to your navigation:

```yaml
menu:
  - ["Tutorials", "tutorials.html"]
  - ["News", "news.html"]
  - ["Archive", "archive.html"]
```

## Stream Features

### Stream Landing Pages

Each stream gets its own landing page:
- `index.html` - Main stream (default)
- `tutorials.html` - Tutorials stream
- `news.html` - News stream

### Stream Feeds

Automatic RSS and JSON feeds for each stream:
- `index.rss` - Main stream RSS
- `tutorials.rss` - Tutorials stream RSS
- `news.rss` - News stream RSS

### Pinned Content

Pin important posts to the top of a stream:

```yaml
---
title: "Important Announcement"
stream: news
pinned: true
---
```

### Next/Previous Navigation

Within streams, posts get automatic next/previous navigation:
- **Next**: Newer post in the same stream
- **Previous**: Older post in the same stream

## Advanced Stream Usage

### Custom Stream Templates

Create custom templates for specific streams:

```
templates/
├── content.html              # Default content template
├── custom_tutorials.html     # Custom template for tutorials
└── custom_news.html          # Custom template for news
```

### Stream Context in Templates

Templates have access to stream information:

```html
{% if content.stream %}
<div class="stream-info">
  <span class="stream-name">{{ stream_display_name(stream=content.stream) }}</span>
  <a href="{{ content.stream }}.html">View all {{ stream_display_name(stream=content.stream) }} posts</a>
</div>
{% endif %}
```

## Stream Organization Strategies

### By Content Type
```
stream: tutorials    # How-to guides
stream: reviews      # Product reviews
stream: news         # Company updates
stream: personal     # Personal reflections
```

### By Project
```
stream: project-a    # Project A updates
stream: project-b    # Project B updates
stream: general      # General updates
```

### By Audience
```
stream: beginners    # Beginner-friendly content
stream: advanced     # Advanced tutorials
stream: experts      # Expert-level content
```

## Stream Best Practices

1. **Keep it simple**: Don't create too many streams
2. **Be consistent**: Use consistent naming conventions
3. **Use descriptive names**: Make stream purposes clear
4. **Combine with tags**: Use streams for categories, tags for topics
5. **Plan navigation**: Ensure easy discovery of all streams

## Troubleshooting

### Stream Not Appearing
- Check that the stream name is spelled correctly
- Ensure the post has a date (only posts can have streams)
- Verify the stream has at least one published post

### Missing Stream Pages
- Run `marmite --force` to rebuild all pages
- Check that the stream name doesn't conflict with existing pages

### Stream Order
- Streams are ordered alphabetically by name
- Posts within streams are ordered by date (newest first)
- Use `pinned: true` to override post order

## Migration from Tags

If you're migrating from a tag-based system:

1. **Identify your main categories**: What were your most important tags?
2. **Convert categories to streams**: Main categories become streams
3. **Keep specific tags**: Technical details stay as tags
4. **Update navigation**: Replace category links with stream links
5. **Test thoroughly**: Ensure all content is still accessible

## Example Configuration

Complete stream setup in `marmite.yaml`:

```yaml
# Stream display names
streams:
  tutorials:
    display_name: "Tutorial Series"
  reviews:
    display_name: "Product Reviews"
  news:
    display_name: "Latest News"

# Stream configuration
streams_title: "Content Categories"
streams_content_title: "All posts in '$stream'"

# Navigation with streams
menu:
  - ["Tutorials", "tutorials.html"]
  - ["Reviews", "reviews.html"]
  - ["News", "news.html"]
  - ["Tags", "tags.html"]
  - ["Archive", "archive.html"]

# Default author for all streams
default_author: yourname
```

Streams provide a powerful way to organize your content beyond traditional tagging, offering better content discovery and a more structured reading experience for your visitors.
