---
tags: docs, templates, reference, theming
description: Complete reference for Marmite's template system, including all available variables, functions, and customization options.
---

# Template Reference

Marmite uses the [Tera](https://tera.netlify.app/) templating engine, which provides a powerful and flexible way to customize your site's appearance. This reference covers all template variables, functions, and customization options available in Marmite.

## Template System Overview

### Template Engine
- **Tera**: Jinja2-inspired templating language
- **Inheritance**: Templates can extend and override parent templates
- **Includes**: Templates can include other templates
- **Filters**: Built-in and custom filters for data transformation
- **Functions**: Custom functions for complex operations

### Core Templates

Marmite provides four core templates that can be customized:

```
templates/
├── base.html          # Base template for all pages
├── content.html       # Individual content pages
├── list.html          # Content listing pages
└── group.html         # Grouped content pages (tags, authors, etc.)
```

## Template Variables

### Global Variables (Available in all templates)

#### Site Configuration
```html
{{ site.name }}                    <!-- Site name -->
{{ site.tagline }}                 <!-- Site tagline -->
{{ site.url }}                     <!-- Site URL -->
{{ site.language }}                <!-- Site language -->
{{ site.footer }}                  <!-- Site footer HTML -->
{{ site.pagination }}              <!-- Posts per page -->
{{ site.default_date_format }}     <!-- Default date format -->
{{ site.enable_search }}           <!-- Search enabled boolean -->
{{ site.enable_related_content }}  <!-- Related content enabled -->
{{ site.toc }}                     <!-- Table of contents enabled -->
{{ site.json_feed }}               <!-- JSON feed enabled -->
{{ site.publish_md }}              <!-- Markdown publishing enabled -->
{{ site.source_repository }}       <!-- Source repository URL -->
```

#### Site Data
```html
{{ site_data.posts }}              <!-- All posts -->
{{ site_data.pages }}              <!-- All pages -->
{{ site_data.tag.map }}            <!-- Posts grouped by tag -->
{{ site_data.archive.map }}        <!-- Posts grouped by year -->
{{ site_data.author.map }}         <!-- Posts grouped by author -->
{{ site_data.stream.map }}         <!-- Posts grouped by stream -->
{{ site_data.series.map }}         <!-- Posts grouped by series -->
{{ site_data.galleries }}          <!-- Gallery collections -->
```

#### Navigation
```html
{{ menu }}                         <!-- Navigation menu items -->
{{ language }}                     <!-- Site language -->
```

#### Fragment Content
```html
{{ announce }}                     <!-- Content from _announce.md -->
{{ header }}                       <!-- Content from _header.md -->
{{ hero }}                         <!-- Content from _hero.md -->
{{ sidebar }}                      <!-- Content from _sidebar.md -->
{{ footer }}                       <!-- Content from _footer.md -->
{{ comments }}                     <!-- Content from _comments.md -->
{{ htmlhead }}                     <!-- Content from _htmlhead.md -->
{{ htmltail }}                     <!-- Content from _htmltail.md -->
```

### Content-Specific Variables (content.html)

#### Content Object
```html
{{ content.title }}                <!-- Content title -->
{{ content.description }}          <!-- Content description -->
{{ content.slug }}                 <!-- URL slug -->
{{ content.html }}                 <!-- Rendered HTML content -->
{{ content.date }}                 <!-- Publication date -->
{{ content.authors }}              <!-- Author names array -->
{{ content.tags }}                 <!-- Tags array -->
{{ content.stream }}               <!-- Stream name -->
{{ content.series }}               <!-- Series name -->
{{ content.pinned }}               <!-- Pinned status boolean -->
{{ content.toc }}                  <!-- Table of contents HTML -->
{{ content.card_image }}           <!-- Social media card image -->
{{ content.banner_image }}         <!-- Banner image -->
{{ content.comments }}             <!-- Comments enabled boolean -->
{{ content.source_path }}          <!-- Source file path -->
{{ content.modified_time }}        <!-- Last modification time -->
```

#### Navigation
```html
{{ content.next }}                 <!-- Next post in stream/series -->
{{ content.previous }}             <!-- Previous post in stream/series -->
{{ content.back_links }}           <!-- Content linking to this post -->
```

**Note:** When both `series` and `stream` are set on content, series navigation takes precedence for next/previous links.

#### Extra Fields
```html
{{ content.extra.math }}           <!-- Enable MathJax -->
{{ content.extra.mermaid }}        <!-- Enable Mermaid diagrams -->
{{ content.extra.mermaid_theme }}  <!-- Mermaid theme -->
{{ content.extra.custom_field }}   <!-- Any custom frontmatter field -->
```

### List-Specific Variables (list.html)

#### Pagination
```html
{{ content_list }}                 <!-- Array of content for current page -->
{{ current_page }}                 <!-- Current page filename -->
{{ current_page_number }}          <!-- Current page number -->
{{ total_pages }}                  <!-- Total number of pages -->
{{ total_content }}                <!-- Total content count -->
{{ per_page }}                     <!-- Items per page -->
{{ previous_page }}                <!-- Previous page filename -->
{{ next_page }}                    <!-- Next page filename -->
```

### Group-Specific Variables (group.html)

#### Grouping
```html
{{ kind }}                         <!-- Group type: "tag", "author", "archive", "stream", "series" -->
{{ title }}                        <!-- Group page title -->
```

## Template Functions

### url_for()
Generate URLs with proper base URL handling:

```html
<!-- Basic usage -->
<a href="{{ url_for(path='about.html') }}">About</a>

<!-- Absolute URL -->
<a href="{{ url_for(path='about.html', abs=true) }}">About</a>

<!-- External links (passed through unchanged) -->
<a href="{{ url_for(path='https://example.com') }}">External</a>
```

### group()
Access grouped content with optional sorting and limiting:

```html
<!-- Get all tags -->
{% for tag_name, tag_posts in group(kind="tag") %}
  <h3>{{ tag_name }}</h3>
  <ul>
    {% for post in tag_posts %}
      <li><a href="{{ post.slug }}.html">{{ post.title }}</a></li>
    {% endfor %}
  </ul>
{% endfor %}

<!-- Get all authors with sorting and limiting -->
{% for author_name, author_posts in group(kind="author", ord="desc", items=5) %}
  <h3>{{ author_name }}</h3>
  <p>{{ author_posts | length }} posts</p>
{% endfor %}

<!-- Get all streams -->
{% for stream_name, stream_posts in group(kind="stream") %}
  <h3>{{ stream_display_name(stream=stream_name) }}</h3>
  <ul>
    {% for post in stream_posts %}
      <li><a href="{{ post.slug }}.html">{{ post.title }}</a></li>
    {% endfor %}
  </ul>
{% endfor %}

<!-- Get archive years -->
{% for year, year_posts in group(kind="archive") %}
  <h3>{{ year }}</h3>
  <p>{{ year_posts | length }} posts</p>
{% endfor %}

<!-- Get all series with limiting -->
{% for series_name, series_posts in group(kind="series", items=10) %}
  <h3>{{ series_display_name(series=series_name) }}</h3>
  <ul>
    {% for post in series_posts %}
      <li><a href="{{ post.slug }}.html">{{ post.title }}</a></li>
    {% endfor %}
  </ul>
{% endfor %}
```

**Parameters:**
- `kind`: Required. One of "tag", "author", "archive", "stream", "series"
- `ord`: Optional. Sort order: "asc" or "desc" (default: "asc")
- `items`: Optional. Maximum number of groups to return (default: all)

### get_data_by_slug()
Retrieve standardized data for any content by its slug:

```html
<!-- Get data for a specific post -->
{% set data = get_data_by_slug(slug="my-blog-post") %}
<div class="content-card">
  <img src="{{ data.image }}" alt="{{ data.title }}">
  <h3>{{ data.title }}</h3>
  <p>{{ data.text }}</p>
  <small>{{ data.content_type }}</small>
</div>

<!-- Get data for a tag -->
{% set tag_data = get_data_by_slug(slug="tag-javascript") %}
<div class="tag-info">
  <h3>{{ tag_data.title }}</h3>
  <p>{{ tag_data.text }}</p>
</div>

<!-- Get data for an author -->
{% set author_data = get_data_by_slug(slug="author-rochacbruno") %}
<div class="author-profile">
  <img src="{{ author_data.image }}" alt="{{ author_data.title }}">
  <h3>{{ author_data.title }}</h3>
  <p>{{ author_data.text }}</p>
</div>

<!-- Get data for a series -->
{% set series_data = get_data_by_slug(slug="series-python-tutorial") %}
<div class="series-info">
  <h3>{{ series_data.title }}</h3>
  <p>{{ series_data.text }}</p>
</div>
```

**Returns SlugData object with:**
- `image`: Content banner/avatar image or placeholder
- `slug`: The content slug
- `title`: Content title, author name, or group name
- `text`: Content description, date, or post count
- `content_type`: Type identifier ("post", "page", "tag", "author", "series", "stream", "archive")

**Supported slug patterns:**
- Posts: `"post-slug"`
- Pages: `"page-slug"`
- Tags: `"tag-tagname"`
- Authors: `"author-username"`
- Series: `"series-seriesname"`
- Streams: `"stream-streamname"`
- Archives: `"archive-year"`

### stream_display_name()
Get friendly display names for streams:

```html
<!-- Show display name for current content's stream -->
{% if content.stream %}
  <span class="stream-name">{{ stream_display_name(stream=content.stream) }}</span>
{% endif %}

<!-- Use in stream listing -->
{% for stream_name, stream_posts in group(kind="stream") %}
  <h3>{{ stream_display_name(stream=stream_name) }}</h3>
  <p>{{ stream_posts | length }} posts in {{ stream_display_name(stream=stream_name) }}</p>
{% endfor %}

<!-- Link to stream page with friendly name -->
<a href="{{ content.stream }}.html">
  View all {{ stream_display_name(stream=content.stream) }} posts
</a>
```

**Configuration:**
```yaml
streams:
  tutorial:
    display_name: "Python Tutorials"
  news:
    display_name: "Latest News"
  guide:
    display_name: "User Guides"
```

If no display name is configured, returns the stream name itself.

### series_display_name()
Get friendly display names for series:

```html
<!-- Show display name for current content's series -->
{% if content.series %}
  <span class="series-name">{{ series_display_name(series=content.series) }}</span>
{% endif %}

<!-- Use in series listing -->
{% for series_name, series_posts in group(kind="series") %}
  <h3>{{ series_display_name(series=series_name) }}</h3>
  <p>{{ series_posts | length }} posts in {{ series_display_name(series=series_name) }}</p>
{% endfor %}

<!-- Link to series page with friendly name -->
<a href="serie-{{ content.series }}.html">
  View all {{ series_display_name(series=content.series) }} posts
</a>
```

**Configuration:**
```yaml
series:
  python-tutorial:
    display_name: "Python Tutorial"
  web-dev-basics:
    display_name: "Web Development Basics"
```

If no display name is configured, returns the series name itself.

### get_gallery()
Get gallery data by path:

```html
<!-- Get a specific gallery -->
{% set gallery = get_gallery(path="summer-2024") %}
{% if gallery %}
  <div class="gallery-preview">
    <h3>{{ gallery.name }}</h3>
    <img src="{{ url_for(path=site.media_path ~ '/' ~ site.gallery_path ~ '/' ~ 'summer-2024' ~ '/' ~ gallery.cover) }}" alt="Gallery cover">
    <p>{{ gallery.files | length }} photos</p>
  </div>
{% endif %}

<!-- Loop through gallery images -->
{% set gallery = get_gallery(path="vacation") %}
{% if gallery %}
  <div class="photo-grid">
    {% for item in gallery.files %}
      <div class="photo">
        <img src="{{ url_for(path=site.media_path ~ '/' ~ site.gallery_path ~ '/' ~ 'vacation' ~ '/' ~ item.thumb) }}" 
             data-full="{{ url_for(path=site.media_path ~ '/' ~ site.gallery_path ~ '/' ~ 'vacation' ~ '/' ~ item.image) }}"
             alt="Gallery photo">
      </div>
    {% endfor %}
  </div>
{% endif %}

<!-- Check gallery order -->
{% set gallery = get_gallery(path="portfolio") %}
{% if gallery %}
  <!-- Gallery files are sorted according to gallery.ord (asc/desc) -->
  <p>Gallery sorted: {{ gallery.ord }}</p>
{% endif %}
```

**Gallery object properties:**
- `name`: Display name of the gallery
- `files`: Array of gallery items
  - `thumb`: Thumbnail filename (in thumbnails/ subdirectory)
  - `image`: Full size image filename
- `cover`: Cover image filename
- `ord`: Sort order ("asc" or "desc")

**Gallery configuration:**
Each gallery folder can have a `gallery.yaml` file:
```yaml
name: "Summer Vacation 2024"
ord: desc
cover: "sunset.jpg"
```

### get_posts()
Get filtered and sorted posts:

```html
<!-- Get latest 5 posts -->
{% for post in get_posts(items=5) %}
  <article>
    <h3><a href="{{ post.slug }}.html">{{ post.title }}</a></h3>
    <time>{{ post.date | default_date_format }}</time>
  </article>
{% endfor %}

<!-- Get all posts in ascending order (oldest first) -->
{% for post in get_posts(ord="asc") %}
  <li>{{ post.title }}</li>
{% endfor %}

<!-- Get latest 10 posts for RSS feed -->
{% set recent_posts = get_posts(items=10) %}
```

**Parameters:**
- `ord`: Optional. Sort order: "asc" or "desc" (default: "desc")
- `items`: Optional. Maximum number of posts to return (default: all)

### source_link()
Generate source file links:

```html
<!-- Generate source link for current content -->
{% set source_url = source_link(content=content) %}
{% if source_url %}
  <a href="{{ source_url }}" rel="nofollow">📄 View source</a>
{% endif %}
```

## Template Filters

### default_date_format
Format dates using the site's default format:

```html
{{ content.date | default_date_format }}
```

### remove_draft
Filter out draft content from arrays:

```html
<!-- Remove draft posts from a list -->
{% for post in site_data.posts | remove_draft %}
  <li>{{ post.title }}</li>
{% endfor %}

<!-- Get count of non-draft items -->
{{ items | remove_draft | length }}

<!-- Use with group function results -->
{% for name, items in group(kind="tag") %}
  <h3>{{ name }} ({{ items | remove_draft | length }} posts)</h3>
  {% for post in items | remove_draft %}
    <li>{{ post.title }}</li>
  {% endfor %}
{% endfor %}
```

### Built-in Tera Filters
Marmite includes all standard Tera filters:

```html
<!-- String filters -->
{{ content.title | upper }}
{{ content.description | truncate(length=100) }}
{{ content.slug | replace(from="-", to="_") }}

<!-- Array filters -->
{{ content.tags | join(sep=", ") }}
{{ content_list | slice(end=5) }}
{{ content_list | length }}

<!-- Date filters -->
{{ content.date | date(format="%Y-%m-%d") }}

<!-- URL filters -->
{{ content.slug | urlencode }}

<!-- Utility filters -->
{{ content.html | striptags }}
{{ content.title | slugify }}
```

## Template Inheritance

### Base Template Pattern
```html
<!-- base.html -->
<!DOCTYPE html>
<html lang="{{ language }}">
<head>
    <title>{% block title %}{{ site.name }}{% endblock %}</title>
    {% block head %}{% endblock %}
</head>
<body>
    <header>
        {% block header %}
        <h1>{{ site.name }}</h1>
        {% endblock %}
    </header>
    
    <main>
        {% block main %}{% endblock %}
    </main>
    
    <footer>
        {% block footer %}
        {{ site.footer }}
        {% endblock %}
    </footer>
    
    {% block tail %}{% endblock %}
</body>
</html>
```

### Extending Templates
```html
<!-- content.html -->
{% extends "base.html" %}

{% block title %}{{ content.title }} | {{ site.name }}{% endblock %}

{% block main %}
<article>
    <h1>{{ content.title }}</h1>
    {{ content.html }}
</article>
{% endblock %}
```

## Template Includes

### Including Other Templates
```html
<!-- Include fragments -->
{% include "content_title.html" %}
{% include "content_date.html" %}
{% include "pagination.html" %}

<!-- Conditional includes -->
{% include "comments.html" ignore missing %}
{% include "custom_sidebar.html" ignore missing %}
```

## Custom Template Creation

### Content-Specific Templates
Create templates for specific content types:

```html
<!-- templates/custom_tutorials.html -->
{% extends "base.html" %}

{% block main %}
<article class="tutorial">
    <div class="tutorial-header">
        <h1>{{ content.title }}</h1>
        <div class="tutorial-meta">
            <span>Difficulty: {{ content.extra.difficulty }}</span>
            <span>Duration: {{ content.extra.duration }}</span>
        </div>
    </div>
    
    {% if content.toc %}
    <div class="tutorial-toc">
        <h2>Table of Contents</h2>
        {{ content.toc }}
    </div>
    {% endif %}
    
    <div class="tutorial-content">
        {{ content.html }}
    </div>
</article>
{% endblock %}
```

### Stream-Specific Templates
Templates for specific streams:

```html
<!-- templates/custom_news.html -->
{% extends "list.html" %}

{% block main %}
<div class="news-header">
    <h1>Latest News</h1>
    <p>Stay updated with our latest announcements</p>
</div>

<div class="news-grid">
    {% for post in content_list %}
    <article class="news-item">
        <h2><a href="{{ post.slug }}.html">{{ post.title }}</a></h2>
        <time>{{ post.date | default_date_format }}</time>
        <p>{{ post.description }}</p>
    </article>
    {% endfor %}
</div>
{% endblock %}
```

### Series-Specific Templates
Templates for specific series with chronological navigation:

```html
<!-- templates/custom_series.html -->
{% extends "base.html" %}

{% block main %}
<div class="series-header">
    <h1>{{ series_display_name(series=content.series) }}</h1>
    <p>Part {{ content.series_position }} of {{ content.series_total }}</p>
</div>

<article class="series-content">
    <h2>{{ content.title }}</h2>
    {{ content.html }}
</article>

<nav class="series-navigation">
    {% if content.previous %}
    <a href="{{ content.previous.slug }}.html" class="prev-post">
        ← Previous: {{ content.previous.title }}
    </a>
    {% endif %}
    
    {% if content.next %}
    <a href="{{ content.next.slug }}.html" class="next-post">
        Next: {{ content.next.title }} →
    </a>
    {% endif %}
</nav>

<div class="series-toc">
    <h3>All Parts in This Series</h3>
    <ul>
        {% for post in site_data.series.map[content.series] %}
        <li {% if post.slug == content.slug %}class="current"{% endif %}>
            <a href="{{ post.slug }}.html">{{ post.title }}</a>
        </li>
        {% endfor %}
    </ul>
</div>
{% endblock %}
```

## Advanced Template Features

### Conditional Content
```html
<!-- Show content based on conditions -->
{% if content.stream == "tutorials" %}
<div class="tutorial-notice">
    This is a tutorial post. Follow along step by step!
</div>
{% endif %}

<!-- Show series navigation -->
{% if content.series %}
<div class="series-notice">
    Part of the <a href="serie-{{ content.series }}.html">{{ series_display_name(series=content.series) }}</a> series
</div>
{% endif %}

<!-- Show content based on site configuration -->
{% if site.enable_search %}
<div class="search-box">
    <input type="search" placeholder="Search...">
</div>
{% endif %}
```

### Loops and Iteration
```html
<!-- Loop through tags -->
{% for tag in content.tags %}
<span class="tag">{{ tag }}</span>
{% endfor %}

<!-- Loop through authors -->
{% for author in content.authors %}
<span class="author">{{ author }}</span>
{% endfor %}

<!-- Loop with conditions -->
{% for post in content_list %}
    {% if post.pinned %}
    <article class="pinned-post">
    {% else %}
    <article class="regular-post">
    {% endif %}
        <h2>{{ post.title }}</h2>
    </article>
{% endfor %}
```

### Template Comments
```html
{# This is a template comment - won't appear in output #}
{# 
Multi-line comments
are also supported
#}
```

## Template Debugging

### Debug Variables
```html
<!-- Debug site data -->
<pre>{{ site | json_encode(pretty=true) }}</pre>

<!-- Debug content -->
<pre>{{ content | json_encode(pretty=true) }}</pre>

<!-- Debug specific values -->
<p>Debug: {{ content.stream | default(value="no stream") }}</p>
```

### Template Errors
Common template errors and solutions:

- **Variable not found**: Use `{{ var | default(value="fallback") }}`
- **Filter not found**: Check filter name spelling
- **Template not found**: Verify template file exists
- **Syntax error**: Check bracket matching and syntax

## Best Practices

1. **Use template inheritance**: Extend base templates for consistency
2. **Keep templates DRY**: Use includes for repeated content
3. **Handle missing data**: Use default filters for optional fields
4. **Comment your templates**: Document complex logic
5. **Test thoroughly**: Verify templates work with different content types
6. **Use semantic HTML**: Structure content meaningfully
7. **Optimize for performance**: Minimize template complexity

This comprehensive reference covers all aspects of Marmite's template system. Use these features to create beautiful, functional, and maintainable themes for your site.