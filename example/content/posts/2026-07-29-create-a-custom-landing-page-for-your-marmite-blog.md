---
date: 2026-07-29
title: "Create a custom landing page for your marmite blog"
tags: tutorial, customization
toc: true
authors: rochacbruno
description: "Learn how to replace the default post list with a fully custom landing page using index.md and shortcodes."
---

By default, marmite renders your most recent posts on the front page. That works well for most blogs, but sometimes you want something different - a welcome section, a curated selection of posts, a call to action, or a mix of content pulled together in a layout you control.

This tutorial shows how to do that with two built-in features: `index.md` and shortcodes.

## How it works

When marmite finds a file named `index.md` in your content folder, it uses that file as the body of your front page (`/index.html`). The default post list is replaced by whatever you write in that file - plain markdown, shortcodes, or a combination of both.

## Step 1: Create index.md

Inside your content folder, create a file named `index.md`:

```bash
touch content/index.md
```

Write whatever you want on your front page. You can use regular markdown, and you can embed dynamic content with shortcodes:

```markdown
---
comments: false
---

# Welcome to my blog

I write about Rust, Python, and whatever else I find interesting.

## Latest posts

<!-- .posts items=5 -->

## All tags

<!-- .tags -->
```

Rebuild and serve to see it live:

```bash
marmite . --serve -w
```

Your front page will now show your custom content instead of the default paginated post list.

## Step 2: Add a custom shortcode (optional)

The built-in shortcodes (`<!-- .posts -->`, `<!-- .tags -->`, `<!-- .authors -->`, etc.) cover common patterns, but you can also write your own to display anything you like.

Create a `shortcodes/` folder in your project root and add a `.html` file. Each shortcode file must define a Tera macro with the same name as the file:

`shortcodes/welcome.html`

```html
{% macro welcome() %}
<section class="welcome-box">
  <h2>Hello, I'm {{ site.authors[site.default_author].name }}</h2>
  <p>{{ site.tagline }}</p>
</section>
{% endmacro %}
```

You have access to `site` and all its data - authors, config, extra fields - inside a shortcode macro.

Verify the shortcode is valid before using it:

```bash
marmite . --shortcodes
```

No errors means the shortcode is ready. Use it in `index.md`:

```markdown
<!-- .welcome -->
```

## What you can use in index.md

Any built-in shortcode works in `index.md`:

| Shortcode | What it renders |
|-----------|----------------|
| `<!-- .posts items=5 -->` | Latest N posts |
| `<!-- .pages -->` | All pages |
| `<!-- .tags -->` | Tag cloud |
| `<!-- .authors -->` | Author list |
| `<!-- .streams -->` | Stream list |
| `<!-- .series -->` | Series list |

You can also use frontmatter fields to control per-page behavior. For example, `comments: false` disables the comment section on the landing page, and `toc: false` hides the table of contents if your theme shows one.

## Full example

Here is a complete `index.md` that combines a personal greeting shortcode, a post list, and a tag cloud:

```markdown
---
comments: false
---

<!-- .welcome -->

## Recent posts

<!-- .posts items=5 -->

## Browse by tag

<!-- .tags ord=asc -->
```

With `shortcodes/welcome.html` defined as shown above, this produces a landing page that feels personal without requiring any template edits.

## Notes

- `index.md` does not appear in the post list or feeds - it only affects `/index.html`.
- Pagination is disabled when `index.md` is present; the page is rendered once as a static layout.
- If you want to keep showing all posts, include `<!-- .posts -->` without an `items` limit.
- The `--shortcodes` flag is useful for catching syntax errors in shortcode files before serving.
