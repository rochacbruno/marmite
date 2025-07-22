---
title: "Organizing Content with Series in Marmite"
date: 2025-07-22
tags: ["docs", "features", "content-organization", "series"]
authors: ["rochacbruno"]
description: "Learn how to organize related content into chronological series with Marmite's new Series feature"
---

# Organizing Content with Series in Marmite

Marmite now supports **Series** - a powerful feature for organizing related content into chronological sequences. Perfect for tutorials, guides, or any multi-part content that should be read in order.

## What are Series?

Series allow you to group related posts together with automatic chronological ordering and navigation. Unlike tags or streams, series content is always ordered from oldest to newest, making them ideal for:

- **Tutorial series** (like step-by-step guides)
- **Course content** (progressive learning materials)
- **Multi-part articles** (book chapters, long-form content)
- **Sequential documentation** (installation → configuration → usage)

## Basic Usage

### Adding Content to a Series

Simply add a `series` field to your content's frontmatter:

```yaml
---
title: "Python Tutorial - Part 1: Getting Started"
date: 2024-10-30
series: python-tutorial
tags: ["python", "tutorial", "programming"]
authors: ["marmite"]
description: "First part of our comprehensive Python tutorial series"
---
```

### Series Naming Conventions

- Use lowercase, hyphenated names: `python-tutorial`, `web-development-basics`
- Keep names descriptive but concise
- Use consistent naming across all parts of the series

### Combining Series with Streams

Series can be used in combination with streams for advanced content organization:

```yaml
---
title: "Python Tutorial - Part 1: Getting Started"
date: 2024-10-30
series: python-tutorial
stream: tutorial
tags: ["python", "tutorial", "programming"]
authors: ["marmite"]
description: "First part of our comprehensive Python tutorial series"
---
```

When both `series` and `stream` are set:
- **Series navigation takes precedence** - next/previous links follow series order
- **Content appears in stream feeds** - but maintains series chronological order
- **Flexible visibility control** - use streams to exclude series content from main blog listing while keeping it organized in dedicated stream pages

## Generated Pages

Marmite automatically generates several pages for your series:

### Individual Series Pages
Each series gets its own page at `serie-{series-name}.html` showing all posts in chronological order (oldest to newest).

### Series Index Page
A master `series.html` page lists all available series with post counts and links.

### RSS/JSON Feeds
Each series automatically generates:
- `serie-{series-name}.rss` - RSS feed for the series
- `serie-{series-name}.json` - JSON feed for the series

## Configuration

### Series Display Names

Configure friendly display names in `marmite.yaml`:

```yaml
series:
  python-tutorial:
    display_name: "Python Tutorial"
    description: "A comprehensive Python programming tutorial series"
  web-dev-basics:
    display_name: "Web Development Basics"
    description: "Essential skills for modern web development"
```

### Adding Series to Navigation

Add series to your site navigation:

```yaml
menu:
  - ["Pages", "pages.html"]
  - ["Tags", "tags.html"] 
  - ["Archive", "archive.html"]
  - ["Authors", "authors.html"]
  - ["Streams", "streams.html"]
  - ["Series", "series.html"]  # Add this line
```

## Template Features

### Series Information Display

Content that's part of a series automatically shows:
- A series link at the top: "Published as part of 'Python Tutorial' series."
- No related content section (series navigation takes precedence)

### Navigation Behavior

Series content gets special navigation treatment:

- **Next/Previous links** follow series chronology instead of publication date
- **Series order** takes precedence over stream navigation
- **Chronological flow** ensures readers follow the intended sequence

## Template Functions

### Series Display Name Function

Use `series_display_name()` in templates:

```html
<!-- Shows configured display name or falls back to series name -->
{{ series_display_name(series=content.series) }}
```

### Accessing Series Data

In templates, access series information:

```html
{% if content.series %}
  <p>Part of the <a href="{{ url_for(path='serie-' ~ content.series ~ '.html') }}">
    {{ series_display_name(series=content.series) }}
  </a> series</p>
{% endif %}
```

## Best Practices

### Content Organization

1. **Plan your series structure** before creating content
2. **Use consistent dating** to ensure proper ordering
3. **Write clear descriptions** for each series part
4. **Cross-reference related series** when appropriate

### SEO Considerations

- Each series page gets proper meta tags and structured data
- Series RSS feeds improve content discoverability
- Chronological navigation helps search engines understand content relationships

### User Experience

- **Clear series indicators** help readers understand they're in a sequence
- **Consistent numbering** in titles (Part 1, Part 2, etc.)
- **Series completion indicators** in descriptions when finished

## Example: Complete Series Setup

Here's how to set up a complete tutorial series:

**Configuration** (`marmite.yaml`):
```yaml
series:
  python-tutorial:
    display_name: "Python Programming Tutorial"
    description: "Learn Python from basics to advanced concepts"
```

**Content files**:
```markdown
# 2024-10-30-python-tutorial-part-1.md
---
title: "Python Tutorial - Part 1: Getting Started"
date: 2024-10-30
series: python-tutorial
tags: ["python", "tutorial", "programming"]
---

# 2024-11-05-python-tutorial-part-2.md
---
title: "Python Tutorial - Part 2: Data Types"  
date: 2024-11-05
series: python-tutorial
tags: ["python", "tutorial", "programming", "data-types"]
---
```

**Generated URLs**:
- Series page: `/serie-python-tutorial.html`
- Series RSS: `/serie-python-tutorial.rss`
- Series JSON: `/serie-python-tutorial.json`
- Series index: `/series.html`

## Migration from Other Systems

### From Tags/Categories
If you have tutorial content currently tagged:
1. Keep existing tags for discoverability
2. Add `series` field to group related content
3. Update frontmatter gradually

### From Manual Navigation
Replace manual "Next/Previous" links - Marmite handles this automatically for series content.

## Technical Details

- Series content is sorted chronologically (oldest first) unlike other content
- Related content sections are automatically disabled for series posts
- Series navigation takes precedence over stream navigation
- All series data is available in templates via the `site_data.series` object

The Series feature makes Marmite perfect for educational content, documentation sites, and any scenario where content order matters. Start organizing your related content today!