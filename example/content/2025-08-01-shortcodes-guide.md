---
title: "Shortcodes Guide"
tags: ["docs", "features", "shortcodes", "guide"]
authors: ["rochacbruno"]
---

# Shortcodes Guide

> [!IMPORTANT]
> This is a **Beta** feature currently available only on the main branch. It has not been released in a stable version yet.

Shortcodes are a powerful feature in Marmite that allow you to insert dynamic content into your posts and pages using simple markers. They work similar to macros in other static site generators.

## What are Shortcodes?

Shortcodes are reusable snippets of HTML or Markdown that can be inserted into your content using a special syntax. They help you:

- Add complex HTML structures without writing HTML in your markdown
- Reuse common content patterns across multiple pages
- Keep your markdown files clean and readable
- Create dynamic content that adapts based on your site data

## Using Shortcodes

To use a shortcode in your content, use this syntax:

```
<!-- .shortcode_name parameter1=value1 parameter2=value2 -->
```

The shortcode will be replaced with the rendered output when your site is generated.

## Built-in Shortcodes

Marmite comes with several built-in shortcodes:

### Table of Contents (`toc`)

Displays the table of contents for the current page:

```
<!-- .toc -->
```

### YouTube Videos (`youtube`)

Embed YouTube videos with optional custom dimensions:

```
<!-- .youtube id=VIDEO_ID -->
<!-- .youtube id=VIDEO_ID width=800 height=600 -->
```

You can provide either just the video ID or the full YouTube URL.

### Authors List (`authors`)

Display a list of all authors on your site:

```
<!-- .authors -->
```

### Streams List (`streams`)

Display a list of content streams with optional sorting and limiting:

```
<!-- .streams -->
<!-- .streams ord=desc items=5 -->
```

Parameters:
- `ord`: Sort order (`asc` or `desc`, default: `asc`)
- `items`: Maximum number of items to display (default: all)

### Series List (`series`)

Display a list of all content series:

```
<!-- .series -->
<!-- .series ord=desc items=5 -->
```

Parameters:
- `ord`: Sort order (`asc` or `desc`, default: `asc`)
- `items`: Maximum number of items to display (default: all)

### Posts List (`posts`)

Display a list of posts:

```
<!-- .posts -->
<!-- .posts ord=asc items=5 -->
```

Parameters:
- `ord`: Sort order (`asc` or `desc`, default: `desc`)
- `items`: Maximum number of items to display (default: 10)

### Pages List (`pages`)

Display a list of pages:

```
<!-- .pages -->
<!-- .pages ord=desc items=5 -->
```

Parameters:
- `ord`: Sort order (`asc` or `desc`, default: `asc`)
- `items`: Maximum number of items to display (default: all)

### Tags List (`tags`)

Display a list of all tags with post counts:

```
<!-- .tags -->
<!-- .tags ord=desc items=10 -->
```

Parameters:
- `ord`: Sort order (`asc` or `desc`, default: `asc`)
- `items`: Maximum number of items to display (default: all)


## Creating Custom Shortcodes

You can create your own shortcodes by adding files to the `shortcodes` directory in your input folder.

> [!IMPORTANT]
> For HTML shortcodes, the macro name MUST match the filename. For example, a file named `alert.html` must contain `{% macro alert(...) %}`. This is the macro that will be called when the shortcode is used.

### Adding Descriptions to Shortcodes

You can add descriptions to your shortcodes by including a Tera comment as the first line of the file:

```html
{# Display a custom alert box #}
{% macro alert(type="info", message="") %}
...
{% endmacro alert %}
```

These descriptions will be shown when you run `marmite --shortcodes`.

### HTML Shortcodes

Create `.html` files with Tera macros:

```html
{# shortcodes/alert.html #}
{# Display a custom alert box #}
{% macro alert(type="info", message="") %}
<div class="alert alert-{{type}}">
  {{message}}
</div>
{% endmacro alert %}
```

Usage:
```
<!-- .alert type=warning message="This is a warning!" -->
```

### Markdown Shortcodes

Create `.md` files that will be processed as Markdown:

```markdown
{# shortcodes/feature.md #}
{# Display a feature highlight box #}
## Feature: {{ title }}

{{ description }}

{% if link %}
[Learn more]({{ link }})
{% endif %}
```

Usage:
```
<!-- .feature title="Awesome Feature" description="This feature is amazing!" link="/features" -->
```

## Configuration

Shortcodes are enabled by default. You can disable them or customize the pattern in your `marmite.yaml`:

```yaml
# Enable/disable shortcodes
enable_shortcodes: true

# Custom shortcode pattern (regex)
# Default: <!-- \.(\w+)(\s+[^>]+)?\s*-->
shortcode_pattern: "\\[\\[(\\w+)\\s*([^\\]]+)?\\]\\]"
```

## Listing Available Shortcodes

To see all available shortcodes in your project:

```bash
marmite --shortcodes
```

This will list both built-in and custom shortcodes.

## Best Practices

1. **Name shortcodes clearly**: Use descriptive names that indicate what the shortcode does
2. **Document parameters**: Include comments in your shortcode files explaining required and optional parameters
3. **Use defaults**: Provide sensible default values for optional parameters
4. **Keep it simple**: Shortcodes should do one thing well
5. **Test thoroughly**: Check that your shortcodes work correctly with different parameters

## Examples

### Custom Gallery Shortcode

Create `shortcodes/gallery.html`:

```html
{% macro gallery(folder, columns="3") %}
<div class="gallery cols-{{columns}}">
  {% for image in site_data.site.extra.galleries[folder] %}
  <img src="{{image.url}}" alt="{{image.alt}}">
  {% endfor %}
</div>
{% endmacro gallery %}
```

Usage:
```
<!-- .gallery folder=vacation columns=4 -->
```

### Custom Quote Shortcode

Create `shortcodes/quote.md`:

```markdown
> {{ text }}
> 
> — _{{ author }}{% if source %}, {{ source }}{% endif %}_
```

Usage:
```
<!-- .quote text="The only way to do great work is to love what you do." author="Steve Jobs" source="Stanford Commencement Address" -->
```

## Troubleshooting

- **Shortcode not rendering**: Check that the shortcode file exists and has the correct extension
- **Invalid parameters**: Ensure parameter values are properly quoted if they contain spaces
- **HTML shortcodes**: Must contain at least one `{% macro %}` definition
- **Context variables**: All template context variables (like `site_data`, `content`, etc.) are available in shortcodes

With shortcodes, you can extend Marmite's functionality and create rich, dynamic content while keeping your markdown files clean and maintainable!