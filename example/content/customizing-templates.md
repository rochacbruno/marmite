---
date: 2024-10-18
tags: docs, templates, theme, customization
---
# Customizing Templates

Marmite uses [Tera](https://keats.github.io/tera/docs/#templates) as its template
parser, the language is very similar to **Jinja** or **Twig**.

> [!IMPORTANT]  
> always link relative to the current path starting with `./`  
> If absolute url is needed then use `{{ url_for(path="path", abs=true) }}` template function.

Example on `templates/list.html`

```html
{% extends "base.html" %}
{% block main %}
  <div class="content-list">
  {%- for content in content_list %}
    <h2 class="content-title"><a href="./{{content.slug}}.html">{{ content.title | capitalize }}</a></h2>
    <p class="content-excerpt">{{ content.html | striptags | truncate(length=100, end="...") }}</p>
  {%- endfor %}
  </div>
{% endblock %}
```

## Templates

all templates are rendered with the global context.

```yaml
site_data:
  posts: [Content]
  pages: [Content]
site:
  name: str
  url: str
  tagline: str
  pagination: int
  extra: {k, v}
  ...the keys on site configuration.
menu: [[name, link]]
```

The `Content` object can be a **page** or a **post** and contains

```yaml
title: str
slug: str
html: str
tags: [str] or []
date: DateTimeObject or None
extra: {key: value}
```

There are 6 templates inside the `templates` folder, each adds more data to context.

- base.html
  - All other templates inherits blocks from this one.
- list.html
  - Renders `index.html`, `pages.html`, `tags.html`
  - adds `title:str`, `content_list: [Content]`, 
  - pagination: `current_page: str`, `next_page:str`, `previous_page:str`, 
    `total_pages:int`, `current_page_number:int`, `total_content:int`
- content.html
  - Renders individual content page `my-post.html`
  - adds `title:str`, `content: [Content]`, `current_page: str`
- group.html
  - Renders grouped information such as `tag/sometag.html` and `archive/2024.html`
  - adds `title:str`, `group_content: [[group, [Content]]]`, `current_page: str`

When customizing the templates you can create new templates to use as `include` or `macro`
but the 4 listed above are required.

If you just want to customize some individual template you can add only it in the
templates/ folder and the rest will be added by marmite.

See the templates on: [https://github.com/rochacbruno/marmite/tree/main/example/templates](https://github.com/rochacbruno/marmite/tree/main/example/templates)

## Static files

Just create a `static` folder side by side with your `templates` folder and add
all `css`, `js`, `fonts` etc on this folder.

Marmite will copy this folder to the output site, if this folder is not found
marmite will then copy the embedded static files to the static folder.

## URL 

Prefer to use relative paths for URLS, examples:

- `./my-blog-post.html`
- `./static/style.css`
- `./media/photo.png`

This is recommended because **marmite** will always generate a **flat** html website,
there is no subpaths.

If you need absolute url use the `url_for` function to refer to urls.

```html
{{ url_for(path='static/mystyle.css', abs=true) }}
```


## Extra data

On site config `marmite.yaml` there is an arbitrary field `extra` that can be accessed
on any template.

```yaml
extra:
  myname: Bruno
```
Then on an template.

```html
{{site.extra.myname}}
```

On each individual post there is a `extra` arbitrary field, so on `list.html` and
`content.html` this field can also be accessed.

```markdown
---
extra:
  banner_image: media/banner.jpg
---
```
then on template
```html
<img src="{{url_for(content.extra.banner_image)}}">
```

## Raw HTML on markdown

Tera is configured to allow raw html on markdown, so any html tag will be 
allowed, a markdown file can include for example embeds, scripts, etc..

## Tera Object

Here the Tera object for reference, 
to see each individual filter or tester documentation read [https://keats.github.io/tera/docs/#templates](https://keats.github.io/tera/docs/#templates)

```rust 
Tera {
    templates: [
            group.html,
            base.html,
            list.html,
            content.html,
    ]
    filters: [
            reverse,
            trim_start,
            trim,
            unique,
            float,
            trim_end,
            join,
            group_by,
            lower,
            trim_start_matches,
            wordcount,
            title,
            trim_end_matches,
            indent,
            filter,
            map,
            pluralize,
            upper,
            first,
            slice,
            last,
            round,
            length,
            date,
            filesizeformat,
            urlencode,
            urlencode_strict,
            nth,
            escape_xml,
            truncate,
            striptags,
            abs,
            linebreaksbr,
            spaceless,
            slugify,
            addslashes,
            capitalize,
            escape,
            split,
            int,
            concat,
            as_str,
            sort,
            replace,
            get,
            json_encode,
    ]
    testers: [
            number,
            object,
            ending_with,
            defined,
            odd,
            matching,
            even,
            containing,
            iterable,
            undefined,
            string,
            starting_with,
            divisibleby,
    ]
}
```
