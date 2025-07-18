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
{{ content.next }}                 <!-- Next post in stream -->
{{ content.previous }}             <!-- Previous post in stream -->
{{ content.back_links }}           <!-- Content linking to this post -->
```

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
{{ kind }}                         <!-- Group type: "tag", "author", "archive", "stream" -->
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
Access grouped content:

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

<!-- Get all authors -->
{% for author_name, author_posts in group(kind="author") %}
  <h3>{{ author_name }}</h3>
  <p>{{ author_posts | length }} posts</p>
{% endfor %}

<!-- Get all streams -->
{% for stream_name, stream_posts in group(kind="stream") %}
  <h3>{{ stream_name }}</h3>
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
```

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

## Advanced Template Features

### Conditional Content
```html
<!-- Show content based on conditions -->
{% if content.stream == "tutorials" %}
<div class="tutorial-notice">
    This is a tutorial post. Follow along step by step!
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