# Tera 2.0 template improvements

Analysis of Tera 2.0 features that marmite templates could leverage.
These are optional improvements - everything works as-is.

## High impact

### 1. Native array slicing replaces the `slice` filter

Currently 6+ places use `| slice(end=N)`. Tera 2.0 has native `[start:end]` syntax:

```html
{# Before #}
{% for tag in content.tags | slice(end=3) %}
{% for item in content.back_links | slice(end=10) %}
{% for item in related_content | sort(attribute="date") | reverse | slice(end=5) %}

{# After #}
{% for tag in content.tags[:3] %}
{% for item in content.back_links[:10] %}
{% for item in (related_content | sort(attribute="date") | reverse)[:5] %}
```

The custom `slice` filter in `tera_filter.rs` stays for backward compat with user templates, but example templates could use native syntax. Also supports negative indexing (`[-1]`) and step (`[::2]`).

### 2. Ternary expressions clean up card.html

The `card.html` shortcode has 8 blocks of this pattern:

```html
{# Before - 4 lines each, repeated 8 times #}
{% if image %}
    {% set final_image = image %}
{% else %}
    {% set final_image = "" %}
{% endif %}

{# After - 1 line each #}
{% set final_image = image if image else "" %}
```

This would cut card.html from 65 lines to roughly 40.

### 3. List comprehension replaces `remove_draft` filter

The `remove_draft` custom filter could become native Tera:

```html
{# Before #}
{% for item in items | remove_draft | slice(end=10) %}

{# After #}
{% for item in [item for item in items if item.stream != "draft"][:10] %}
```

Judgment call - the filter is more readable, the comprehension is more flexible and eliminates a custom Rust filter. Could keep the filter for readability and offer the comprehension as an alternative in docs.

## Medium impact

### 4. Optional chaining for more patterns

Already used for `site?.extra?.comments?.source is defined`. More patterns could benefit:

```html
{# Before #}
{% if site.atproto and site.atproto.publication_uri %}

{# After #}
{% if site?.atproto?.publication_uri %}
```

Several places in templates do `if X and X.Y` guards that optional chaining eliminates.

### 5. Spread operator for map merging

The spread operator (`{...base, "key": value}`) creates a new map by copying all fields from `base` and overriding specific keys. This maps directly to the "defaults + overrides" pattern used heavily in `card.html` and `gallery.html`.

#### card.html - before (current, 52 lines of logic)

The shortcode builds `final_*` variables by checking each caller parameter against a data lookup fallback. There are two branches (external URL vs internal slug), each with 4 repeated if/else blocks:

```html
{% shortcode card(slug, image="", title="", text="", content_type="") %}
{% if slug is starting_with("http://") or slug is starting_with("https://") %}
    {% if image %}
        {% set final_image = image %}
    {% else %}
        {% set final_image = "" %}
    {% endif %}
    {% if title %}
        {% set final_title = title %}
    {% else %}
        {% set final_title = "Link" %}
    {% endif %}
    {# ... same pattern for text, content_type ... #}
    {% set final_url = slug %}
    {% set target_blank = true %}
{% else %}
    {% set data = get_data_by_slug(slug=slug) %}
    {% if image %}
        {% set final_image = image %}
    {% else %}
        {% set final_image = data.image %}
    {% endif %}
    {% if title %}
        {% set final_title = title %}
    {% else %}
        {% set final_title = data.title %}
    {% endif %}
    {# ... same pattern for text, content_type ... #}
    {% set final_url = url_for(path=data.slug ~ ".html") %}
    {% set target_blank = false %}
{% endif %}
```

#### card.html - after (with spread + ternary)

Build a defaults map, then spread caller overrides on top. Non-empty caller values win:

```html
{% shortcode card(slug, image="", title="", text="", content_type="") %}
{% if slug is starting_with("http://") or slug is starting_with("https://") %}
    {% set card = {"image": "", "title": "Link", "text": "External Link", "content_type": "Link", "url": slug, "target_blank": true} %}
{% else %}
    {% set data = get_data_by_slug(slug=slug) %}
    {% set card = {"image": data.image, "title": data.title, "text": data.text, "content_type": data.content_type, "url": url_for(path=data.slug ~ ".html"), "target_blank": false} %}
{% endif %}
{# Caller overrides - spread base card, override non-empty params #}
{% set card = {
    ...card,
    "image": image if image else card.image,
    "title": title if title else card.title,
    "text": text if text else card.text,
    "content_type": content_type if content_type else card.content_type
} %}
```

This reduces the logic from 52 lines to about 15, eliminates 8 if/else blocks, and makes the override pattern explicit: build defaults, then spread overrides on top.

#### gallery.html - before

```html
{% if name %}
    {% set gallery_name = name %}
{% else %}
    {% set gallery_name = gallery_data.name %}
{% endif %}

{% if cover %}
    {% set cover_image = cover %}
{% else %}
    {% set cover_image = gallery_data.cover %}
{% endif %}
```

#### gallery.html - after (with spread + ternary)

```html
{% set gallery = {
    ...gallery_data,
    "name": name if name else gallery_data.name,
    "cover": cover if cover else gallery_data.cover
} %}
```

Two override blocks become a single map expression. The template then uses `gallery.name` and `gallery.cover` instead of separate variables.

#### Path building note

The gallery's verbose path concatenation does not benefit from spread (it's string concat, not map merging):

```html
{{ site_data.site.media_path ~ '/' ~ site_data.site.gallery_path ~ '/' ~ path ~ '/' ~ item.thumb }}
```

This could be improved separately by precomputing a `gallery_base_path` variable once:

```html
{% set base = site_data.site.media_path ~ '/' ~ site_data.site.gallery_path ~ '/' ~ path ~ '/' %}
{{ url_for(path=base ~ item.thumb) }}
```

## Keep as-is (no Tera 2.0 replacement)

| What | Why it stays |
|------|-------------|
| `url_for`, `group`, `get_data_by_slug`, `get_gallery`, `source_link`, `*_display_name`, `get_posts` functions | Domain logic with no native Tera equivalent |
| `slugify` filter | No native slugify in Tera 2.0 (used 74+ times) |
| `striptags` filter | Removed from Tera 2.0, our regex impl is correct |
| `trim_start_matches` filter | Tera 2.0's `trim_start` only trims whitespace, not arbitrary patterns |
| `date` filter | Could use `tera-contrib` crate but not worth a dependency for one filter |
| `default_date_format` filter | Uses site-specific config, no native equivalent |

## Summary

| Feature | Replaces | Effort | Impact |
|---------|----------|--------|--------|
| Native slicing `[:3]` | `\| slice(end=3)` filter calls | Low | High - 6+ callsites |
| Ternary `x if cond else y` | 8 if/else blocks in card.html | Low | High - halves shortcode |
| Optional chaining `?.` | `if X and X.Y` guards | Low | Medium - cleaner conditionals |
| List comprehension | `remove_draft` filter | Medium | Medium - eliminates filter |
| `tera-contrib` date | Custom date filter | Low | Low - saves 20 lines of Rust |
