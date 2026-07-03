# Marmite

<img src="https://github.com/rochacbruno/marmite/raw/main/assets/_resized/logo_160x120.png" align="left" alt="marmite">

Marmite [**Mar**kdown **m**akes s**ite**s] is a **very!** simple static site generator.

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)
[![Crates.io Version](https://img.shields.io/crates/v/marmite)](https://crates.io/crates/marmite)
[![Docs and Demo](https://img.shields.io/badge/docs-demo-blue)](https://marmite.blog/)  
  
[![Create blog](https://img.shields.io/badge/CREATE%20YOUR%20BLOG%20WITH%20ONE%20CLICK-20B2AA?style=for-the-badge)](https://github.com/rochacbruno/blog)

[![Playground](https://img.shields.io/badge/PLAYGROUND-20B2AA?style=for-the-badge)](https://play.marmite.blog)

> I'm a big user of other SSGs but it is frequently frustrating that it takes so much setup to get started.  
Just having a directory of markdown files and running a single command sounds really useful.  
&mdash; Michael, marmite user.

## How it works

It does **"one"** simple thing only:

- Reads all `.md` files on the `input` directory.
- Using `CommonMark` parse it to `HTML` content.
- Extract optional metadata from `frontmatter` or `filename`.
- Generated `html` file for each page.
- Outputs the rendered static site to the `output` folder.

It also handles generating or copying `static/` and `media/` to the `output` dir.

## Before you start, you should know

1. Marmite is meant to be simple, don't expect complex features
2. Marmite is for **bloggers**, so writing and publishing articles in chronological order is the main use case.
3. The generated static site is **flat** HTML by default (`./{name}.html|rss|json`). Workspaces can produce subdirectory-based multi-site layouts.
4. Taxonomies: `tags:`, `stream:`, `series:`, `authors:`, and `languages:` (i18n)
5. Marmite uses the `date:` attribute to differentiate `posts` from `pages`

## Features

- Everything embedded in a single binary.
- Zero-Config to get started.
  - Optionally fully configurable via `marmite.yaml`
  - Configurable markdown parser (CommonMark extensions, rendering options)
- Common-mark + Github Flavoured Markdown + Extensions.
- Raw HTML allowed.
- Emojis `:smile:`, spoiler `||secret||`.
- Wikilinks `[[name|url]]` and Obsidian links `[[page]]`.
- Markdown alerts (GitHub-style callouts).
- Backlinks and related content.
- Tags.
- Multi authors with profile pages.
- Multi streams (separate content in different listings).
- Series support (ordered, chronological content groups with navigation).
- Language streams / i18n (multilingual content with auto-discovery, translation links, hreflang SEO tags).
- Draft content management (filtered from feeds and search).
- Pagination.
- Static search index with inline match previews.
- RSS Feeds (index, tags, authors, streams, languages).
- Next/previous post navigation (stream-aware).
- Built-in HTTP server with WebSocket live reload.
- Auto rebuild when content changes.
- Shortcodes (YouTube, Spotify, cards, galleries, table of contents, custom templates).
- Image gallery with automatic thumbnail generation.
- Automatic image resizing (parallel processing, incremental builds, configurable quality).
- Media organization (slug-based subfolders, `@/` shorthand, content subfolder media).
- Automatic sitemap generation.
- `--show-urls` dry run to preview all site URLs without building.
- File mappings (copy arbitrary files during site generation).
- Redirect aliases (frontmatter `aliases` field generates redirect pages for old URLs).
- Internal link validation (build-time checking, optional strict failure mode).
- IndieWeb compliance (microformats, semantic HTML).
- Markdown source publishing alongside HTML.
- Built-in theme
  - Light and Dark modes.
  - Multiple colorschemes.
  - Fully responsive.
  - Spotlight search.
  - Easy to replace the index page and add custom CSS/JS.
  - Easy to customize Tera templates.
  - Math and Mermaid diagrams.
  - Build-time syntax highlighting via [arborium](https://arborium.bearcove.eu/).
  - Commenting system integration.
  - Banner images and `og:` tags.
- Theme system with remote theme installation.
- CLI to start a new theme from scratch.
- Workspace multi-site support (single command builds, config inheritance, cross-site references).
- AT Protocol / standard.site integration (publish posts to the decentralized social web).
- Embedded AI agent skills for AI-assisted site management.
- Available via cargo, pip/uvx, Homebrew, AUR, FreeBSD, Docker, and install script.


## Installation

Install with cargo

```bash
cargo binstall marmite
```
or

```bash
cargo install marmite
```

Or download the pre-built **binary** from the [releases](https://github.com/rochacbruno/marmite/releases)

### Alternative installation methods

<details>
<summary>Package managers</summary>

**Homebrew (macOS/Linux)**
```bash
brew install marmite
```
[View formula](https://formulae.brew.sh/formula/marmite)

**Arch Linux (AUR)**
```bash
yay -S marmite-bin
```
[View package](https://aur.archlinux.org/packages/marmite-bin)

**FreeBSD**
```bash
pkg install marmite
```
[View port](https://www.freshports.org/www/marmite/)

</details>


<details>

<summary>Or use docker</summary>


> [!IMPORTANT]  
> The directory containing your marmite project must be mapped to containers `/input`  
> If running inside the directory use `$PWD:/input` 
> The result will be generates in a `site` folder inside the input dir.

Build
```console
$ docker run -v $PWD:/input ghcr.io/rochacbruno/marmite
Site generated at: site/
```
Serve (just add port mapping and the --serve)
```console
$ docker run -p 8000:8000 -v $PWD:/input ghcr.io/rochacbruno/marmite --serve
```

> [!INFO]  
> By default will run `:latest`, Add `:x.y.z` with the version you want to run.

</details>

## Usage

It's simple, really!

```console
$ marmite folder_with_markdown_files path_to_generated_site
Site generated at path_to_generated_site/
```

CLI

```console
$ marmite --help
Marmite is the easiest static site generator.

Usage: marmite [OPTIONS] [INPUT_FOLDER] [OUTPUT_FOLDER] [COMMAND]

Commands:
  atproto  Manage atproto / standard.site integration
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [INPUT_FOLDER]   Input folder containing markdown files
  [OUTPUT_FOLDER]  Output folder to generate the site [default: `input_folder/site`]

Options:
  -v, --verbose...            Verbosity level (0-4) [default: 0 warn]
  -w, --watch                 Detect changes and rebuild the site automatically
      --serve                 Serve the site with a built-in HTTP server
      --bind <BIND>           Address to bind the server [default: 0.0.0.0:8000]
  -c, --config <CONFIG>       Path to custom configuration file [default: marmite.yaml]
      --init-templates        Initialize templates in the project
      --start-theme <NAME>    Initialize a theme with templates and static assets
      --set-theme <THEME>     Download and set a theme from a remote URL or local folder
      --generate-config       Generate the configuration file
      --init-site             Init a new site with sample content and default configuration
      --force                 Force the rebuild of the site even if no changes detected
      --shortcodes            List all available shortcodes
      --show-urls             Show all site URLs organized by content type
      --skill                 Print the embedded agent skill document (SKILL.md) to stdout
      --skill-install         Install the skill into .agents/skills/
      --skill-install-claude  Install the skill into .claude/skills/ for Claude Code
      --new <TITLE>           Create a new post with the given title
  -e                          Edit the file in the default editor
  -p                          Set the new content as a page
  -t <TAGS>                   Set the tags for the new content (comma separated)
      --site <SITE>           Target site within a workspace
      --skip-image-resize     Skip image resizing during build
      --check-internal-links  Check internal links during build
      --strict-internal-links Fail the build on broken internal links
  -h, --help                  Print help
  -V, --version               Print version
```

> Run `marmite --help` for the full list of options including site metadata overrides,
> search, pagination, and other configuration flags.

### Live reload in development

When running with `--serve --watch` (or `--serve -w`), Marmite automatically rebuilds
on file changes and refreshes the browser via WebSocket. The default theme includes
live reload out of the box. For custom themes, add this snippet to your base template:

```html
<script src="/__marmite__/livereload.js"></script>
```

## Getting started

Read a tutorial on how to get started https://marmite.blog/getting-started.html and create your blog in minutes.


## Docs 

Read more on how to customize templates, add comments etc on https://marmite.blog/ 


## That's all!

**Marmite** is very simple.

If this simplicity does not suit your needs, there are other awesome static site generators.


Here are some that I recommend:

- [Cobalt](https://cobalt-org.github.io/)
- [Zola](https://www.getzola.org/)
- [Zine](https://zineland.github.io/)
