# Marmite

![Logo](https://github.com/rochacbruno/marmite/raw/main/assets/_resized/logo_160x120.png)

Marmite [**Mar**kdown **m**akes s**ite**s] is a **very!** simple static site generator.

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)
[![Crates.io Version](https://img.shields.io/crates/v/marmite)](https://crates.io/crates/marmite)
[![Docs and Demo](https://img.shields.io/badge/docs-demo-blue)](https://rochacbruno.github.io/marmite/)  
  
[![Create blog](https://img.shields.io/badge/CREATE%20YOUR%20BLOG%20WITH%20ONE%20CLICK-20B2AA?style=for-the-badge)](https://github.com/rochacbruno/make-me-a-blog)

## How it works

It does **"one"** simple thing only:

- Reads all `.md` files on the `input` directory.
- Using `CommonMark` parse it to `HTML` content.
- Extract optional metadata from `frontmatter` or `filename`.
- Generated `html` file for each page (templates are customizable).
- Outputs the rendered static site to the `output` folder.

It also handles generating or copying `static/` `media/` to the `output` dir.

## Installation

Install with cargo

```bash
cargo install marmite
```

Or download the pre-built binary from the [releases](https://github.com/rochacbruno/marmite/releases)


## Usage

~It's simple, really!

```console
$ marmite folder_with_markdown_files path_to_generated_site
Site generated at path_to_generated_site/
```

CLI

```console
❯ marmite --help
MARkdown Makes sITEs is a very simple static site generator, mainly for simple blogs.

Usage: marmite [OPTIONS] <INPUT_FOLDER> <OUTPUT_FOLDER>

Arguments:
  <INPUT_FOLDER>   Input folder containing markdown files
  <OUTPUT_FOLDER>  Output folder to generate the site

Options:
      --serve            Serve the site with a built-in HTTP server
      --watch            Watch for file changes and rebuild site
      --bind <BIND>      [default: localhost:8000]
      --config <CONFIG>  Path to custom configuration file (defaults to marmite.yaml) [default: marmite.yaml]
      --debug            Print debug messages
  -h, --help             Print help
  -V, --version          Print version
```

### Build a site from markdown content

Put some markdown in a folder.

```console
myblog
|__ about.md
|__ 2024-01-31-my-first-post.md
|__ another-post.md
```

Or use your favorite markdown editor to edit.

Then, build:

```console
$ marmite myblog site

building index.html
building pages.html
building tags.html
building archive.html
building feed.rss
building feed.json
building 404.html

building my-page.html
building my-first-post.html
building another-post.html

Site generated at: site/
```

### Result

Site is generated from embedded templates that are purposely very simple!

- **Front page** lists all blog posts (markdown containing `date` attribute)
- **Menu** is shown with links to `pages`, `tags`, `archive` (menu is customizable)
- **Feeds** are generated as RSS and JSON formats
- **Content** pages are generated from every `.md` file

Deploy `site/` folder to your favorite webserver.

Open `index.html` on your web-browser or run `marmite myblog site --serve` to run
the embedded webserver.

### Screenshots

<details>
<summary> CLICK HERE TO SEE SOME SCREENSHOTS </summary>

The following screenshots are using the default embedded
templates (from [/example](https://github.com/rochacbruno/marmite/blob/main/example) folder)

**Light Mode**

Index:

![Index Light](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/index-light.png)

Content:

![Post Light](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/post-light.png)

**Dark mode**

Index:

![Index Dark](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/index-dark.png)

Content:

![Post Dark](https://github.com/rochacbruno/marmite/raw/main/assets/screenshots/post-dark.png)

</details>


## Customization

Marmite allows customization of the website using custom `templates` that
are written using `Tera` template language (similar to Jinja and Twig).

Site metadata can be customized on `marmite.yaml`.

### Folder structure

```plain
myblog
├── marmite.yaml        # Site configuration
├── content
│   └── *.md            # Site content
├── static
│   └── *.css|js|ttf    # Static files (CSS, JS, Fonts)
└── templates
    ├── base.html       # Common HTML
    ├── content.html    # Renders page and post
    ├── group.html      # Archive and tags list
    └── list.html       # Renders index, tags, archive
```

### Optional configuration

the `marmite.yaml` is optional, you simply omit and use defaults.

> All keys are optional, but you probably want to set at least `name`,`tagline`, `url`

`marmite.yaml`
```yaml
name: My Blog
tagline: This blog is awesome
url: https://www.myblog.com
# footer: This is an example site generated with Marmite
# pagination: 10

# list_title: Blog Posts
# tags_title: Tags
# archive_title: Archive

# templates_path: templates
# static_path: static
# media_path: media

# menu:
#  - ["Title", "link.html"]
# data: Custom key:pair values to be exposed to template context.
```

### Content types

- **Post**: If `.md` has a `date` property on filename prefix or `frontmatter` it is considered a post to show in the index.html posts list.
- **Page**: If it does not have a `date` it is considered a page, listed on pages.html and acessible via direct link.

### Metadata

FrontMatter can optionally be specified in `yaml` format.

```yaml
---
date: "2024-01-01"
title: Title
slug: title
tags: comma,separated,tags
tags:
  - yaml
  - list
  - also
  - works
---
# Title of my content

Content Text ...
```

- **date**: If informed, the content is considered a `Post` and shows in index list.
  - Date formats supported are `%Y-%m-%d` and `%Y-%m-%d %H:%M`
  - Date can be defined as `date` on frontmatter or in the filename `%Y-%m-%d-name.md`
- **title**: If not defined, the first line of the markdown content is used.
- **slug**: If not defined, the title or filename is used to build the url.
- **tags**: Optional list or comma separated string of tags.

All fields are optional, if nothing is declared then the content is considered an unlisted `Page`.

### Example content

`content/first-post.md`
````markdown
---
date: "2024-01-01"
title: My First Blog Post
slug: my-first-blog-post
tags: poetry,life
---

# Hello this is my first post

This blog was generated by `Marmite` the simplest static site generator

## Images

![local image](/media/simple.png)

I can also have remote images

![remote image](https://github.com/rochacbruno/marmite/logo.png)


## Code Snippets

```python
de foo():
    return "bar"
```

Everything CommonMark and Github flavoured markdown supports.

</strong>Raw HTML also allowed</strong>
````

## editors and deployment

Marmite does not come with an editor but as the content is simply markdown files, **any** text editor will work!

Generated site is pure HTML + CSS so it can be served on **any** static webserver.

Workflow is generally:

- Create a new `content/your-content-title.md`.
- Edit in your preferred **editor** (There are some with good preview support).
- Add the `date` metadata when it is ready to publish.
- commit to your preferred **repository**.
- use your preferred **automation system** to publish it to your preferred **web server**.

Common examples:

- Edit in **Marktext** Editor, configure it to move pasted images to `media`, commit to Github, Add an action to build and Publish as a Github Page.
- Edit directly in the Github Web UI, commit, let CI Action to build and publish.
- Edit in **vim**, Generate the site locally, Publish via FTP.
- Edit in **VsCode**, Commit to Git Repository, Have the CI to build and Publish (GH pages, netlify etc)

> Marmite focus on generating the site from markdown only, the deployment and media management is a separate problem to solve.

## That's all!

**Marmite** is very simple, and limited in functionality, there is no intention to add more features or built-in themes.

If this simplicity does not suit your needs, there are other awesome static site generators.


Here are some that I recommend:

- [Cobalt](https://cobalt-org.github.io/)
- [Zola](https://www.getzola.org/)
