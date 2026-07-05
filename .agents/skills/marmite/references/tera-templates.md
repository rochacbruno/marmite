# Marmite Template Reference

Marmite uses the Tera template engine (Jinja2-like syntax). Templates live in the `templates/` directory or within a theme's `templates/` folder.

Run `marmite <folder> --init-templates` to export the default templates for customization.

## Template Files

### Required Templates

| Template | Purpose | Key Variables |
|----------|---------|---------------|
| `base.html` | Base layout all pages extend | `site`, `menu`, `language` |
| `content.html` | Single post or page | `content`, `site` |
| `list.html` | Content listings (index, pagination, tag/stream pages) | `content_list`, `title`, `current_page_number`, `total_pages` |
| `group.html` | Grouped taxonomy views (all tags, all streams, etc.) | `title`, `kind` |

### Optional Templates

| Template | Purpose |
|----------|---------|
| `pagination.html` | Pagination controls (included in list.html) |
| `comments.html` | Comments section (included in content.html) |
| `content_title.html` | Post title component |
| `content_date.html` | Post date display |
| `content_authors.html` | Authors display with avatars |
| `group_author_avatar.html` | Author avatar in group views |
| `json_ld_content.html` | Structured data for posts |
| `json_ld_index.html` | Structured data for index |
| `json_ld_author.html` | Structured data for author pages |
| `base_feeds.html` | RSS/JSON feed template |
| `sitemap.xml` | Sitemap template |
| `custom_index.html` | Custom homepage (overrides default list view) |

## Template Blocks

`base.html` defines these overridable blocks:

```html
{% block seo %}
  <!-- Open Graph, Twitter cards, meta tags -->
{% endblock seo %}

{% block head %}
  <!-- CSS includes, fonts, code highlight styles -->
{% endblock head %}

{% block main %}
  <!-- Main content area -->
{% endblock main %}

{% block tail %}
  <!-- JavaScript, search script, custom scripts -->
{% endblock tail %}
```

Child templates extend base: `{% extends "base.html" %}`

## Global Template Variables

Available in all templates:

| Variable | Type | Description |
|----------|------|-------------|
| `site` | Object | Full site configuration (the Marmite struct) |
| `site.name` | String | Site name |
| `site.tagline` | String | Site tagline |
| `site.url` | String | Site base URL |
| `site.footer` | String | Footer HTML |
| `site.language` | String | Language code |
| `site.extra` | Object | Extra config values |
| `site.enable_search` | bool | Search enabled |
| `site.toc` | bool | TOC enabled globally |
| `site.publish_md` | bool | Markdown source publishing |
| `language` | String | Shortcut for `site.language` |
| `menu` | Array | Navigation menu items `[[label, url], ...]` |
| `title` | String | Page title |
| `hero` | String | Hero section HTML (from `_hero.md`) |
| `sidebar` | String | Sidebar HTML (from `_sidebar.md`) |
| `header` | String | Header HTML (from `_header.md`) |
| `footer` | String | Footer HTML (from `_footer.md`) |
| `announce` | String | Announcement HTML (from `_announce.md`) |
| `comments` | String | Comments HTML (from `_comments.md`) |
| `htmlhead` | String | Extra head HTML (from `_htmlhead.md`) |
| `htmltail` | String | Extra tail HTML |
| `site_data` | Object | All site content data |

## Content Page Variables

Available in `content.html` when rendering a single post or page:

| Variable | Type | Description |
|----------|------|-------------|
| `content.title` | String | Post/page title |
| `content.slug` | String | URL slug |
| `content.html` | String | Rendered HTML content |
| `content.description` | String | Description/excerpt |
| `content.date` | String | Publication date (null for pages) |
| `content.tags` | Array | List of tag strings |
| `content.authors` | Array | List of author usernames |
| `content.stream` | String | Stream name (null if none) |
| `content.series` | String | Series name (null if none) |
| `content.pinned` | bool | Whether post is pinned |
| `content.toc` | String | Table of contents HTML (null if disabled) |
| `content.card_image` | String | Social card image URL |
| `content.banner_image` | String | Banner image URL |
| `content.extra` | Object | Custom frontmatter fields |
| `content.back_links` | Array | Content that links to this page |
| `content.next` | Object | Next post in series/stream (null if none) |
| `content.previous` | Object | Previous post in series/stream (null if none) |
| `content.source_path` | String | Path to source markdown file |
| `content.comments` | bool | Whether comments are enabled |
| `content.language` | String | Language code (null if not set or no i18n) |
| `content.translations` | Array | Translation references (empty if none) |

## List Page Variables

Available in `list.html`:

| Variable | Type | Description |
|----------|------|-------------|
| `content_list` | Array | Posts for the current page |
| `title` | String | Page title |
| `current_page_number` | int | Current page number (1-based) |
| `total_pages` | int | Total number of pages |
| `total_content` | int | Total number of posts |
| `per_page` | int | Posts per page |
| `previous_page` | String | URL of previous page (null if first) |
| `next_page` | String | URL of next page (null if last) |
| `author` | Object | Author object (on author pages only) |

## Group Page Variables

Available in `group.html`:

| Variable | Type | Description |
|----------|------|-------------|
| `title` | String | Page title |
| `kind` | String | Group type: `tag`, `author`, `archive`, `stream`, `series` |

Use the `group()` function to get grouped content within the template.

## Custom Tera Functions

### `url_for(path, abs)`

Generate URLs relative to the site base:

```html
<a href="{{ url_for(path='about.html') }}">About</a>
<link rel="canonical" href="{{ url_for(path='index.html', abs=true) }}">
```

- `path` (required): The target path
- `abs` (optional, default `false`): Generate absolute URL with `site.url`

### `group(kind, ord, items)`

Get grouped content for taxonomy displays:

```html
{% set tags = group(kind="tag", ord="desc", items=10) %}
{% for name, posts in tags %}
  <h3>{{ name }} ({{ posts | length }})</h3>
{% endfor %}
```

- `kind` (required): `"tag"`, `"archive"`, `"author"`, `"stream"`, `"series"`, `"language"`
- `ord` (optional, default `"desc"`): Sort order. `"desc"` sorts by post count (most first). `"asc"` sorts alphabetically. For `"language"`, the default site language always appears last regardless of sort order.
- `items` (optional, default `0`): Max items to return (0 = all)

### `get_posts(ord, items)`

Get a sorted, limited list of posts:

```html
{% set recent = get_posts(ord="desc", items=5) %}
{% for post in recent %}
  <a href="{{ url_for(path=post.slug ~ '.html') }}">{{ post.title }}</a>
{% endfor %}
```

- `ord` (optional, default `"desc"`): Sort order
- `items` (optional, default `0`): Max posts (0 = all)

### `get_data_by_slug(slug)`

Look up content metadata by slug for card displays:

```html
{% set data = get_data_by_slug(slug="my-post") %}
<div>{{ data.title }} - {{ data.content_type }}</div>
```

Returns a `SlugData` object: `image`, `slug`, `title`, `text`, `content_type`.

Supports prefixed slugs:
- `series-python-tutorial` - looks up series
- `stream-tutorial` - looks up stream
- `tag-rust` - looks up tag
- `author-alice` - looks up author
- `archive-2024` - looks up archive year
- `my-post` - looks up post or page by slug

### `source_link(content)`

Generate a link to the markdown source of a post:

```html
{% set src = source_link(content=content) %}
{% if src %}<a href="{{ src }}">View source</a>{% endif %}
```

Uses `source_repository` config URL if set, falls back to local `.md` file if `publish_md` is true.

### `stream_display_name(stream)`

Get the configured display name for a stream:

```html
{{ stream_display_name(stream="tutorial") }}
<!-- Output: "Tutorials" (if configured) or "tutorial" (fallback) -->
```

### `series_display_name(series)`

Get the configured display name for a series:

```html
{{ series_display_name(series="python-tutorial") }}
<!-- Output: "Python Tutorial" (if configured) -->
```

### `language_display_name(language)`

Get the configured display name for a language:

```html
{{ language_display_name(language="pt") }}
<!-- Output: "Portugues" (if configured in languages: config) -->
```

### `get_gallery(path)`

Get gallery data by path:

```html
{% set gallery = get_gallery(path="photos") %}
```

## Custom Tera Filters

### `default_date_format`

Format a date string using the site's configured format:

```html
{{ content.date | default_date_format }}
<!-- Output: "Jun 15, 2024" (with default format) -->
```

The format is controlled by the `default_date_format` config field (chrono strftime syntax).

### `remove_draft`

Filter an array of content to exclude items with `stream == "draft"`:

```html
{% set published = content_list | remove_draft %}
```

## Tera Syntax Quick Reference

Marmite uses Tera 2.0. A backward-compatibility preprocessor auto-converts old Tera 1.x syntax, so existing templates continue to work. The examples below show Tera 2.0 syntax (recommended for new templates).

```html
<!-- Variables -->
{{ variable }}
{{ object.field }}

<!-- Array indexing (Tera 2.0 uses brackets, old dot syntax auto-converted) -->
{{ menu[0] }}
{{ link[1] }}

<!-- Optional chaining (new in Tera 2.0, safe access to undefined values) -->
{{ site?.extra?.comments?.source }}

<!-- Conditionals -->
{% if condition %}...{% elif other %}...{% else %}...{% endif %}

<!-- Tests with keyword args (Tera 2.0 requires named args, old syntax auto-converted) -->
{% if url is starting_with(pat="http") %}...{% endif %}
{% if name is containing(pat="hello") %}...{% endif %}

<!-- Loops -->
{% for item in items %}
  {{ item }}
  {{ loop.index }}     <!-- 1-based index -->
  {{ loop.index0 }}    <!-- 0-based index -->
  {{ loop.first }}     <!-- true on first iteration -->
  {{ loop.last }}      <!-- true on last iteration -->
{% endfor %}

<!-- Filters -->
{{ value | upper }}
{{ value | default(value="fallback") }}
{{ value | length }}
{{ value | truncate(length=100) }}
{{ value | replace(from="x", to="y") }}
{{ value | urlencode }}
{{ value | escape }}

<!-- Template inheritance -->
{% extends "base.html" %}
{% block name %}...{% endblock name %}

<!-- Includes -->
{% include "partial.html" %}

<!-- Components (replaced macros in Tera 2.0, registered globally, no imports needed) -->
{% component my_button(label: string, variant: string = "primary") %}
<button class="btn btn-{{ variant }}">{{ label }}</button>
{% endcomponent my_button %}

<!-- Call a component (self-closing, no body) -->
{{<my_button label="Click me" variant="secondary" />}}

<!-- Call a component (with body content, accessible via {{ body }} inside) -->
{% <info_card title="Hello"> %}
  <p>This content is passed as the body variable</p>
{% </info_card> %}

<!-- Open component with spread (accepts extra kwargs) -->
{% component form_input(name: string, label: string = "", ...rest) %}
<label>{{ label }}<input name="{{ name }}" /></label>
{% endcomponent form_input %}
{{<form_input name="email" label="Email" required={true} />}}

<!-- Native array slicing (Tera 2.0) -->
{{ items[:3] }}                   <!-- first 3 elements -->
{{ items[1:5] }}                  <!-- elements 1 through 4 -->
{{ items[::-1] }}                 <!-- reversed -->

<!-- Ternary expressions (Tera 2.0) -->
{{ value if condition else "fallback" }}
{{ image if image else "" }}

<!-- Map literals and spread (Tera 2.0) -->
{% set base = {"key": "val", "other": 42} %}
{% set merged = {...base, "key": "override", "new": true} %}

<!-- List comprehension (Tera 2.0) -->
{% set filtered = [x for x in items if x.active] %}
{% set titles = [item.title for item in posts if item.pinned] %}

<!-- Set variables -->
{% set myvar = "value" %}
{% set_global myvar = "value" %}  <!-- persists outside of for loops -->

<!-- Raw (no template processing) -->
{% raw %}{{ this is not processed }}{% endraw %}
```

### Tera 2.0 Backward Compatibility

Marmite provides these compatibility features for templates written with Tera 1.x syntax:

- **Auto-converted syntax:** Array dot indexing (`item.0`), test positional args (`is starting_with("http")`), and `ignore missing` on includes are automatically converted by the preprocessor.
- **Compatibility filters:** `striptags`, `slice`, `trim_start_matches`, and `date` were removed from Tera 2.0 core. Marmite provides them as built-in filters so they continue to work. The `slice` filter works alongside native slicing syntax (`items[:3]`).
- **Old conditionals:** Verbose `if/else` blocks and `and` guards still work. Tera 2.0 ternary expressions (`x if cond else y`) are preferred for new templates but not required.
- **Shortcodes:** Shortcode files use `{% shortcode name() %}` syntax (recommended) or `{% macro name() %}` (backward compatible). Both are extracted by marmite's own parser, not by Tera.

## Common Template Patterns

### Custom homepage

Create `templates/custom_index.html`:
```html
{% extends "base.html" %}
{% block main %}
<main class="container">
  <h1>Welcome to {{ site.name }}</h1>
  {{ hero | safe }}
  {% set recent = get_posts(items=5) %}
  {% for post in recent %}
    <article>
      <h2><a href="{{ url_for(path=post.slug ~ '.html') }}">{{ post.title }}</a></h2>
      <p>{{ post.description }}</p>
    </article>
  {% endfor %}
</main>
{% endblock main %}
```

### Conditional content display

```html
{% if content.banner_image %}
  <img src="{{ content.banner_image }}" alt="{{ content.title }}">
{% endif %}

{% if content.series %}
  <span>Part of: {{ series_display_name(series=content.series) }}</span>
{% endif %}

{% if content.tags | length > 0 %}
  {% for tag in content.tags %}
    <a href="{{ url_for(path='tag-' ~ tag ~ '.html') }}">{{ tag }}</a>
  {% endfor %}
{% endif %}
```

### Author display with profile

```html
{% for author_name in content.authors %}
  {% if site.authors[author_name] is defined %}
    {% set author = site.authors[author_name] %}
    <img src="{{ author.avatar }}" alt="{{ author.name }}">
    <span>{{ author.name }}</span>
  {% else %}
    <span>{{ author_name }}</span>
  {% endif %}
{% endfor %}
```
