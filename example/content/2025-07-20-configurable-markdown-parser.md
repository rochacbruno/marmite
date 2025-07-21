---
title: Configurable Markdown Parser Options
date: 2025-07-20
tags: [docs, configuration, markdown, features]
authors: [rochacbruno]
---

# Configurable Markdown Parser Options

Marmite now supports configurable markdown parser options through the `markdown_parser` section in your `marmite.yaml` configuration file. This allows you to customize how your markdown content is processed and rendered.

## Configuration Structure

Add a `markdown_parser` section to your `marmite.yaml` file with the following structure:

```yaml
markdown_parser:
  render:
    unsafe: true                    # Allow/disallow unsafe HTML (default: true)
    ignore_empty_links: true        # Ignore empty link references (default: true)
    figure_with_caption: true       # Wrap images in figure tags (default: true)
  parse:
    relaxed_tasklist_matching: true # Allow relaxed task list syntax (default: true)
  extension:
    tagfilter: false               # Enable/disable tag filtering (default: false)
    strikethrough: true            # Enable/disable ~~strikethrough~~ (default: true)
    table: true                    # Enable/disable table support (default: true)
    autolink: true                 # Enable/disable automatic link detection (default: true)
    tasklist: true                 # Enable/disable task lists (default: true)
    footnotes: true                # Enable/disable footnote support (default: true)
    description_lists: true        # Enable/disable description lists (default: true)
    multiline_block_quotes: true   # Enable/disable multiline block quotes (default: true)
    underline: true                # Enable/disable __underline__ syntax (default: true)
    spoiler: true                  # Enable/disable spoiler text (default: true)
    greentext: true                # Enable/disable greentext (default: true)
    shortcodes: true               # Enable/disable shortcodes (default: true)
    wikilinks_title_before_pipe: true # Wiki-style links with title before pipe (default: true)
    wikilinks_title_after_pipe: false # Wiki-style links with title after pipe (default: false)
    alerts: true                   # Enable/disable alert blocks (default: true)
```

## Security Considerations

The `unsafe` option controls whether raw HTML is allowed in your markdown content:

- `unsafe: true` (default) - Allows all HTML tags, including `<script>` and other potentially dangerous elements
- `unsafe: false` - Escapes all HTML tags for security, rendering them as text

For public sites or when processing untrusted content, consider setting `unsafe: false`.

## Extension Options

### Strikethrough
```yaml
strikethrough: true  # ~~text~~ → <del>text</del>
strikethrough: false # ~~text~~ → ~~text~~
```

### Underline
```yaml
underline: true  # __text__ → <u>text</u>
underline: false # __text__ → <strong>text</strong>
```

### Tables
```yaml
table: true # Enables GitHub-style table syntax
```

### Task Lists
```yaml
tasklist: true # Enables - [x] checked and - [ ] unchecked syntax
```

### Footnotes
```yaml
footnotes: true # Enables footnote syntax with [^1] references
```

## Partial Configuration

You only need to specify the options you want to change from the defaults. For example, to disable unsafe HTML and strikethrough:

```yaml
markdown_parser:
  render:
    unsafe: false
  extension:
    strikethrough: false
```

All other options will use their default values.

## Backward Compatibility

If no `markdown_parser` section is specified in your configuration, Marmite will use the same default settings as before, ensuring backward compatibility with existing sites.