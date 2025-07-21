---
title: "Link Checking with Lychee Integration"
date: 2025-07-21
author: marmite
tags: [docs, features, lychee, link-checking]
stream: alt
---

Marmite now includes built-in link checking functionality using [Lychee](https://lychee.cli.rs/), a fast, async link checker written in Rust.

## Using Link Checking

To check all links in your generated website, use the `--check-links` flag:

```bash
marmite input_folder output_folder --check-links
```

This will:
1. Build your static site
2. Start a temporary HTTP server (automatically enables `--serve`)
3. Scan all HTML files for links
4. Check each link for validity
5. Report any broken links and exit with an error code if found

## How It Works

When you use `--check-links`, Marmite automatically:
- Enables the built-in HTTP server to allow internal link checking
- Calls the Lychee binary to perform comprehensive link checking
- Checks both internal and external links
- Provides detailed reports about link status

## Requirements

The link checking feature requires the Lychee binary to be installed:

```bash
cargo install lychee
```

If Lychee is not installed, Marmite will display a warning and skip the link checking step gracefully.

## Link Checking Results

Lychee provides detailed information about each link:
- ✓ Successful links
- ↳ Redirected links (warnings)
- ✗ Broken links (errors)
- - Excluded links
- ? Unsupported or unknown status links

## Exit Codes

- **0**: All links are working correctly
- **1**: Broken links were found or link checking failed

This makes it perfect for CI/CD pipelines where you want to ensure all links are valid before deploying your site.

## Example Usage

```bash
# Basic link checking
marmite example ./example/public --check-links

# Link checking with custom bind address
marmite example ./example/public --check-links --bind 127.0.0.1:3000

# Watch mode with link checking (checks on each rebuild)
marmite example ./example/public --check-links --watch
```

## Integration with CI/CD

You can use this feature in your CI/CD pipeline to prevent broken links from being deployed:

```yaml
# GitHub Actions example
- name: Build and check links
  run: marmite . ./dist --check-links
```

The command will exit with status code 1 if any broken links are found, causing the CI/CD pipeline to fail.