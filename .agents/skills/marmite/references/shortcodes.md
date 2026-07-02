# Marmite Shortcodes Reference

Shortcodes are reusable content blocks that can be embedded in markdown files. They are processed during site generation and replaced with their rendered output.

## Shortcode Syntax

The default syntax uses HTML comments:

```markdown
<!-- .shortcode_name -->
<!-- .shortcode_name param1=value1 param2=value2 -->
```

This can be changed via the `shortcode_pattern` config field.

## Built-in Shortcodes

### youtube

Embed a YouTube video:

```markdown
<!-- .youtube id=dQw4w9WgXcQ -->
<!-- .youtube id=dQw4w9WgXcQ width=800 height=450 -->
```

Parameters:
- `id` (required): YouTube video ID or full URL
- `width` (optional, default `560`): Player width in pixels
- `height` (optional, default `315`): Player height in pixels

### spotify

Embed a Spotify player:

```markdown
<!-- .spotify uri=spotify:track:6rqhFgbbKwnb9MLmUQDhG6 -->
<!-- .spotify uri=spotify:track:6rqhFgbbKwnb9MLmUQDhG6 width=300 height=380 -->
```

Parameters:
- `uri` (required): Spotify URI
- `width` (optional, default `300`): Player width
- `height` (optional, default `380`): Player height

### posts

List recent posts:

```markdown
<!-- .posts -->
<!-- .posts ord=asc items=5 -->
```

Parameters:
- `ord` (optional, default `desc`): Sort order - `asc` or `desc`
- `items` (optional, default `10`): Maximum number of posts to show

### pages

List all pages:

```markdown
<!-- .pages -->
<!-- .pages ord=asc -->
```

Parameters:
- `ord` (optional, default `desc`): Sort order
- `items` (optional, default shows all): Maximum items

### tags

List all tags with post counts:

```markdown
<!-- .tags -->
<!-- .tags ord=asc items=20 -->
```

Parameters:
- `ord` (optional, default `desc`): Sort order - `desc` sorts by count, `asc` alphabetically
- `items` (optional, default shows all): Maximum tags

### streams

List all streams:

```markdown
<!-- .streams -->
<!-- .streams ord=asc items=5 -->
```

Parameters:
- `ord` (optional, default `desc`): Sort order
- `items` (optional, default shows all): Maximum streams

### series

List all series:

```markdown
<!-- .series -->
<!-- .series ord=desc items=5 -->
```

Parameters:
- `ord` (optional, default `desc`): Sort order
- `items` (optional, default shows all): Maximum series

### authors

List all authors with avatars:

```markdown
<!-- .authors -->
```

### card

Display a content card with image, title, and description:

```markdown
<!-- .card slug=my-post-slug -->
<!-- .card slug=series-python-tutorial -->
<!-- .card slug=https://example.com image=https://example.com/img.jpg title=External text=Description -->
```

Parameters:
- `slug` (required): Content slug, prefixed slug (`series-*`, `stream-*`, `tag-*`, `author-*`, `archive-*`), or external URL
- `image` (optional): Override image URL
- `title` (optional): Override title
- `text` (optional): Override description text
- `content_type` (optional): Override content type label

### gallery

Display an image gallery:

```markdown
<!-- .gallery path=photos -->
<!-- .gallery path=photos width=200 height=200 ord=asc -->
<!-- .gallery path=photos name=My Photos cover=photo1.jpg -->
```

Parameters:
- `path` (required): Gallery folder path (relative to gallery_path config)
- `width` (optional): Thumbnail width
- `height` (optional): Thumbnail height
- `ord` (optional): Image sort order
- `name` (optional): Gallery display name
- `cover` (optional): Cover image filename

### toc

Insert a table of contents at this position:

```markdown
<!-- .toc -->
```

No parameters. Generates a TOC from the document's headings.

### socials

Display social network links (markdown shortcode):

```markdown
<!-- .socials -->
```

Renders social network links from the site's `extra.social_networks` config.

## Creating Custom Shortcodes

Place shortcode files in the `shortcodes/` directory at the project root.

### HTML Shortcodes

HTML shortcodes use `{% shortcode %}` definitions. The filename (without extension) becomes the shortcode name, and the file must contain a definition with the same name. The older `{% macro %}` syntax also works for backward compatibility.

Example: `shortcodes/alert.html`
```html
{% shortcode alert(type="info", message="") %}
<div class="alert alert-{{ type }}">
  <strong>{{ type | upper }}:</strong> {{ message }}
</div>
{% endshortcode alert %}
```

Usage:
```markdown
<!-- .alert type=warning message=This is important! -->
```

Example: `shortcodes/button.html`
```html
{% shortcode button(url="#", label="Click", style="primary") %}
<a href="{{ url }}" class="btn btn-{{ style }}">{{ label }}</a>
{% endshortcode button %}
```

Usage:
```markdown
<!-- .button url=https://example.com label=Visit style=secondary -->
```

### Markdown Shortcodes

Markdown shortcodes are `.md` files that use Tera template variables. The rendered output is treated as markdown.

Example: `shortcodes/note.md`
```markdown
> **{{ title | default(value="Note") }}**: {{ content | default(value="") }}
```

Usage:
```markdown
<!-- .note title=Important content=Remember to save your work -->
```

### Shortcode with Site Data Access

HTML shortcodes have full access to the rendering context, including `site_data`, `site`, `content` (on content pages), and all registered Tera functions (`url_for`, `group`, `get_posts`, etc.).

```html
{% shortcode featured(count=3) %}
{% set posts = get_posts(ord="desc", items=count) %}
<div class="featured">
  {% for post in posts %}
    <article>
      <h3><a href="{{ url_for(path=post.slug ~ '.html') }}">{{ post.title }}</a></h3>
      {% if post.description %}<p>{{ post.description }}</p>{% endif %}
    </article>
  {% endfor %}
</div>
{% endshortcode featured %}
```

### Overriding Built-in Shortcodes

To override a built-in shortcode, create a file with the same name in your `shortcodes/` directory. Your version takes precedence.

## Shortcode Configuration

### Enable/Disable

```yaml
# In marmite.yaml
enable_shortcodes: true   # default
```

### Custom Pattern

Change the shortcode invocation syntax:

```yaml
# Hugo-style: {{< name param=value >}}
shortcode_pattern: '\{\{<\s*(\w+)([^>]*)\s*>\}\}'

# Jekyll-style: {% name param=value %}
shortcode_pattern: '\{%\s*(\w+)([^%]*)\s*%\}'
```

### Listing Available Shortcodes

```bash
marmite <folder> --shortcodes
```

This shows all available shortcodes with their descriptions and usage examples.

## Parameter Handling

- Parameters are passed as `key=value` pairs separated by spaces
- String values do not need quotes: `message=Hello World` works
- Boolean values: `enabled=true`
- Numeric values: `count=5`
- Default values are specified in the shortcode definition: `{% shortcode name(param="default") %}`
