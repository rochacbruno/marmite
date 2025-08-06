---
tags: installation,python,pip
---
# Python Installation with pip

Marmite can now be installed directly from Python's package index using pip! This makes it incredibly easy to get started with marmite, especially if you're already working in a Python environment.

## Installation

Simply run:

```bash
pip install marmite
```

This will install the marmite binary and make it available in your system PATH.

## Quick Start with uvx (Recommended)

For one-time usage or testing, you can use `uvx` to run marmite without installing it:

```bash
uvx marmite myblog --init-site
uvx marmite myblog --serve --watch
```

Or if you prefer to work from inside the directory:

```bash
uvx marmite myblog --init-site
cd myblog
uvx marmite . --serve --watch
```

## How It Works

The Python package includes:
- The compiled Rust binary for your platform
- A Python wrapper that forwards commands to the native binary
- Cross-platform compatibility for Windows, macOS, and Linux

## Usage

After installation, you can use marmite just like any other command-line tool:

```bash
# Create a new site
marmite myblog --init-site

# Build your site
marmite myblog

# Serve with live reload
marmite myblog --serve --watch
```

Or working from inside the site directory:

```bash
marmite myblog --init-site
cd myblog
marmite . --serve --watch
```

## Alternative Installation Methods

If you prefer other installation methods, marmite is also available via:
- Direct binary downloads from [GitHub Releases](https://github.com/rochacbruno/marmite/releases)
- Building from source with Cargo: `cargo install marmite`
- Package managers (coming soon)

The pip installation method is perfect for:
- Python developers who want to include marmite in their development workflow
- Users who prefer pip for managing command-line tools
- CI/CD pipelines that already use Python environments

