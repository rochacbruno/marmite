# Marmite

<img src="https://github.com/rochacbruno/marmite/raw/main/assets/_resized/logo_160x120.png" align="left" alt="marmite">

Marmite [**Mar**kdown **m**akes s**ite**s] is a **very!** simple static site generator.

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)
[![Crates.io Version](https://img.shields.io/crates/v/marmite)](https://crates.io/crates/marmite)
[![Docs and Demo](https://img.shields.io/badge/docs-demo-blue)](https://marmite.blog/)  
  
[![Create blog](https://img.shields.io/badge/CREATE%20YOUR%20BLOG%20WITH%20ONE%20CLICK-20B2AA?style=for-the-badge)](https://github.com/rochacbruno/blog)


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
3. The generated static site is a **flat** HTML site, no subpaths, all content is published in extension ending URLS ex: `./{name}.html|rss|json`
4. There are only 2 taxonomies `tags:` (to group similar content together) and `stream:` (to separate content in a different listing) 
5. Marmite uses the `date:` attribute to differentiate `posts` from `pages`

## Features

- Everything embedded in a single binary.
- Zero-Config to get started.
  - optionally fully configurable
- Common-mark + Github Flavoured Markdown + Extensions.
- Raw HTML allowed.
- Emojis `:smile:`, spoiler `||secret||`.
- Wikilinks `[[name|url]]` and Obsidian links `[[page]]`.
- Backlinks.
- Tags.
- Multi authors.
  - Author profile page
- Multi streams.
  - Separate content in different listing
- Pagination.
- Static Search Index.
- RSS Feeds.
  - Multiple feeds (index, tags, authors, streams)
- Built-in HTTP server.
- Auto rebuild when content changes.
- Built-in theme 
  - Light and Dark modes.
  - Multiple colorschemes
  - Fully responsive
  - Spotlight Search.
  - Easy to replace the index page and add custom CSS/JS
  - Easy to customize the templates
  - Math and Mermaid diagrams.
  - Syntax Highlight.
  - Commenting system integration.
  - Banner images and `og:` tags.
- CLI to start a new theme from scratch


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
❯ marmite --help
Marmite is the easiest static site generator.

Usage: marmite [OPTIONS] <INPUT_FOLDER> <OUTPUT_FOLDER>

Arguments:
  <INPUT_FOLDER>   Input folder containing markdown files
  <OUTPUT_FOLDER>  Output folder to generate the site

Options:
      --serve            Serve the site with a built-in HTTP server
      --watch            Detect changes and rebuild the site automatically
      --bind <BIND>      Address to bind the server [default: localhost:8000]
      --config <CONFIG>  Path to custom configuration file [default: marmite.yaml]
      --debug            Print debug messages Deprecated: Use -vv for debug messages
      --init-templates   Initialize templates in the project
      --start-theme      Initialize a theme with templates and static assets
      --generate-config  Generate the configuration file
  -v, --verbose...       Verbosity level (0-4) [default: 0 warn] options: -v: info,-vv: debug,-vvv: trace,-vvvv: trace all
  -h, --help             Print help
  -V, --version          Print version

```

### Live reload in development

When running with `--serve --watch`, Marmite exposes a WebSocket-based live reload helper. Add this snippet to your base template so the browser refreshes after each rebuild:

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
