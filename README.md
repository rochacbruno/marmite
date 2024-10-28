# Marmite

<img src="https://github.com/rochacbruno/marmite/raw/main/assets/_resized/logo_160x120.png" align="left" alt="marmite">

Marmite [**Mar**kdown **m**akes s**ite**s] is a **very!** simple static site generator.

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)
[![Crates.io Version](https://img.shields.io/crates/v/marmite)](https://crates.io/crates/marmite)
[![Docs and Demo](https://img.shields.io/badge/docs-demo-blue)](https://rochacbruno.github.io/marmite/)  
  
[![Create blog](https://img.shields.io/badge/CREATE%20YOUR%20BLOG%20WITH%20ONE%20CLICK-20B2AA?style=for-the-badge)](https://github.com/rochacbruno/make-me-a-blog)



> I'm a big user of other SSGs but it is frequently frustrating that it takes so much setup to get started.  
Just having a directory of markdown files and running a single command sounds really useful.  
&mdash; Michael, marmite user.

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
      --debug            Print debug messages
      --init-templates   Initialize templates in the project
      --start-theme      Initialize a theme with templates and static assets
  -h, --help             Print help
  -V, --version          Print version

```


## Getting started

Read a tutorial on how to get started https://rochacbruno.github.io/marmite/getting-started.html and create your blog in minutes.


## Docs 

Read more on how to customize templates, add comments etc on https://rochacbruno.github.io/marmite/ 


## That's all!

**Marmite** is very simple.

If this simplicity does not suit your needs, there are other awesome static site generators.


Here are some that I recommend:

- [Cobalt](https://cobalt-org.github.io/)
- [Zola](https://www.getzola.org/)
